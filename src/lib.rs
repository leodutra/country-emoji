mod countries;
use countries::{COUNTRIES, COUNTRIES_CODE_MAP, COUNTRIES_FLAG_MAP};
use once_cell::sync::Lazy;
use regex::Regex;
use unidecode::unidecode;

const FLAG_MAGIC_NUMBER: u32 = 127462 - 65;
static SAINT_ST_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(st\.?\s+)").unwrap()
});

static AMPERSAND_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\s*&\s*").unwrap()
});

static MULTIPLE_SPACES_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\s{2,}").unwrap()
});

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
    normalized = AMPERSAND_REGEX.replace_all(&normalized, " and ").into_owned();
    normalized = SAINT_ST_REGEX.replace_all(&normalized, "saint ").into_owned();
    normalized = MULTIPLE_SPACES_REGEX.replace_all(&normalized, " ").into_owned();

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

    let ambiguous_terms = [
        "korea", "guinea", "congo", "virgin", "samoa", "sudan"
    ];
    for (regex, replacement) in GOVERNMENT_PATTERNS.iter() {
        let stripped = regex.replace_all(text, *replacement).trim().to_string();
        let stripped_lower = stripped.to_lowercase();

        if !stripped.is_empty()
            && stripped != text
            && !variants.contains(&stripped)
            && stripped.len() >= 4
            && !is_too_generic(&stripped_lower)
            && !ambiguous_terms.contains(&stripped_lower.as_str()) {
            variants.push(stripped);
        }
    }

    variants
}

fn is_too_generic(word: &str) -> bool {
    let generic_words = [
        "united", "republic", "democratic", "kingdom", "state", "states",
        "island", "islands", "federation", "people", "socialist", "islamic",
        "principality", "commonwealth", "the", "of", "and", "&",
        "new", "north", "south", "east", "west", "central", "saint", "st"
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
        let primary_words_input: Vec<&str> = input_words.iter()
            .filter(|&&word| !is_too_generic(word))
            .copied()
            .collect();
        let primary_words_country: Vec<&str> = country_words.iter()
            .filter(|&&word| !is_too_generic(word))
            .copied()
            .collect();

        let shared_primary = primary_words_input.iter()
            .any(|&word| primary_words_country.contains(&word));

        if !shared_primary && intersection > 0 {
            return jaccard_score * 0.1;
        }

        return jaccard_score;
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
        .get(trim_upper(code).as_str())
        .map(|x| *x)
}

fn check_by_flag(flag: &str) -> bool {
    COUNTRIES_FLAG_MAP.contains_key(flag.trim())
}

fn get_by_flag(flag: &str) -> Option<&Country> {
    COUNTRIES_FLAG_MAP.get(flag.trim()).map(|x| *x)
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
    code.map_or(false, check_by_code)
}

pub fn code_to_name(code: &str) -> Option<&'static str> {
    get_by_code(code).map(|country| country_name(country))
}

pub fn code_to_flag(code: &str) -> Option<String> {
    get_by_code(code).map(|country| code_to_flag_emoji(country_code(country)))
}

pub fn is_country_flag(flag: &str) -> bool {
    check_by_flag(flag)
}

pub fn flag_to_code(flag: &str) -> Option<&'static str> {
    get_by_flag(flag).map(|country| country_code(country))
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



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_operations() {
        // Test flag generation from codes
        assert_eq!(flag("US"), Some("🇺🇸".to_string()));
        assert_eq!(flag("CL"), Some("🇨🇱".to_string()));
        assert_eq!(flag("JP"), Some("🇯🇵".to_string()));
        assert_eq!(flag("GB"), Some("🇬🇧".to_string()));

        // Test direct code-to-flag API
        assert_eq!(code_to_flag("US"), Some("🇺🇸".to_string()));
        assert_eq!(code_to_flag("us"), Some("🇺🇸".to_string()));

        // Test flag-to-code conversion
        assert_eq!(flag_to_code("🇺🇸"), Some("US"));
        assert_eq!(flag_to_code("🇨🇱"), Some("CL"));
        assert_eq!(flag_to_code("🇯🇵"), Some("JP"));

        // Test invalid inputs
        assert_eq!(flag("XX"), None);
        assert_eq!(code_to_flag("XX"), None);
        assert_eq!(flag_to_code("🇿🇿"), None);
        assert_eq!(flag(""), None);
    }

    #[test]
    fn test_flag_from_name() {
        // Test flag generation from country names
        assert_eq!(flag("United States"), Some("🇺🇸".to_string()));
        assert_eq!(flag("Chile"), Some("🇨🇱".to_string()));
        assert_eq!(flag("Japan"), Some("🇯🇵".to_string()));

        // Test alternative names
        assert_eq!(flag("United States"), Some("🇺🇸".to_string()));
        assert_eq!(flag("UK"), Some("🇬🇧".to_string()));

        // Test partial matches
        assert_eq!(flag("United Kingdom"), Some("🇬🇧".to_string()));

        // Test ambiguous names should return None
        assert_eq!(flag("United"), None); // Could be US, UK, UAE, etc.

        // Test invalid names
        assert_eq!(flag("Atlantis"), None);
    }

    #[test]
    fn test_code_operations() {
        // Test code extraction from flags
        assert_eq!(code("🇺🇸"), Some("US"));
        assert_eq!(code("🇨🇱"), Some("CL"));
        assert_eq!(code("🇯🇵"), Some("JP"));
        assert_eq!(code("🇬🇧"), Some("GB"));

        // Test code extraction from country names
        assert_eq!(code("United States"), Some("US"));
        assert_eq!(code("Chile"), Some("CL"));
        assert_eq!(code("Japan"), Some("JP"));
        assert_eq!(code("UK"), Some("GB"));

        // Test direct name-to-code API
        assert_eq!(name_to_code("United States"), Some("US"));
        assert_eq!(name_to_code("chile"), Some("CL"));

        // Test invalid inputs
        assert_eq!(code("🇿🇿"), None);
        assert_eq!(code("🎌"), None);
        assert_eq!(code("Atlantis"), None);
        assert_eq!(code(""), None);

        // Test ambiguous names - Congo actually matches CD (Democratic Republic of the Congo)
        assert_eq!(code("Congo"), Some("CD")); // Matches "Congo" exactly

        // Test invalid names
        assert_eq!(code("Atlantis"), None);
    }

    #[test]
    fn test_name_operations() {
        // Test name extraction from codes
        assert_eq!(name("US"), Some("United States"));
        assert_eq!(name("CL"), Some("Chile"));
        assert_eq!(name("JP"), Some("Japan"));
        assert_eq!(name("us"), Some("United States"));

        // Test name extraction from flags
        assert_eq!(name("🇺🇸"), Some("United States"));
        assert_eq!(name("🇨🇱"), Some("Chile"));
        assert_eq!(name("🇯🇵"), Some("Japan"));

        // Test direct code-to-name API
        assert_eq!(code_to_name("US"), Some("United States"));
        assert_eq!(code_to_name("us"), Some("United States"));

        // Test invalid inputs
        assert_eq!(name("XX"), None);
        assert_eq!(name("🇿🇿"), None);
        assert_eq!(code_to_name("XX"), None);
        assert_eq!(name(""), None);
    }

    #[test]
    fn test_updated_country_names() {
        // Test countries with updated primary names (our name ordering fixes)
        assert_eq!(name("KP"), Some("North Korea"));
        assert_eq!(name("KR"), Some("South Korea"));
        assert_eq!(name("LA"), Some("Laos"));
        assert_eq!(name("MK"), Some("North Macedonia"));
        assert_eq!(name("SY"), Some("Syria"));
        assert_eq!(name("TZ"), Some("Tanzania"));
        assert_eq!(name("CD"), Some("Congo-Kinshasa"));
        assert_eq!(name("RU"), Some("Russia"));
        assert_eq!(name("BN"), Some("Brunei"));
        assert_eq!(name("VA"), Some("Vatican City"));
        assert_eq!(name("SZ"), Some("Eswatini"));
        assert_eq!(name("LY"), Some("Libya"));
    }

    #[test]
    fn test_new_countries_added() {
        // Test newly added countries/territories
        assert!(name("SS").is_some()); // South Sudan
        assert_eq!(name("SS"), Some("South Sudan"));

        assert!(name("CW").is_some()); // Curaçao
        assert_eq!(name("CW"), Some("Curaçao"));

        assert!(name("SX").is_some()); // Sint Maarten
        assert_eq!(name("SX"), Some("Sint Maarten"));

        assert!(name("BQ").is_some()); // Caribbean Netherlands
        assert_eq!(name("BQ"), Some("Caribbean Netherlands"));
    }

    #[test]
    fn test_legacy_compatibility() {
        // Test that Netherlands Antilles is kept for legacy compatibility
        // Even though dissolved in 2010, it may still be referenced in legacy systems
        assert_eq!(name("AN"), Some("Netherlands Antilles"));
        assert_eq!(code("Netherlands Antilles"), Some("AN"));
        assert!(flag("AN").is_some()); // Should still generate flag emoji
    }

    #[test]
    fn test_alternative_names() {
        // Test that alternative names still work for lookup
        assert_eq!(code("Russian Federation"), Some("RU"));
        assert_eq!(code("Korea, Democratic People's Republic of"), Some("KP"));
        assert_eq!(code("Korea, Republic of"), Some("KR"));
        assert_eq!(code("Lao People's Democratic Republic"), Some("LA"));
        assert_eq!(code("Republic of North Macedonia"), Some("MK"));
        assert_eq!(code("Syrian Arab Republic"), Some("SY"));
        assert_eq!(code("United Republic of Tanzania"), Some("TZ"));
        assert_eq!(code("Holy See (Vatican City State)"), Some("VA"));
        assert_eq!(code("Swaziland"), Some("SZ")); // Old name should still work
    }

    #[test]
    fn test_is_code_validation() {
        // Test valid codes
        assert!(is_code(Some("US")));
        assert!(is_code(Some("CL")));
        assert!(is_code(Some("JP")));

        // Test case insensitivity
        assert!(is_code(Some("us")));
        assert!(is_code(Some("Us")));

        // Test invalid codes
        assert!(!is_code(Some("XX")));
        assert!(!is_code(Some("123")));
        assert!(!is_code(Some("")));
        assert!(!is_code(None));
    }

    #[test]
    fn test_is_country_flag_validation() {
        // Test valid flags
        assert!(is_country_flag("🇺🇸"));
        assert!(is_country_flag("🇨🇱"));
        assert!(is_country_flag("🇯🇵"));

        // Test invalid flags
        assert!(!is_country_flag("🇿🇿")); // Invalid country code
        assert!(!is_country_flag("🎌")); // Not a country flag
        assert!(!is_country_flag(""));
        assert!(!is_country_flag("US")); // Code, not flag
    }

    #[test]
    fn test_fuzzy_matching() {
        // Test fuzzy matching for names with different formats
        assert_eq!(name_to_code("Virgin Islands, British"), Some("VG"));
        assert_eq!(name_to_code("British Virgin Islands"), Some("VG"));

        assert_eq!(name_to_code("Moldova, Republic of"), Some("MD"));
        assert_eq!(name_to_code("Republic of Moldova"), Some("MD"));

        // Test partial matches
        assert_eq!(name_to_code("Vatican"), Some("VA")); // Matches "Holy See (Vatican City State)"

        // Test that ambiguous partial matches return None
        assert_eq!(name_to_code("Korea"), None); // Could be North or South Korea
    }

    #[test]
    fn test_edge_cases() {
        // Test empty and whitespace inputs
        assert_eq!(flag(""), None);
        assert_eq!(flag("   "), None);
        assert_eq!(code(""), None);
        assert_eq!(name(""), None);

        // Test invalid characters and patterns
        assert_eq!(code("123"), None);
        assert_eq!(code("@#$%"), None);
        assert_eq!(code("A"), None);
        assert_eq!(code("AB"), None); // Not a valid country code
        assert_eq!(flag("US1"), None);

        // Test very long inputs
        let long_string = "a".repeat(1000);
        assert_eq!(flag(&long_string), None);
        assert_eq!(code(&long_string), None);
        assert_eq!(name(&long_string), None);

        // Test special characters and diacritics
        assert_eq!(code("Côte d'Ivoire"), Some("CI"));
        assert_eq!(code("Cote D'Ivoire"), Some("CI"));
        assert_eq!(name("CI"), Some("Côte d'Ivoire"));
        assert_eq!(name("CW"), Some("Curaçao"));

        // Test punctuation and hyphens
        assert_eq!(code("Guinea-Bissau"), Some("GW"));
        assert_eq!(code("St. Kitts & Nevis"), Some("KN"));
        assert_eq!(code("Holy See (Vatican City State)"), Some("VA"));
    }

    #[test]
    fn test_comprehensive_country_coverage() {
        // Test a sampling of countries from different regions
        let test_countries = vec![
            ("AD", "Andorra"),
            ("BR", "Brazil"),
            ("CN", "China"),
            ("DE", "Germany"),
            ("EG", "Egypt"),
            ("FR", "France"),
            ("GR", "Greece"),
            ("IN", "India"),
            ("IT", "Italy"),
            ("KE", "Kenya"),
            ("MX", "Mexico"),
            ("NG", "Nigeria"),
            ("PL", "Poland"),
            ("ZA", "South Africa"),
        ];

        for (code_str, expected_name) in test_countries {
            assert_eq!(name(code_str), Some(expected_name));
            assert_eq!(code(expected_name), Some(code_str));
            assert!(flag(code_str).is_some());
            assert!(flag(expected_name).is_some());
        }
    }

    #[test]
    fn test_diacritic_variations() {
        // Test that diacritics don't prevent matching (if implemented)
        // These should all match regardless of accent marks
        assert_eq!(code("Cote d'Ivoire"), Some("CI"));
        assert_eq!(code("Côte d'Ivoire"), Some("CI"));
        assert_eq!(code("COTE D'IVOIRE"), Some("CI"));

        // Test other countries with diacritics
        assert_eq!(name("CW"), Some("Curaçao"));
        // These should work if diacritic normalization is implemented
        // assert_eq!(code("Curacao"), Some("CW")); // Without diacritic
        // assert_eq!(code("CURACAO"), Some("CW"));
    }

    #[test]
    fn test_and_ampersand_variations() {
        // Test "and" vs "&" handling in country names
        // Bosnia and Herzegovina
        assert_eq!(code("Bosnia and Herzegovina"), Some("BA"));
        // Current system doesn't handle "&" variations - this would need enhancement
        // assert_eq!(code("Bosnia & Herzegovina"), Some("BA"));

        // Antigua and Barbuda
        assert_eq!(code("Antigua and Barbuda"), Some("AG"));
        // assert_eq!(code("Antigua & Barbuda"), Some("AG"));

        // Trinidad and Tobago
        assert_eq!(code("Trinidad and Tobago"), Some("TT"));
        // assert_eq!(code("Trinidad & Tobago"), Some("TT"));

        // Saint Vincent & the Grenadines (this is the actual name in data)
        assert_eq!(code("Saint Vincent & the Grenadines"), Some("VC"));
        // This would need enhancement to work:
        // assert_eq!(code("Saint Vincent and the Grenadines"), Some("VC"));
    }

    #[test]
    fn test_government_title_variations() {
        // Test that government titles don't prevent matching
        // These are exact matches that should work
        assert_eq!(code("Russian Federation"), Some("RU"));
        // Check what the actual names are in the data:
        assert_eq!(code("Korea, Democratic People's Republic of"), Some("KP"));
        assert_eq!(code("Korea, Republic of"), Some("KR"));

        // If enhanced matching is implemented, these should also work:
        // assert_eq!(code("Democratic People's Republic of Korea"), Some("KP"));
        // assert_eq!(code("Republic of Korea"), Some("KR"));
        // assert_eq!(code("Republic of France"), Some("FR")); // Not an official name
        // assert_eq!(code("Kingdom of Spain"), Some("ES")); // Not an official name
        // assert_eq!(code("United States of America"), Some("US"));
    }

    #[test]
    fn test_case_insensitivity_edge_cases() {
        // Test various case combinations
        assert_eq!(code("UNITED STATES"), Some("US"));
        assert_eq!(code("united states"), Some("US"));
        assert_eq!(code("United States"), Some("US"));
        assert_eq!(code("UnItEd StAtEs"), Some("US"));

        // Test with other countries
        assert_eq!(code("UNITED KINGDOM"), Some("GB"));
        assert_eq!(code("united kingdom"), Some("GB"));
        assert_eq!(code("United Kingdom"), Some("GB"));

        // Test abbreviated forms
        assert_eq!(code("UK"), Some("GB"));
        assert_eq!(code("uk"), Some("GB"));
        assert_eq!(code("Uk"), Some("GB"));
        assert_eq!(code("uK"), Some("GB"));
    }

    #[test]
    fn test_ambiguous_names() {
        // Test names that could match multiple countries should return None or handle appropriately

        // "Guinea" could match Guinea, Guinea-Bissau, Equatorial Guinea, Papua New Guinea
        // The current implementation should handle this appropriately
        let guinea_result = code("Guinea");
        // Should either return None (ambiguous) or match the exact "Guinea" country (GN)
        assert!(guinea_result == None || guinea_result == Some("GN"));

        // "Korea" could match North or South Korea - should be ambiguous
        assert_eq!(code("Korea"), None);

        // "Congo" could match Democratic Republic of Congo or Republic of Congo
        // Should either be None or match the one with exact "Congo" name
        let congo_result = code("Congo");
        assert!(congo_result == None || congo_result == Some("CD"));
    }

    #[test]
    fn test_alternative_and_historical_names() {
        // Test alternative names that are officially recognized
        assert_eq!(code("Swaziland"), Some("SZ")); // Old name for Eswatini
        assert_eq!(name("SZ"), Some("Eswatini")); // New official name

        // Test other alternative names
        assert_eq!(code("Burma"), Some("MM")); // Old name for Myanmar
        assert_eq!(name("MM"), Some("Myanmar")); // Current name

        // Test abbreviated forms
        assert_eq!(code("UAE"), Some("AE"));
        assert_eq!(code("UK"), Some("GB"));
        assert_eq!(code("USA"), Some("US"));
    }

    #[test]
    fn test_territory_and_dependency_handling() {
        // Test various territories and dependencies
        assert_eq!(code("Puerto Rico"), Some("PR"));
        assert_eq!(code("Guam"), Some("GU"));
        assert_eq!(code("American Samoa"), Some("AS"));
        // "Virgin Islands" is ambiguous - need to be more specific
        assert_eq!(code("U.S. Virgin Islands"), Some("VI"));
        assert_eq!(code("US Virgin Islands"), Some("VI"));

        // British territories
        assert_eq!(code("British Virgin Islands"), Some("VG"));
        assert_eq!(code("Cayman Islands"), Some("KY"));
        assert_eq!(code("Bermuda"), Some("BM"));

        // French territories
        assert_eq!(code("French Guiana"), Some("GF"));
        assert_eq!(code("Martinique"), Some("MQ"));
        assert_eq!(code("Guadeloupe"), Some("GP"));
    }



    #[test]
    fn test_whitespace_normalization() {
        // Test basic whitespace trimming (this should work)
        assert_eq!(code("  United States  "), Some("US"));

        // These advanced whitespace normalizations would need enhancement:
        // assert_eq!(code("United   States"), Some("US")); // Multiple spaces
        // assert_eq!(code("United\tStates"), Some("US")); // Tab character
        // assert_eq!(code("United\nStates"), Some("US")); // Newline character

        // Test with other countries
        assert_eq!(code("  United Kingdom  "), Some("GB"));
        // assert_eq!(code("New   Zealand"), Some("NZ")); // Multiple spaces would need enhancement
    }

    #[test]
    fn test_partial_name_matching() {
        // Test that partial matches work appropriately
        assert_eq!(code("Vatican"), Some("VA")); // Matches "Holy See (Vatican City State)"

        // Test that overly generic partial matches are rejected
        assert_eq!(code("United"), None); // Too ambiguous - could be US, UK, UAE, etc.
        assert_eq!(code("Republic"), None); // Too generic
        assert_eq!(code("Island"), None); // Too generic
        assert_eq!(code("New"), None); // Too generic - New Zealand, New Caledonia, etc.
    }



    #[test]
    fn test_flag_emoji_edge_cases() {
        // Test flag generation for special cases
        assert!(flag("US").is_some());
        assert!(flag("GB").is_some());
        assert!(flag("FR").is_some());

        // Test that invalid codes don't generate flags
        assert_eq!(flag("XX"), None);
        assert_eq!(flag("ZZ"), None);
        assert_eq!(flag("123"), None);

        // Test flag recognition
        assert_eq!(code("🇺🇸"), Some("US"));
        assert_eq!(code("🇬🇧"), Some("GB"));
        assert_eq!(code("🇫🇷"), Some("FR"));

        // Test invalid flag emojis
        assert_eq!(code("🎌"), None); // Japanese flag emoji, not country flag
        assert_eq!(code("🏴"), None); // Black flag, not country
    }

    #[test]
    fn test_saint_st_equivalence() {
        // Test Saint vs St. vs St equivalence
        assert_eq!(code("Saint Lucia"), Some("LC"));
        assert_eq!(code("St. Lucia"), Some("LC"));
        assert_eq!(code("St Lucia"), Some("LC")); // Without period

        assert_eq!(code("Saint Martin"), Some("MF"));
        assert_eq!(code("St. Martin"), Some("MF"));
        assert_eq!(code("St Martin"), Some("MF")); // Without period

        assert_eq!(code("Saint Helena"), Some("SH"));
        assert_eq!(code("St. Helena"), Some("SH"));
        assert_eq!(code("St Helena"), Some("SH")); // Without period

        // Test with multiple Saint entries
        assert_eq!(code("Saint Kitts & Nevis"), Some("KN"));
        assert_eq!(code("St. Kitts & Nevis"), Some("KN"));
        assert_eq!(code("St Kitts & Nevis"), Some("KN")); // Without period

        assert_eq!(code("Saint Vincent & the Grenadines"), Some("VC"));
        assert_eq!(code("St. Vincent & the Grenadines"), Some("VC"));
        assert_eq!(code("St Vincent & the Grenadines"), Some("VC")); // Without period

        assert_eq!(code("Saint Pierre & Miquelon"), Some("PM"));
        assert_eq!(code("St. Pierre & Miquelon"), Some("PM"));
        assert_eq!(code("St Pierre & Miquelon"), Some("PM")); // Without period
    }

    #[test]
    fn test_comma_reversal_patterns() {
        // Test that comma-reversed names work correctly due to fuzzy matching

        // Virgin Islands cases - both should work
        assert_eq!(code("U.S. Virgin Islands"), Some("VI"));
        assert_eq!(code("Virgin Islands, U.S."), Some("VI"));
        assert_eq!(code("Virgin Islands US"), Some("VI"));

        // Korean cases
        assert_eq!(code("North Korea"), Some("KP"));
        assert_eq!(code("Korea, Democratic People's Republic of"), Some("KP"));
        assert_eq!(code("Democratic People's Republic of Korea"), Some("KP"));

        assert_eq!(code("South Korea"), Some("KR"));
        assert_eq!(code("Korea, Republic of"), Some("KR"));
        assert_eq!(code("Republic of Korea"), Some("KR"));

        // Congo cases
        assert_eq!(code("Congo-Kinshasa"), Some("CD"));
        assert_eq!(code("Congo, The Democratic Republic of the"), Some("CD"));
        assert_eq!(code("Democratic Republic of the Congo"), Some("CD"));

        // British Virgin Islands
        assert_eq!(code("Virgin Islands, British"), Some("VG"));
        assert_eq!(code("British Virgin Islands"), Some("VG"));
    }

    #[test]
    fn test_comprehensive_fuzzy_matching() {
        // Test comprehensive fuzzy matching capabilities
        assert_eq!(code("United States"), Some("US"));
        assert_eq!(code("UK"), Some("GB"));
        assert_eq!(code("UAE"), Some("AE"));
        assert_eq!(code("Vatican"), Some("VA"));
        assert_eq!(code("Virgin Islands, British"), Some("VG"));
        assert_eq!(code("British Virgin Islands"), Some("VG"));
        assert_eq!(code("Bosnia & Herzegovina"), Some("BA"));
        assert_eq!(code("Republic of Moldova"), Some("MD"));
        assert_eq!(code("Democratic People's Republic of Korea"), Some("KP"));
        assert_eq!(code("United   States"), Some("US"));
        assert_eq!(code("Curacao"), Some("CW"));

        // Test ambiguous term rejection
        assert_eq!(code("United"), None);
        assert_eq!(code("Korea"), None);
    }
}
