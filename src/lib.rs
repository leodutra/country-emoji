//! # country-emoji
//!
//! A lightweight, fast Rust library for converting between country names, ISO 3166-1 codes, and flag emojis.
//! Features intelligent fuzzy matching, normalization, and comprehensive country data.
//!
//! ## Features
//!
//! - ** Fast lookups** - Optimized for performance with pre-compiled regex patterns
//! - ** Fuzzy matching** - Handles alternative names, government titles, and formatting variations
//! - ** Comprehensive data** - All ISO 3166-1 countries including recent additions
//! - ** Normalization** - Handles diacritics, case-insensitivity, whitespace, and abbreviations
//! - ** Bidirectional conversion** - Convert between any combination of codes, names, and flag emojis
//! - ** Zero-copy** - Returns string slices where possible for optimal memory usage
//!
//! ## Quick Start
//!
//! ```rust
//! use country_emoji::{flag, code, name};
//!
//! // Convert country code to flag emoji
//! assert_eq!(flag("US"), Some("🇺🇸".to_string()));
//!
//! // Convert flag emoji to country code
//! assert_eq!(code("🇨🇦"), Some("CA"));
//!
//! // Convert country code to name
//! assert_eq!(name("DE"), Some("Germany"));
//!
//! // Convert country name to code (with fuzzy matching)
//! assert_eq!(code("United Kingdom"), Some("GB"));
//! assert_eq!(code("UAE"), Some("AE"));  // Handles abbreviations
//! ```
//!
//! ## Advanced Fuzzy Matching
//!
//! The library intelligently handles various name formats and variations:
//!
//! ```rust
//! use country_emoji::code;
//!
//! // Government titles and formal names
//! assert_eq!(code("Republic of Korea"), Some("KR"));
//! assert_eq!(code("United States of America"), Some("US"));
//!
//! // Saint/St. normalization
//! assert_eq!(code("Saint Lucia"), Some("LC"));
//! assert_eq!(code("St. Lucia"), Some("LC"));
//!
//! // Diacritic handling
//! assert_eq!(code("Cote d'Ivoire"), Some("CI"));
//! assert_eq!(code("Côte d'Ivoire"), Some("CI"));
//!
//! // And/ampersand equivalence
//! assert_eq!(code("Bosnia and Herzegovina"), Some("BA"));
//! assert_eq!(code("Bosnia & Herzegovina"), Some("BA"));
//! ```
//!
//! ## Direct API Functions
//!
//! For explicit conversions when you know the input type:
//!
//! ```rust
//! use country_emoji::{code_to_flag, flag_to_code, name_to_code, code_to_name};
//!
//! assert_eq!(code_to_flag("FR"), Some("🇫🇷".to_string()));
//! assert_eq!(flag_to_code("🇮🇹"), Some("IT"));
//! assert_eq!(name_to_code("Spain"), Some("ES"));
//! assert_eq!(code_to_name("BR"), Some("Brazil"));
//! ```

mod countries;
use countries::{COUNTRIES, COUNTRIES_CODE_MAP, COUNTRIES_FLAG_MAP};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use unidecode::unidecode;

const FLAG_MAGIC_NUMBER: u32 = 127462 - 65;
static SAINT_ST_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b(st\.?\s+)").unwrap());

static AMPERSAND_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*&\s*").unwrap());

static MULTIPLE_SPACES_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s{2,}").unwrap());

static GOVERNMENT_PATTERNS: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    vec![
        (Regex::new(r"^the\s+").unwrap(), ""),
        (Regex::new(r"^republic\s+of\s+").unwrap(), ""),
        (Regex::new(r"^democratic\s+republic\s+of\s+").unwrap(), ""),
        (Regex::new(r"^people's\s+republic\s+of\s+").unwrap(), ""),
        (Regex::new(r"^kingdom\s+of\s+").unwrap(), ""),
        (Regex::new(r"^principality\s+of\s+").unwrap(), ""),
        (Regex::new(r"^federation\s+of\s+").unwrap(), ""),
        (Regex::new(r"^state\s+of\s+").unwrap(), ""),
        (Regex::new(r"^commonwealth\s+of\s+").unwrap(), ""),
        (Regex::new(r"^united\s+states\s+of\s+").unwrap(), ""),
        (Regex::new(r"^islamic\s+republic\s+of\s+").unwrap(), ""),
        (Regex::new(r"^socialist\s+republic\s+of\s+").unwrap(), ""),
        (Regex::new(r"\s+republic$").unwrap(), ""),
        (Regex::new(r"\s+federation$").unwrap(), ""),
        (Regex::new(r"\s+kingdom$").unwrap(), ""),
        (Regex::new(r"\s+islands?$").unwrap(), ""),
        (Regex::new(r"\s+island$").unwrap(), ""),
    ]
});

pub(crate) type Country = (&'static str, &'static [&'static str]);

use std::sync::Arc;

type NormalizedCountryData = (Arc<str>, Vec<Arc<str>>, &'static str);

static COUNTRIES_DATA: Lazy<(HashMap<Arc<str>, &'static str>, Vec<NormalizedCountryData>)> =
    Lazy::new(|| {
        let mut map = HashMap::new();
        let mut normalized_countries = Vec::new();

        // Pass 1: Insert all explicit country names and normalized forms
        // This prioritizes exact matches (e.g., "China" for CN) over stripped variants (e.g., "China" from "Republic of China" for TW)
        for country in COUNTRIES.iter() {
            let code = country_code(country);
            let names = country_names(country);

            // Prepare normalized data
            let primary_normalized: Arc<str> = Arc::from(normalize_text(names[0]));
            map.insert(primary_normalized.clone(), code);

            let mut all_variants = Vec::new();
            for name in names {
                let normalized_name = normalize_text(name);
                let normalized_arc: Arc<str> = Arc::from(normalized_name);

                if !all_variants.contains(&normalized_arc) {
                    all_variants.push(normalized_arc.clone());
                }
                map.insert(normalized_arc, code);

                // Add lowercased name to map
                map.insert(Arc::from(name.to_lowercase()), code);
            }

            normalized_countries.push((primary_normalized, all_variants, code));
        }

        // Pass 2: Insert derived variants (stripped government patterns, etc.)
        // Only insert if the key doesn't already exist to avoid overwriting explicit names with ambiguous derivatives
        for country in COUNTRIES.iter() {
            let code = country_code(country);
            let names = country_names(country);

            for name in names {
                for variant in strip_government_patterns(name) {
                    map.entry(Arc::from(variant)).or_insert(code);
                }
            }
        }

        (map, normalized_countries)
    });

static COUNTRIES_NAME_MAP: Lazy<&HashMap<Arc<str>, &'static str>> = Lazy::new(|| &COUNTRIES_DATA.0);

static NORMALIZED_COUNTRIES: Lazy<&Vec<NormalizedCountryData>> = Lazy::new(|| &COUNTRIES_DATA.1);

pub(crate) fn country_code(country: &Country) -> &'static str {
    country.0
}

pub(crate) fn country_names(country: &Country) -> &'static [&'static str] {
    country.1
}

pub(crate) fn country_name(country: &Country) -> &'static str {
    country.1[0]
}

fn trim_upper(text: &str) -> String {
    text.trim().to_uppercase()
}

fn normalize_text(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut normalized = unidecode(trimmed).to_lowercase();
    normalized = AMPERSAND_REGEX
        .replace_all(&normalized, " and ")
        .into_owned();
    normalized = SAINT_ST_REGEX
        .replace_all(&normalized, "saint ")
        .into_owned();
    normalized = MULTIPLE_SPACES_REGEX
        .replace_all(&normalized, " ")
        .into_owned();

    normalized.trim().to_string()
}

fn strip_government_patterns(text: &str) -> Vec<String> {
    let normalized = normalize_text(text);
    let mut variants = vec![normalized.clone()];

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

    let pattern_variants = strip_government_patterns_internal(&normalized);
    for variant in pattern_variants {
        if !variants.contains(&variant) {
            variants.push(variant);
        }
    }

    variants
}

fn strip_government_patterns_internal(text: &str) -> Vec<String> {
    let mut variants = Vec::new();

    let ambiguous_terms = ["korea", "guinea", "congo", "virgin", "samoa", "sudan"];
    for (regex, replacement) in GOVERNMENT_PATTERNS.iter() {
        let stripped = regex.replace_all(text, *replacement).trim().to_string();
        let stripped_lower = stripped.to_lowercase();

        if !stripped.is_empty()
            && stripped != text
            && !variants.contains(&stripped)
            && stripped.len() >= 4
            && !is_too_generic(&stripped_lower)
            && !ambiguous_terms.contains(&stripped_lower.as_str())
        {
            variants.push(stripped);
        }
    }

    variants
}

fn is_too_generic(word: &str) -> bool {
    let generic_words = [
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
    generic_words.contains(&word)
}

fn calculate_similarity_score(input: &str, country_name: &str) -> f32 {
    if input == country_name {
        return 1.0;
    }
    let input_len = input.len();
    let country_len = country_name.len();
    let length_ratio = if input_len > country_len {
        country_len as f32 / input_len as f32
    } else {
        input_len as f32 / country_len as f32
    };

    if length_ratio < 0.2 {
        return 0.0;
    }
    if country_name.contains(input) {
        let containment_score = input_len as f32 / country_len as f32;
        if input_len <= 6 && containment_score < 0.6 {
            return containment_score * 0.3;
        }
        return containment_score;
    }

    if input.contains(country_name) {
        return country_len as f32 / input_len as f32;
    }

    // Optimization: Avoid HashSet allocations
    let input_words: Vec<&str> = input.split_whitespace().collect();
    let country_words: Vec<&str> = country_name.split_whitespace().collect();

    let intersection = input_words
        .iter()
        .filter(|&w| country_words.contains(w))
        .count();
    let union = input_words.len() + country_words.len() - intersection;

    if union > 0 {
        let jaccard_score = intersection as f32 / union as f32;

        if input_words.len() == 1 && country_words.len() > 1 {
            return jaccard_score * 0.2;
        }

        let has_shared_primary = input_words
            .iter()
            .any(|&word| !is_too_generic(word) && country_words.contains(&word));

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

fn check_by_code(code: &str) -> bool {
    COUNTRIES_CODE_MAP.contains_key(trim_upper(code).as_str())
}

fn get_by_code(code: &str) -> Option<&Country> {
    COUNTRIES_CODE_MAP.get(trim_upper(code).as_str()).copied()
}

fn check_by_flag(flag: &str) -> bool {
    COUNTRIES_FLAG_MAP.contains_key(flag.trim())
}

fn get_by_flag(flag: &str) -> Option<&Country> {
    COUNTRIES_FLAG_MAP.get(flag.trim()).copied()
}

/// Convert a flag emoji or country name to its ISO 3166-1 alpha-2 code.
///
/// This is the primary lookup function that accepts both flag emojis and country names,
/// including alternative names, government titles, and various text formats.
/// Uses intelligent fuzzy matching to handle variations in naming conventions.
///
/// # Arguments
/// * `input` - A flag emoji (🇨🇦) or country name ("Canada", "UK", "United States of America")
///
/// # Returns
/// * `Some(&str)` - The ISO 3166-1 alpha-2 country code if found
/// * `None` - If the input is invalid, ambiguous, or not found
///
/// # Examples
///
/// ```
/// use country_emoji::code;
///
/// // Flag emoji to code
/// assert_eq!(code("🇨🇦"), Some("CA"));
/// assert_eq!(code("🇺🇸"), Some("US"));
///
/// // Country names to codes
/// assert_eq!(code("Canada"), Some("CA"));
/// assert_eq!(code("United States"), Some("US"));
///
/// // Alternative names and abbreviations
/// assert_eq!(code("UK"), Some("GB"));
/// assert_eq!(code("UAE"), Some("AE"));
///
/// // Government titles and formal names
/// assert_eq!(code("Republic of Korea"), Some("KR"));
/// assert_eq!(code("United States of America"), Some("US"));
///
/// // Invalid or ambiguous inputs
/// assert_eq!(code("ZZ"), None);  // Invalid code
/// assert_eq!(code("Korea"), None);  // Ambiguous (North or South?)
/// ```
pub fn code(input: &str) -> Option<&'static str> {
    flag_to_code(input).or_else(|| name_to_code(input))
}

/// Convert a country code or name to its flag emoji.
///
/// Accepts both ISO 3166-1 alpha-2 codes and country names (including alternative names).
/// Returns the corresponding Unicode flag emoji as a String.
///
/// # Arguments
/// * `input` - An ISO country code ("US") or country name ("United States")
///
/// # Returns
/// * `Some(String)` - The Unicode flag emoji if the country is found
/// * `None` - If the input is invalid or not found
///
/// # Examples
///
/// ```
/// use country_emoji::flag;
///
/// // Country codes to flags
/// assert_eq!(flag("US"), Some("🇺🇸".to_string()));
/// assert_eq!(flag("CL"), Some("🇨🇱".to_string()));
///
/// // Country names to flags (with fuzzy matching)
/// assert_eq!(flag("Chile"), Some("🇨🇱".to_string()));
/// assert_eq!(flag("United Kingdom"), Some("🇬🇧".to_string()));
/// assert_eq!(flag("UAE"), Some("🇦🇪".to_string()));
///
/// // Invalid inputs
/// assert_eq!(flag("XX"), None);
/// assert_eq!(flag("Atlantis"), None);
/// ```
pub fn flag(mut input: &str) -> Option<String> {
    if let Some(code) = name_to_code(input) {
        input = code;
    }
    code_to_flag(input)
}

/// Convert a flag emoji or country code to its official country name.
///
/// Returns the primary English name for a country given either its flag emoji
/// or ISO 3166-1 alpha-2 code.
///
/// # Arguments
/// * `input` - A flag emoji ("🇶🇦") or ISO country code ("QA")
///
/// # Returns
/// * `Some(&str)` - The official country name if found
/// * `None` - If the input is invalid or not found
///
/// # Examples
///
/// ```
/// use country_emoji::name;
///
/// // Flag emoji to name
/// assert_eq!(name("🇶🇦"), Some("Qatar"));
/// assert_eq!(name("🇨🇦"), Some("Canada"));
///
/// // Country code to name
/// assert_eq!(name("QA"), Some("Qatar"));
/// assert_eq!(name("CA"), Some("Canada"));
/// assert_eq!(name("GB"), Some("United Kingdom"));
///
/// // Invalid inputs
/// assert_eq!(name("XX"), None);
/// assert_eq!(name("🏳️"), None);  // Not a country flag
/// ```
pub fn name(mut input: &str) -> Option<&'static str> {
    if let Some(code) = flag_to_code(input) {
        input = code;
    }
    code_to_name(input)
}

/// Validate if an optional string is a valid ISO 3166-1 alpha-2 country code.
///
/// # Arguments
/// * `code` - An optional string slice that may contain a country code
///
/// # Returns
/// * `true` - If the code is Some and represents a valid country code
/// * `false` - If the code is None or invalid
///
/// # Examples
///
/// ```
/// use country_emoji::is_code;
///
/// assert!(is_code(Some("US")));   // Valid country code
/// assert!(is_code(Some("CA")));   // Valid country code
/// assert!(!is_code(Some("ZZ")));  // Invalid country code
/// assert!(!is_code(None));        // None input
/// ```
pub fn is_code(code: Option<&str>) -> bool {
    code.is_some_and(check_by_code)
}

/// Convert an ISO 3166-1 alpha-2 country code to its official name.
///
/// Direct conversion function that only accepts country codes, not names or flags.
/// For more flexible input handling, use [`name`] instead.
///
/// # Arguments
/// * `code` - An ISO 3166-1 alpha-2 country code (case-insensitive)
///
/// # Returns
/// * `Some(&str)` - The official country name if the code is valid
/// * `None` - If the code is invalid or not found
///
/// # Examples
///
/// ```
/// use country_emoji::code_to_name;
///
/// assert_eq!(code_to_name("US"), Some("United States"));
/// assert_eq!(code_to_name("gb"), Some("United Kingdom"));  // Case insensitive
/// assert_eq!(code_to_name("DE"), Some("Germany"));
/// assert_eq!(code_to_name("ZZ"), None);  // Invalid code
/// ```
pub fn code_to_name(code: &str) -> Option<&'static str> {
    get_by_code(code).map(country_name)
}

/// Convert an ISO 3166-1 alpha-2 country code to its flag emoji.
///
/// Direct conversion function that only accepts country codes, not names or flags.
/// For more flexible input handling, use [`flag`] instead.
///
/// # Arguments
/// * `code` - An ISO 3166-1 alpha-2 country code (case-insensitive)
///
/// # Returns
/// * `Some(String)` - The Unicode flag emoji if the code is valid
/// * `None` - If the code is invalid or not found
///
/// # Examples
///
/// ```
/// use country_emoji::code_to_flag;
///
/// assert_eq!(code_to_flag("FR"), Some("🇫🇷".to_string()));
/// assert_eq!(code_to_flag("jp"), Some("🇯🇵".to_string()));  // Case insensitive
/// assert_eq!(code_to_flag("BR"), Some("🇧🇷".to_string()));
/// assert_eq!(code_to_flag("ZZ"), None);  // Invalid code
/// ```
pub fn code_to_flag(code: &str) -> Option<String> {
    get_by_code(code).map(|country| code_to_flag_emoji(country_code(country)))
}

/// Validate if a string represents a valid country flag emoji.
///
/// # Arguments
/// * `flag` - A string that may contain a Unicode flag emoji
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
/// assert!(is_country_flag("🇺🇸"));   // US flag
/// assert!(is_country_flag("🇨🇦"));   // Canada flag
/// assert!(is_country_flag("🇬🇧"));   // UK flag
/// assert!(!is_country_flag("🏳️"));   // White flag (not a country)
/// assert!(!is_country_flag("US"));   // Text, not emoji
/// assert!(!is_country_flag("🎌"));   // Japanese flag (not country flag emoji)
/// ```
pub fn is_country_flag(flag: &str) -> bool {
    check_by_flag(flag)
}

/// Convert a country flag emoji to its ISO 3166-1 alpha-2 code.
///
/// Direct conversion function that only accepts flag emojis, not names or codes.
/// For more flexible input handling, use [`code`] instead.
///
/// # Arguments
/// * `flag` - A Unicode country flag emoji
///
/// # Returns
/// * `Some(&str)` - The ISO 3166-1 alpha-2 country code if the flag is valid
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
/// assert_eq!(flag_to_code("🏳️"), None);   // Not a country flag
/// assert_eq!(flag_to_code("US"), None);   // Text, not emoji
/// ```
pub fn flag_to_code(flag: &str) -> Option<&'static str> {
    get_by_flag(flag).map(country_code)
}

/// Convert a country name to its ISO 3166-1 alpha-2 code using fuzzy matching.
///
/// This function performs intelligent fuzzy matching to handle various name formats,
/// including alternative names, government titles, diacritic variations, and
/// different naming conventions. It's the most flexible name-to-code conversion function.
///
/// # Supported Name Variations
/// - Official names: "United States", "United Kingdom"
/// - Alternative names: "USA", "UK", "UAE"
/// - Government titles: "Republic of Korea", "United States of America"
/// - Comma-reversed: "Virgin Islands, British", "Korea, Republic of"
/// - Saint/St. variations: "Saint Lucia", "St. Lucia", "St Lucia"
/// - Diacritic handling: "Cote d'Ivoire" ↔ "Côte d'Ivoire"
/// - And/ampersand: "Bosnia and Herzegovina" ↔ "Bosnia & Herzegovina"
///
/// # Arguments
/// * `name` - A country name in various formats (case-insensitive)
///
/// # Returns
/// * `Some(&str)` - The ISO 3166-1 alpha-2 country code if found
/// * `None` - If the name is invalid, too ambiguous, or not found
///
/// # Examples
///
/// ```
/// use country_emoji::name_to_code;
///
/// // Official names
/// assert_eq!(name_to_code("Canada"), Some("CA"));
/// assert_eq!(name_to_code("Germany"), Some("DE"));
///
/// // Alternative names and abbreviations
/// assert_eq!(name_to_code("UK"), Some("GB"));
/// assert_eq!(name_to_code("UAE"), Some("AE"));
/// assert_eq!(name_to_code("USA"), Some("US"));
///
/// // Government titles
/// assert_eq!(name_to_code("Republic of Korea"), Some("KR"));
/// assert_eq!(name_to_code("United States of America"), Some("US"));
///
/// // Saint/St. normalization
/// assert_eq!(name_to_code("Saint Lucia"), Some("LC"));
/// assert_eq!(name_to_code("St. Lucia"), Some("LC"));
///
/// // Diacritic handling
/// assert_eq!(name_to_code("Cote d'Ivoire"), Some("CI"));
/// assert_eq!(name_to_code("Côte d'Ivoire"), Some("CI"));
///
/// // Invalid or ambiguous
/// assert_eq!(name_to_code("Atlantis"), None);     // Non-existent
/// assert_eq!(name_to_code("Korea"), None);        // Ambiguous
/// assert_eq!(name_to_code("United"), None);       // Too vague
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
    for variant in strip_government_patterns(trimmed_input) {
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
    let mut best_match: Option<(&'static str, f32)> = None;
    let mut best_score = 0.0f32;

    for (primary_normalized, all_variants, code) in NORMALIZED_COUNTRIES.iter() {
        if best_score >= 1.0 {
            break;
        }
        let score = calculate_similarity_score(&normalized_input, primary_normalized);
        if score > best_score {
            best_match = Some((code, score));
            best_score = score;

            if score >= 1.0 {
                continue;
            }
        }
        for variant in all_variants {
            let score = calculate_similarity_score(&normalized_input, variant);
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
