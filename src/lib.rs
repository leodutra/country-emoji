//! # country-emoji
//!
//! country-emoji provides conversions between country names, ISO 3166-1 alpha-2 codes,
//! and Unicode flag emojis.
//!
//! The crate supports exact lookups by code or flag emoji, plus alias-aware and normalized
//! matching for country names.
//!
//! ## Features
//!
//! - Exact lookups backed by precomputed tables for codes, flags, and normalized names
//! - Country-name normalization for case, whitespace, diacritics, and common abbreviations
//! - Alias, formal-name, and fuzzy matching for text input
//! - Bidirectional conversions between names, ISO codes, and flag emojis
//! - Borrowed string returns for code and name APIs where possible
//!
//! ## Quick Start
//!
//! ```rust
//! use country_emoji::{code, flag, name};
//!
//! // Convert a country code to a flag emoji.
//! assert_eq!(flag("US"), Some("🇺🇸".to_string()));
//!
//! // Convert a flag emoji to a country code.
//! assert_eq!(code("🇨🇦"), Some("CA"));
//!
//! // Convert a country code to a display name.
//! assert_eq!(name("DE"), Some("Germany"));
//!
//! // Convert a country name to a code.
//! assert_eq!(code("United Kingdom"), Some("GB"));
//! assert_eq!(code("UAE"), Some("AE"));
//! ```
//!
//! ## Name Matching
//!
//! Text lookups support several normalized and alias-based variations:
//!
//! ```rust
//! use country_emoji::code;
//!
//! // Formal names and government titles.
//! assert_eq!(code("Republic of Korea"), Some("KR"));
//! assert_eq!(code("United States of America"), Some("US"));
//!
//! // Saint/St. normalization.
//! assert_eq!(code("Saint Lucia"), Some("LC"));
//! assert_eq!(code("St. Lucia"), Some("LC"));
//!
//! // Diacritic handling.
//! assert_eq!(code("Cote d'Ivoire"), Some("CI"));
//! assert_eq!(code("Côte d'Ivoire"), Some("CI"));
//!
//! // And/ampersand equivalence.
//! assert_eq!(code("Bosnia and Herzegovina"), Some("BA"));
//! assert_eq!(code("Bosnia & Herzegovina"), Some("BA"));
//! ```
//!
//! ## Explicit Conversion APIs
//!
//! When the input type is already known, use the direct conversion functions:
//!
//! ```rust
//! use country_emoji::{code_to_flag, code_to_name, flag_to_code, name_to_code};
//!
//! assert_eq!(code_to_flag("FR"), Some("🇫🇷".to_string()));
//! assert_eq!(flag_to_code("🇮🇹"), Some("IT"));
//! assert_eq!(name_to_code("Spain"), Some("ES"));
//! assert_eq!(code_to_name("BR"), Some("Brazil"));
//! ```

mod countries;
use countries::{country_code_index_from_bytes, COUNTRIES, COUNTRIES_BY_CODE_INDEX};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use unidecode::unidecode;

// Unicode regional indicator symbols start at U+1F1E6, which corresponds to 'A'.
// This lets us convert between ASCII country-code letters and flag-symbol letters.
const REGIONAL_INDICATOR_START: u32 = FLAG_MAGIC_NUMBER + b'A' as u32;
const FLAG_MAGIC_NUMBER: u32 = 127462 - 65;
const GOVERNMENT_PREFIXES: &[&str] = &[
    "the ",
    "federal republic of ",
    "republic of ",
    "democratic republic of ",
    "people's republic of ",
    "kingdom of ",
    "principality of ",
    "federation of ",
    "state of ",
    "commonwealth of ",
    "united states of ",
    "islamic republic of ",
    "socialist republic of ",
];

const GOVERNMENT_SUFFIXES: &[&str] = &[
    " republic",
    " federation",
    " kingdom",
    " islands",
    " island",
];

const AMBIGUOUS_STRIPPED_TERMS: &[&str] = &["korea", "guinea", "congo", "virgin", "samoa", "sudan"];

const GENERIC_WORDS: &[&str] = &[
    "united",
    "republic",
    "democratic",
    "kingdom",
    "state",
    "states",
    "island",
    "islands",
    "federation",
    "people",
    "socialist",
    "islamic",
    "principality",
    "commonwealth",
    "the",
    "of",
    "and",
    "&",
    "new",
    "north",
    "south",
    "east",
    "west",
    "central",
    "saint",
    "st",
    "sao",
    "tome",
    "principe",
];

pub(crate) type Country = (&'static str, &'static [&'static str]);

use std::sync::Arc;

struct NormalizedNameData {
    text: Arc<str>,
    words: Box<[Arc<str>]>,
}

type NormalizedCountryData = (NormalizedNameData, Vec<NormalizedNameData>, &'static str);

type CountryNameMap = HashMap<Arc<str>, &'static str>;
type WordCountryIndex = HashMap<Arc<str>, Vec<usize>>;

static COUNTRIES_DATA: Lazy<(CountryNameMap, Vec<NormalizedCountryData>, WordCountryIndex)> =
    Lazy::new(|| {
        let mut map: CountryNameMap = HashMap::new();
        let mut normalized_countries = Vec::with_capacity(COUNTRIES.len());
        let mut word_index: WordCountryIndex = HashMap::new();

        // Single Pass: Insert explicit names, build normalized data, and insert derived variants
        // Explicit names use `insert` (overwrite), Derived use `or_insert` (no overwrite)
        // This ensures explicit names always take precedence, regardless of country order.
        for country in COUNTRIES.iter() {
            let code = country.0;
            let names = country.1;
            let country_index = normalized_countries.len();

            // Prepare normalized data
            let primary_normalized_text: Arc<str> = Arc::from(normalize_text(names[0]));
            index_variant_words(&mut word_index, &primary_normalized_text, country_index);
            let primary_normalized = build_normalized_name_data(primary_normalized_text.clone());

            // Note: We insert primary_normalized_text in the loop below as well, but we need the
            // tokenized representation for normalized_countries.

            let mut all_variants = Vec::new();
            for name in names {
                let normalized_name = normalize_text(name);
                let normalized_arc: Arc<str> = Arc::from(normalized_name);

                if normalized_arc.as_ref() != primary_normalized.text.as_ref()
                    && !all_variants.iter().any(|variant: &NormalizedNameData| {
                        variant.text.as_ref() == normalized_arc.as_ref()
                    })
                {
                    index_variant_words(&mut word_index, &normalized_arc, country_index);
                    all_variants.push(build_normalized_name_data(normalized_arc.clone()));
                }
                // Explicit name - Force Insert (Overwrite derived if any)
                map.insert(normalized_arc.clone(), code);

                if let Some(articleless_variant) = remove_articles(normalized_arc.as_ref()) {
                    let articleless_arc: Arc<str> = Arc::from(articleless_variant.as_str());
                    map.entry(articleless_arc.clone()).or_insert(code);
                    index_variant_words(&mut word_index, &articleless_arc, country_index);
                }

                // Add lowercased name to map
                map.insert(Arc::from(name.to_lowercase()), code);

                // Derived variants - Only Insert if Missing
                for variant in strip_government_patterns(normalized_arc.as_ref()) {
                    let variant_arc: Arc<str> = Arc::from(variant.as_str());
                    map.entry(variant_arc.clone()).or_insert(code);
                    index_variant_words(&mut word_index, &variant_arc, country_index);
                }
            }

            normalized_countries.push((primary_normalized, all_variants, code));
        }

        (map, normalized_countries, word_index)
    });

static COUNTRIES_NAME_MAP: Lazy<&HashMap<Arc<str>, &'static str>> = Lazy::new(|| &COUNTRIES_DATA.0);

static NORMALIZED_COUNTRIES: Lazy<&Vec<NormalizedCountryData>> = Lazy::new(|| &COUNTRIES_DATA.1);

static WORD_COUNTRY_INDEX: Lazy<&WordCountryIndex> = Lazy::new(|| &COUNTRIES_DATA.2);

static ALL_COUNTRY_INDICES: Lazy<Vec<usize>> =
    Lazy::new(|| (0..NORMALIZED_COUNTRIES.len()).collect());

fn trim_upper(text: &str) -> String {
    text.trim().to_ascii_uppercase()
}

fn normalize_text(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if trimmed.is_ascii() {
        return normalize_ascii_text(trimmed.as_bytes());
    }

    let normalized = unidecode(trimmed);
    normalize_ascii_text(normalized.as_bytes())
}

fn normalize_ascii_text(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() + 8);
    let mut index = 0;
    let mut pending_space = false;

    while index < bytes.len() {
        let byte = bytes[index].to_ascii_lowercase();

        if byte.is_ascii_whitespace() {
            pending_space = !result.is_empty();
            index += 1;
            continue;
        }

        let at_word_boundary = index == 0 || !bytes[index - 1].is_ascii_alphanumeric();
        if at_word_boundary && byte == b's' && index + 1 < bytes.len() && bytes[index + 1] == b't' {
            let mut next = index + 2;
            if next < bytes.len() && bytes[next] == b'.' {
                next += 1;
            }

            if next < bytes.len() && bytes[next].is_ascii_whitespace() {
                if pending_space && !result.is_empty() {
                    result.push(' ');
                }
                result.push_str("saint");
                pending_space = true;
                while next < bytes.len() && bytes[next].is_ascii_whitespace() {
                    next += 1;
                }
                index = next;
                continue;
            }
        }

        if byte == b'&' {
            if !result.is_empty() && !result.ends_with(' ') {
                result.push(' ');
            }
            result.push_str("and");
            pending_space = true;
            index += 1;
            while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                index += 1;
            }
            continue;
        }

        if pending_space && !result.is_empty() {
            result.push(' ');
        }

        result.push(byte as char);
        pending_space = false;
        index += 1;
    }

    result
}

fn build_normalized_name_data(text: Arc<str>) -> NormalizedNameData {
    let words = text
        .split_whitespace()
        .map(Arc::<str>::from)
        .collect::<Vec<_>>()
        .into_boxed_slice();

    NormalizedNameData { text, words }
}

fn remove_articles(text: &str) -> Option<String> {
    if !text.contains(" the ") {
        return None;
    }

    let articleless = text
        .split_whitespace()
        .filter(|word| *word != "the")
        .collect::<Vec<_>>()
        .join(" ");

    if articleless == text {
        None
    } else {
        Some(articleless)
    }
}

fn strip_government_patterns(normalized: &str) -> Vec<String> {
    let mut variants = vec![normalized.to_string()];

    if normalized.contains(',') {
        let mut name_parts: Vec<&str> = normalized.split(", ").collect();
        name_parts.reverse();
        let reversed_name = name_parts.join(" ");
        if !variants.contains(&reversed_name) {
            variants.push(reversed_name.clone());

            let reversed_variants = strip_government_patterns_internal(&reversed_name);
            for variant in reversed_variants {
                if !variants.contains(&variant) {
                    variants.push(variant);
                }
            }
        }
    }

    let pattern_variants = strip_government_patterns_internal(normalized);
    for variant in pattern_variants {
        if !variants.contains(&variant) {
            variants.push(variant);
        }
    }

    variants
}

fn strip_government_patterns_internal(text: &str) -> Vec<String> {
    let mut variants = Vec::new();

    for prefix in GOVERNMENT_PREFIXES {
        if let Some(stripped) = text.strip_prefix(prefix) {
            push_variant_if_valid(&mut variants, stripped.trim(), text);
        }
    }

    for suffix in GOVERNMENT_SUFFIXES {
        if let Some(stripped) = text.strip_suffix(suffix) {
            push_variant_if_valid(&mut variants, stripped.trim(), text);
        }
    }

    variants
}

fn push_variant_if_valid(variants: &mut Vec<String>, stripped: &str, original: &str) {
    if stripped.is_empty()
        || stripped == original
        || stripped.len() < 4
        || is_too_generic(stripped)
        || AMBIGUOUS_STRIPPED_TERMS.contains(&stripped)
        || variants.iter().any(|variant| variant == stripped)
    {
        return;
    }

    variants.push(stripped.to_string());
}

fn index_variant_words(
    word_index: &mut WordCountryIndex,
    variant: &Arc<str>,
    country_index: usize,
) {
    for word in variant
        .split_whitespace()
        .filter(|word| !is_too_generic(word))
    {
        let entry = word_index.entry(Arc::from(word)).or_default();
        if !entry.contains(&country_index) {
            entry.push(country_index);
        }
    }
}

fn collect_candidate_countries(input_words: &[&str]) -> Option<Vec<usize>> {
    let mut seen = vec![false; NORMALIZED_COUNTRIES.len()];
    let mut candidates = Vec::new();

    for word in input_words
        .iter()
        .copied()
        .filter(|word| !is_too_generic(word))
    {
        let indices = WORD_COUNTRY_INDEX.get(word)?;
        for &country_index in indices.iter() {
            if !seen[country_index] {
                seen[country_index] = true;
                candidates.push(country_index);
            }
        }
    }

    if candidates.is_empty() {
        None
    } else {
        Some(candidates)
    }
}

fn is_too_generic(word: &str) -> bool {
    GENERIC_WORDS.contains(&word)
}

fn contains_country_word(country_words: &[Arc<str>], word: &str) -> bool {
    country_words
        .iter()
        .any(|country_word| country_word.as_ref() == word)
}

fn calculate_similarity_score(
    input: &str,
    input_words: &[&str],
    country_name: &NormalizedNameData,
) -> f32 {
    if input == country_name.text.as_ref() {
        return 1.0;
    }
    let input_len = input.len();
    let country_len = country_name.text.len();
    let length_ratio = if input_len > country_len {
        country_len as f32 / input_len as f32
    } else {
        input_len as f32 / country_len as f32
    };

    if length_ratio < 0.2 {
        return 0.0;
    }
    if country_name.text.contains(input) {
        let containment_score = input_len as f32 / country_len as f32;
        if input_len <= 6 && containment_score < 0.6 {
            return containment_score * 0.3;
        }
        return containment_score;
    }

    if input.contains(country_name.text.as_ref()) {
        return country_len as f32 / input_len as f32;
    }

    let country_words = country_name.words.as_ref();

    let intersection = input_words
        .iter()
        .filter(|word| contains_country_word(country_words, word))
        .count();
    let union = input_words.len() + country_words.len() - intersection;

    if union > 0 {
        let jaccard_score = intersection as f32 / union as f32;

        if input_words.len() == 1 && country_words.len() > 1 {
            return jaccard_score * 0.2;
        }

        let has_shared_primary = input_words
            .iter()
            .any(|&word| !is_too_generic(word) && contains_country_word(country_words, word));

        if !has_shared_primary && intersection > 0 {
            return jaccard_score * 0.1;
        }

        jaccard_score
    } else {
        0.0
    }
}

pub(crate) fn code_to_flag_emoji(code: &str) -> String {
    let mut flag = String::new();
    for c in trim_upper(code).chars() {
        if let Some(c) = std::char::from_u32(c as u32 + FLAG_MAGIC_NUMBER) {
            flag.push(c);
        } else {
            panic!("Could not convert code \"{}\" to flag", code);
        }
    }
    flag
}

fn country_code_index(code: &str) -> Option<usize> {
    country_code_index_from_bytes(code.trim().as_bytes())
}

fn regional_indicator_index(indicator: char) -> Option<usize> {
    let indicator = indicator as u32;

    if (REGIONAL_INDICATOR_START..=REGIONAL_INDICATOR_START + 25).contains(&indicator) {
        Some((indicator - REGIONAL_INDICATOR_START) as usize)
    } else {
        None
    }
}

fn flag_country_index(flag: &str) -> Option<usize> {
    let mut indicators = flag.trim().chars();
    let first = regional_indicator_index(indicators.next()?)?;
    let second = regional_indicator_index(indicators.next()?)?;

    if indicators.next().is_some() {
        return None;
    }

    Some(first * 26 + second)
}

fn check_by_code(code: &str) -> bool {
    get_by_code(code).is_some()
}

fn get_by_code(code: &str) -> Option<&Country> {
    country_code_index(code).and_then(|index| COUNTRIES_BY_CODE_INDEX[index])
}

fn check_by_flag(flag: &str) -> bool {
    get_by_flag(flag).is_some()
}

fn get_by_flag(flag: &str) -> Option<&Country> {
    flag_country_index(flag).and_then(|index| COUNTRIES_BY_CODE_INDEX[index])
}

/// Resolves a flag emoji or country-like text to an ISO 3166-1 alpha-2 code.
///
/// This is the primary lookup entry point. Text input is matched case-insensitively and
/// may resolve through aliases, normalized forms, or fuzzy matching.
///
/// # Arguments
/// * `input` - A flag emoji or country-like text such as `"Canada"`, `"UK"`, or
///   `"United States of America"`
///
/// # Returns
/// * `Some(&str)` - The resolved ISO 3166-1 alpha-2 code
/// * `None` - If the input is invalid, ambiguous, or not found
///
/// # Examples
///
/// ```
/// use country_emoji::code;
///
/// // Flag emoji to code.
/// assert_eq!(code("🇨🇦"), Some("CA"));
/// assert_eq!(code("🇺🇸"), Some("US"));
///
/// // Country names to codes.
/// assert_eq!(code("Canada"), Some("CA"));
/// assert_eq!(code("United States"), Some("US"));
///
/// // Alternative names and abbreviations.
/// assert_eq!(code("UK"), Some("GB"));
/// assert_eq!(code("UAE"), Some("AE"));
///
/// // Formal names.
/// assert_eq!(code("Republic of Korea"), Some("KR"));
/// assert_eq!(code("United States of America"), Some("US"));
///
/// // Invalid or ambiguous inputs.
/// assert_eq!(code("ZZ"), None);
/// assert_eq!(code("Korea"), None);
/// ```
pub fn code(input: &str) -> Option<&'static str> {
    flag_to_code(input).or_else(|| name_to_code(input))
}

/// Resolves a country code or country-like text to a Unicode flag emoji.
///
/// Country codes are handled directly. Other text inputs are first resolved through
/// [`name_to_code`] and then converted to a flag emoji.
///
/// # Arguments
/// * `input` - An ISO country code such as `"US"` or a country name such as
///   `"United States"`
///
/// # Returns
/// * `Some(String)` - The resolved Unicode flag emoji
/// * `None` - If the input is invalid or not found
///
/// # Examples
///
/// ```
/// use country_emoji::flag;
///
/// // Country codes to flags.
/// assert_eq!(flag("US"), Some("🇺🇸".to_string()));
/// assert_eq!(flag("CL"), Some("🇨🇱".to_string()));
///
/// // Country names to flags.
/// assert_eq!(flag("Chile"), Some("🇨🇱".to_string()));
/// assert_eq!(flag("United Kingdom"), Some("🇬🇧".to_string()));
/// assert_eq!(flag("UAE"), Some("🇦🇪".to_string()));
///
/// // Invalid inputs.
/// assert_eq!(flag("XX"), None);
/// assert_eq!(flag("Atlantis"), None);
/// ```
pub fn flag(mut input: &str) -> Option<String> {
    if let Some(flag) = code_to_flag(input) {
        return Some(flag);
    }

    if let Some(code) = name_to_code(input) {
        input = code;
    }

    code_to_flag(input)
}

/// Resolves a flag emoji or ISO 3166-1 alpha-2 code to the preferred country name.
///
/// This function does not perform general name matching. If you need to resolve arbitrary
/// country text first, use [`code`] or [`name_to_code`].
///
/// # Arguments
/// * `input` - A flag emoji such as `"🇶🇦"` or an ISO country code such as `"QA"`
///
/// # Returns
/// * `Some(&str)` - The preferred country name
/// * `None` - If the input is invalid or not found
///
/// # Examples
///
/// ```
/// use country_emoji::name;
///
/// // Flag emoji to name.
/// assert_eq!(name("🇶🇦"), Some("Qatar"));
/// assert_eq!(name("🇨🇦"), Some("Canada"));
///
/// // Country code to name.
/// assert_eq!(name("QA"), Some("Qatar"));
/// assert_eq!(name("CA"), Some("Canada"));
/// assert_eq!(name("GB"), Some("United Kingdom"));
///
/// // Invalid inputs.
/// assert_eq!(name("XX"), None);
/// assert_eq!(name("🏳️"), None);
/// ```
pub fn name(mut input: &str) -> Option<&'static str> {
    if let Some(country_name) = code_to_name(input) {
        return Some(country_name);
    }

    if let Some(code) = flag_to_code(input) {
        input = code;
    }
    code_to_name(input)
}

/// Returns whether an optional string is a valid ISO 3166-1 alpha-2 country code.
///
/// # Arguments
/// * `code` - An optional string slice that may contain a country code
///
/// # Returns
/// * `true` - If `code` is `Some` and resolves to a known country code
/// * `false` - If the code is None or invalid
///
/// # Examples
///
/// ```
/// use country_emoji::is_code;
///
/// assert!(is_code(Some("US")));
/// assert!(is_code(Some("CA")));
/// assert!(!is_code(Some("ZZ")));
/// assert!(!is_code(None));
/// ```
pub fn is_code(code: Option<&str>) -> bool {
    code.is_some_and(check_by_code)
}

/// Converts an ISO 3166-1 alpha-2 country code to the preferred country name.
///
/// This function only accepts country codes. Use [`name`] if the input may be a flag emoji.
///
/// # Arguments
/// * `code` - An ISO 3166-1 alpha-2 country code (case-insensitive)
///
/// # Returns
/// * `Some(&str)` - The preferred country name
/// * `None` - If the code is invalid or not found
///
/// # Examples
///
/// ```
/// use country_emoji::code_to_name;
///
/// assert_eq!(code_to_name("US"), Some("United States"));
/// assert_eq!(code_to_name("gb"), Some("United Kingdom"));
/// assert_eq!(code_to_name("DE"), Some("Germany"));
/// assert_eq!(code_to_name("ZZ"), None);
/// ```
pub fn code_to_name(code: &str) -> Option<&'static str> {
    get_by_code(code).map(|country| country.1[0])
}

/// Converts an ISO 3166-1 alpha-2 country code to its flag emoji.
///
/// This function only accepts country codes. Use [`flag`] if the input may be a country name.
///
/// # Arguments
/// * `code` - An ISO 3166-1 alpha-2 country code (case-insensitive)
///
/// # Returns
/// * `Some(String)` - The corresponding Unicode flag emoji
/// * `None` - If the code is invalid or not found
///
/// # Examples
///
/// ```
/// use country_emoji::code_to_flag;
///
/// assert_eq!(code_to_flag("FR"), Some("🇫🇷".to_string()));
/// assert_eq!(code_to_flag("jp"), Some("🇯🇵".to_string()));
/// assert_eq!(code_to_flag("BR"), Some("🇧🇷".to_string()));
/// assert_eq!(code_to_flag("ZZ"), None);
/// ```
pub fn code_to_flag(code: &str) -> Option<String> {
    get_by_code(code).map(|country| code_to_flag_emoji(country.0))
}

/// Returns whether a string is a valid country flag emoji.
///
/// # Arguments
/// * `flag` - A string that may contain a Unicode country flag emoji
///
/// # Returns
/// * `true` - If the input is a valid country flag emoji
/// * `false` - If the input is not a valid country flag
///
/// # Examples
///
/// ```
/// use country_emoji::is_country_flag;
///
/// assert!(is_country_flag("🇺🇸"));
/// assert!(is_country_flag("🇨🇦"));
/// assert!(is_country_flag("🇬🇧"));
/// assert!(!is_country_flag("🏳️"));
/// assert!(!is_country_flag("US"));
/// assert!(!is_country_flag("🎌"));
/// ```
pub fn is_country_flag(flag: &str) -> bool {
    check_by_flag(flag)
}

/// Converts a country flag emoji to its ISO 3166-1 alpha-2 code.
///
/// This function only accepts flag emojis. Use [`code`] if the input may be country text.
///
/// # Arguments
/// * `flag` - A Unicode country flag emoji
///
/// # Returns
/// * `Some(&str)` - The corresponding ISO 3166-1 alpha-2 country code
/// * `None` - If the flag is invalid or not a country flag
///
/// # Examples
///
/// ```
/// use country_emoji::flag_to_code;
///
/// assert_eq!(flag_to_code("🇺🇸"), Some("US"));
/// assert_eq!(flag_to_code("🇨🇦"), Some("CA"));
/// assert_eq!(flag_to_code("🇬🇧"), Some("GB"));
/// assert_eq!(flag_to_code("🏳️"), None);
/// assert_eq!(flag_to_code("US"), None);
/// ```
pub fn flag_to_code(flag: &str) -> Option<&'static str> {
    get_by_flag(flag).map(|country| country.0)
}

/// Resolves country-like text to an ISO 3166-1 alpha-2 code.
///
/// Text input is matched case-insensitively and may resolve through exact matches,
/// normalized variants, aliases, or fuzzy matching.
///
/// # Supported Name Variations
/// - Official names such as `"United States"` and `"United Kingdom"`
/// - Common aliases such as `"USA"`, `"UK"`, and `"UAE"`
/// - Formal names such as `"Republic of Korea"` and `"United States of America"`
/// - Comma-reversed names such as `"Virgin Islands, British"`
/// - Saint/St. variations such as `"Saint Lucia"`, `"St. Lucia"`, and `"St Lucia"`
/// - Diacritic-insensitive matches such as `"Cote d'Ivoire"` and `"Côte d'Ivoire"`
/// - `and`/`&` normalization such as `"Bosnia and Herzegovina"` and
///   `"Bosnia & Herzegovina"`
///
/// # Arguments
/// * `name` - Country-like text in one of the supported formats
///
/// # Returns
/// * `Some(&str)` - The resolved ISO 3166-1 alpha-2 country code
/// * `None` - If the name is invalid, too ambiguous, or not found
///
/// # Examples
///
/// ```
/// use country_emoji::name_to_code;
///
/// // Official names.
/// assert_eq!(name_to_code("Canada"), Some("CA"));
/// assert_eq!(name_to_code("Germany"), Some("DE"));
///
/// // Aliases and abbreviations.
/// assert_eq!(name_to_code("UK"), Some("GB"));
/// assert_eq!(name_to_code("UAE"), Some("AE"));
/// assert_eq!(name_to_code("USA"), Some("US"));
///
/// // Formal names.
/// assert_eq!(name_to_code("Republic of Korea"), Some("KR"));
/// assert_eq!(name_to_code("United States of America"), Some("US"));
///
/// // Saint/St. normalization.
/// assert_eq!(name_to_code("Saint Lucia"), Some("LC"));
/// assert_eq!(name_to_code("St. Lucia"), Some("LC"));
///
/// // Diacritic handling.
/// assert_eq!(name_to_code("Cote d'Ivoire"), Some("CI"));
/// assert_eq!(name_to_code("Côte d'Ivoire"), Some("CI"));
///
/// // Invalid or ambiguous input.
/// assert_eq!(name_to_code("Atlantis"), None);
/// assert_eq!(name_to_code("Korea"), None);
/// assert_eq!(name_to_code("United"), None);
/// ```
pub fn name_to_code(name: &str) -> Option<&'static str> {
    let trimmed_input = name.trim();
    if trimmed_input.is_empty() {
        return None;
    }

    // Exact/Fast lookups
    let lower_input = trimmed_input.to_lowercase();
    if let Some(&code) = COUNTRIES_NAME_MAP.get(lower_input.as_str()) {
        return Some(code);
    }

    let normalized_input = normalize_text(trimmed_input);
    if let Some(&code) = COUNTRIES_NAME_MAP.get(normalized_input.as_str()) {
        return Some(code);
    }

    // Optimization: pattern matching on input
    for variant in strip_government_patterns(normalized_input.as_str()) {
        if let Some(&code) = COUNTRIES_NAME_MAP.get(variant.as_str()) {
            return Some(code);
        }
    }

    // Fuzzy matching
    let input_words: Vec<&str> = normalized_input.split_whitespace().collect();
    let all_generic = input_words.iter().all(|&word| is_too_generic(word));
    if all_generic && !input_words.is_empty() {
        return None;
    }

    let candidate_indices = collect_candidate_countries(&input_words);
    let candidate_indices = candidate_indices
        .as_deref()
        .unwrap_or(ALL_COUNTRY_INDICES.as_slice());

    let mut best_match: Option<(&'static str, f32)> = None;
    let mut best_score = 0.0f32;

    for &country_index in candidate_indices {
        let (primary_normalized, all_variants, code) = &NORMALIZED_COUNTRIES[country_index];
        if best_score >= 1.0 {
            break;
        }
        let score = calculate_similarity_score(&normalized_input, &input_words, primary_normalized);
        if score > best_score {
            best_match = Some((code, score));
            best_score = score;

            if score >= 1.0 {
                continue;
            }
        }
        for variant in all_variants {
            let score = calculate_similarity_score(&normalized_input, &input_words, variant);
            if score > best_score {
                best_match = Some((code, score));
                best_score = score;

                if score >= 1.0 {
                    break;
                }
            }
        }
    }

    if let Some((code, score)) = best_match {
        let word_count = normalized_input.matches(' ').count() + 1;
        let threshold = if word_count == 1 { 0.4 } else { 0.2 };

        if score >= threshold {
            return Some(code);
        }
    }

    None
}
