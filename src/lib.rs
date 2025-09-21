// NOTE: update flag() whenever we add 2-letter country names
// TODO: improve using https://github.com/bendodson/flag-emoji-from-country-code/blob/master/FlagPlayground.playground

mod countries;
use countries::{COUNTRIES, COUNTRIES_CODE_MAP, COUNTRIES_FLAG_MAP};

const FLAG_MAGIC_NUMBER: u32 = 127462 - 65;

// Optimized: (code, names_slice)
pub(crate) type Country = (&'static str, &'static [&'static str]);

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
    let name = trim_upper(name);

    // exact match lookup
    for country in COUNTRIES.iter() {
        for n in country_names(country) {
            if n.to_uppercase() == name {
                return Some(country_code(country));
            }
        }
    }

    // inexact match lookup
    let matches = COUNTRIES.iter().fold(vec![], |mut matches, country| {
        for &n in country_names(country) {
            if fuzzy_compare(&name, n) {
                matches.push(country_code(country))
            }
        }
        matches
    });

    // Return only when exactly one match was found
    //   prevents cases like "United"
    if matches.len() == 1 {
        Some(matches[0])
    } else {
        None
    }
}

fn fuzzy_compare(input: &str, name: &str) -> bool {
    let name = name.to_uppercase();

    // Cases like:
    //    "Vatican" <-> "Holy See (Vatican City State)"
    //    "Russia"  <-> "Russian Federation"
    if name.contains(input) || input.contains(&name) {
        return true;
    }

    // Cases like:
    //    "British Virgin Islands" <-> "Virgin Islands, British"
    //    "Republic of Moldova"    <-> "Moldova, Republic of"
    if name.contains(',') {
        let mut name_parts: Vec<&str> = name.split(", ").collect();
        name_parts.reverse();
        let reversed_name = name_parts.join(" ");
        if reversed_name.contains(input) || input.contains(&reversed_name) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_from_code() {
        // Test basic flag generation
        assert_eq!(flag("US"), Some("🇺🇸".to_string()));
        assert_eq!(flag("CL"), Some("🇨🇱".to_string()));
        assert_eq!(flag("JP"), Some("🇯🇵".to_string()));
        assert_eq!(flag("GB"), Some("🇬🇧".to_string()));

        // Test case insensitivity
        assert_eq!(flag("us"), Some("🇺🇸".to_string()));
        assert_eq!(flag("Us"), Some("🇺🇸".to_string()));

        // Test invalid codes
        assert_eq!(flag("XX"), None);
        assert_eq!(flag("123"), None);
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
    fn test_code_from_flag() {
        // Test code extraction from flags
        assert_eq!(code("🇺🇸"), Some("US"));
        assert_eq!(code("🇨🇱"), Some("CL"));
        assert_eq!(code("🇯🇵"), Some("JP"));
        assert_eq!(code("🇬🇧"), Some("GB"));

        // Test invalid flags
        assert_eq!(code("🇿🇿"), None); // Invalid flag
        assert_eq!(code("🎌"), None); // Not a country flag
        assert_eq!(code(""), None);
    }

    #[test]
    fn test_code_from_name() {
        // Test code extraction from country names
        assert_eq!(code("United States"), Some("US"));
        assert_eq!(code("Chile"), Some("CL"));
        assert_eq!(code("Japan"), Some("JP"));

        // Test alternative names
        assert_eq!(code("United States"), Some("US"));
        assert_eq!(code("UK"), Some("GB"));

        // Test case insensitivity
        assert_eq!(code("CHILE"), Some("CL"));
        assert_eq!(code("chile"), Some("CL"));

        // Test ambiguous names - Congo actually matches CD (Democratic Republic of the Congo)
        assert_eq!(code("Congo"), Some("CD")); // Matches "Congo" exactly

        // Test invalid names
        assert_eq!(code("Atlantis"), None);
    }

    #[test]
    fn test_name_from_code() {
        // Test name extraction from codes
        assert_eq!(name("US"), Some("United States"));
        assert_eq!(name("CL"), Some("Chile"));
        assert_eq!(name("JP"), Some("Japan"));

        // Test case insensitivity
        assert_eq!(name("us"), Some("United States"));
        assert_eq!(name("Us"), Some("United States"));

        // Test invalid codes
        assert_eq!(name("XX"), None);
        assert_eq!(name(""), None);
    }

    #[test]
    fn test_name_from_flag() {
        // Test name extraction from flags
        assert_eq!(name("🇺🇸"), Some("United States"));
        assert_eq!(name("🇨🇱"), Some("Chile"));
        assert_eq!(name("🇯🇵"), Some("Japan"));

        // Test invalid flags
        assert_eq!(name("🇿🇿"), None);
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
    fn test_code_to_name_direct() {
        // Test direct code to name conversion
        assert_eq!(code_to_name("US"), Some("United States"));
        assert_eq!(code_to_name("CL"), Some("Chile"));
        assert_eq!(code_to_name("JP"), Some("Japan"));

        // Test case insensitivity
        assert_eq!(code_to_name("us"), Some("United States"));

        // Test invalid codes
        assert_eq!(code_to_name("XX"), None);
        assert_eq!(code_to_name(""), None);
    }

    #[test]
    fn test_code_to_flag_direct() {
        // Test direct code to flag conversion
        assert_eq!(code_to_flag("US"), Some("🇺🇸".to_string()));
        assert_eq!(code_to_flag("CL"), Some("🇨🇱".to_string()));
        assert_eq!(code_to_flag("JP"), Some("🇯🇵".to_string()));

        // Test case insensitivity
        assert_eq!(code_to_flag("us"), Some("🇺🇸".to_string()));

        // Test invalid codes
        assert_eq!(code_to_flag("XX"), None);
        assert_eq!(code_to_flag(""), None);
    }

    #[test]
    fn test_flag_to_code_direct() {
        // Test direct flag to code conversion
        assert_eq!(flag_to_code("🇺🇸"), Some("US"));
        assert_eq!(flag_to_code("🇨🇱"), Some("CL"));
        assert_eq!(flag_to_code("🇯🇵"), Some("JP"));

        // Test invalid flags
        assert_eq!(flag_to_code("🇿🇿"), None);
        assert_eq!(flag_to_code(""), None);
    }

    #[test]
    fn test_name_to_code_direct() {
        // Test direct name to code conversion
        assert_eq!(name_to_code("United States"), Some("US"));
        assert_eq!(name_to_code("Chile"), Some("CL"));
        assert_eq!(name_to_code("Japan"), Some("JP"));

        // Test case insensitivity
        assert_eq!(name_to_code("CHILE"), Some("CL"));
        assert_eq!(name_to_code("chile"), Some("CL"));

        // Test fuzzy matching
        assert_eq!(name_to_code("Vatican"), Some("VA"));
        assert_eq!(name_to_code("Russia"), Some("RU"));

        // Test ambiguous cases - Guinea actually matches GN (Guinea) exactly
        assert_eq!(name_to_code("Guinea"), Some("GN")); // Matches "Guinea" exactly

        // Test invalid names
        assert_eq!(name_to_code("Atlantis"), None);
        assert_eq!(name_to_code(""), None);
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
    fn test_special_characters() {
        // Test countries with special characters
        assert_eq!(code("Côte d'Ivoire"), Some("CI"));
        assert_eq!(code("Cote D'Ivoire"), Some("CI")); // Alternative spelling
        assert_eq!(name("CI"), Some("Côte d'Ivoire"));

        // Test unicode handling
        assert_eq!(name("CW"), Some("Curaçao"));
    }

    #[test]
    fn test_edge_cases() {
        // Test empty and whitespace inputs
        assert_eq!(flag(""), None);
        assert_eq!(flag("   "), None);
        assert_eq!(code(""), None);
        assert_eq!(code("   "), None);
        assert_eq!(name(""), None);
        assert_eq!(name("   "), None);

        // Test very long inputs
        let long_string = "a".repeat(1000);
        assert_eq!(flag(&long_string), None);
        assert_eq!(code(&long_string), None);
        assert_eq!(name(&long_string), None);

        // Test inputs with numbers
        assert_eq!(flag("US1"), None);
        assert_eq!(code("123"), None);
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
}
