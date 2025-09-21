mod countries;
use countries::{COUNTRIES, COUNTRIES_CODE_MAP, COUNTRIES_FLAG_MAP};
use once_cell::sync::Lazy;
use regex::Regex;
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

type NormalizedCountryData = (String, Vec<String>, &'static str);

static NORMALIZED_COUNTRIES: Lazy<Vec<NormalizedCountryData>> = Lazy::new(|| {
    COUNTRIES
        .iter()
        .map(|country| {
            let code = country_code(country);
            let names = country_names(country);
            let primary_normalized = normalize_text(names[0]);

            let mut all_variants = Vec::new();
            for name in names {
                let normalized_name = normalize_text(name);
                if !all_variants.contains(&normalized_name) {
                    all_variants.push(normalized_name.clone());
                }

                let variants = strip_government_patterns(name);
                for variant in variants {
                    if !all_variants.contains(&variant) {
                        all_variants.push(variant);
                    }
                }
            }

            (primary_normalized, all_variants, code)
        })
        .collect()
});

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
    let input_words: std::collections::HashSet<&str> = input.split_whitespace().collect();
    let country_words: std::collections::HashSet<&str> = country_name.split_whitespace().collect();

    let intersection = input_words.intersection(&country_words).count();
    let union = input_words.union(&country_words).count();

    if union > 0 {
        let jaccard_score = intersection as f32 / union as f32;

        if input_words.len() == 1 && country_words.len() > 1 {
            return jaccard_score * 0.2;
        }
        let primary_words_input: Vec<&str> = input_words
            .iter()
            .filter(|&&word| !is_too_generic(word))
            .copied()
            .collect();
        let primary_words_country: Vec<&str> = country_words
            .iter()
            .filter(|&&word| !is_too_generic(word))
            .copied()
            .collect();

        let shared_primary = primary_words_input
            .iter()
            .any(|&word| primary_words_country.contains(&word));

        if !shared_primary && intersection > 0 {
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
    COUNTRIES_CODE_MAP
        .get(trim_upper(code).as_str()).copied()
}

fn check_by_flag(flag: &str) -> bool {
    COUNTRIES_FLAG_MAP.contains_key(flag.trim())
}

fn get_by_flag(flag: &str) -> Option<&Country> {
    COUNTRIES_FLAG_MAP.get(flag.trim()).copied()
}

/// Convert a flag emoji or country name to its ISO code
///
/// # Examples
///
/// ```
/// use country_emoji::code;
///
/// assert_eq!(code("🇨🇦"), Some("CA"));
/// assert_eq!(code("Canada"), Some("CA"));
/// assert_eq!(code("UAE"), Some("AE"));
/// ```
pub fn code(input: &str) -> Option<&'static str> {
    flag_to_code(input).or_else(|| name_to_code(input))
}

/// Convert a country code or name to its flag emoji
///
/// # Examples
///
/// ```
/// use country_emoji::flag;
///
/// assert_eq!(flag("CL"), Some("🇨🇱".to_string()));
/// assert_eq!(flag("Chile"), Some("🇨🇱".to_string()));
/// assert_eq!(flag("XX"), None);
/// ```
pub fn flag(mut input: &str) -> Option<String> {
    if let Some(code) = name_to_code(input) {
        input = code;
    }
    code_to_flag(input)
}

/// Convert a flag emoji or country code to its name
///
/// # Examples
///
/// ```
/// use country_emoji::name;
///
/// assert_eq!(name("🇶🇦"), Some("Qatar"));
/// assert_eq!(name("QA"), Some("Qatar"));
/// assert_eq!(name("XX"), None);
/// ```
pub fn name(mut input: &str) -> Option<&'static str> {
    if let Some(code) = flag_to_code(input) {
        input = code;
    }
    code_to_name(input)
}

pub fn is_code(code: Option<&str>) -> bool {
    code.is_some_and(check_by_code)
}

pub fn code_to_name(code: &str) -> Option<&'static str> {
    get_by_code(code).map(country_name)
}

pub fn code_to_flag(code: &str) -> Option<String> {
    get_by_code(code).map(|country| code_to_flag_emoji(country_code(country)))
}

pub fn is_country_flag(flag: &str) -> bool {
    check_by_flag(flag)
}

pub fn flag_to_code(flag: &str) -> Option<&'static str> {
    get_by_flag(flag).map(country_code)
}

pub fn name_to_code(name: &str) -> Option<&'static str> {
    let trimmed_input = name.trim();
    if trimmed_input.is_empty() {
        return None;
    }
    let upper_input = trimmed_input.to_uppercase();
    for country in COUNTRIES.iter() {
        for country_name in country_names(country) {
            if country_name.to_uppercase() == upper_input {
                return Some(country_code(country));
            }
        }
    }

    let normalized_input = normalize_text(trimmed_input);
    for (primary_normalized, all_variants, code) in NORMALIZED_COUNTRIES.iter() {
        if *primary_normalized == normalized_input {
            return Some(code);
        }
        for variant in all_variants {
            if *variant == normalized_input {
                return Some(code);
            }
        }
    }

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
