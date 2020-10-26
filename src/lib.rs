// NOTE: update flag() whenever we add 2-letter country names

#[macro_use]
extern crate lazy_static;

mod countries;

use countries::{COUNTRIES, COUNTRIES_MAP};
use regex::Regex;

const FLAG_MAGIC_NUMBER: u32 = 127397;

lazy_static! {
    static ref FLAG_RE: Regex = Regex::new(r"^[\u{0001F1E6}-\u{0001F1FF}]{2}$").unwrap();
    static ref CODE_RE: Regex = Regex::new(r"^(?i)[a-z]{2}$").unwrap();
}

#[derive(Clone)]
pub(crate) struct Country {
    code: &'static str,
    names: Vec<&'static str>,
}

impl Country {
    pub fn name(&self) -> &'static str {
        self.names[0]
    }
}

pub fn code(input: &str) -> Option<&'static str> {
    flag_to_code(input).or_else(|| name_to_code(input))
}

pub fn flag(mut input: &str) -> Option<String> {
	if !CODE_RE.is_match(input) || input == "UK" {
		if let Some(code) = name_to_code(input) {
            input = code;
        }
	}
	code_to_flag(input) 
}

pub fn name(mut input: &str) -> Option<&'static str> {
	if FLAG_RE.is_match(input) {
        if let Some(code) = flag_to_code(input) {
            input = code;
        }
	}
	code_to_name(input)
}

pub fn is_code(code: Option<&str>) -> bool {
    code.map_or(false, |code| {
        COUNTRIES_MAP.contains_key(code.trim().to_uppercase().as_str())
    })
}

pub fn code_to_name(code: &str) -> Option<&'static str> {
    COUNTRIES_MAP
        .get(code.trim().to_uppercase().as_str())
        .map(|country| country.name())
}

pub fn code_to_flag(code: &str) -> Option<String> {
    if is_code(Some(code)) {
        let mut flag = String::new();
        for c in code.chars() {
            if let Some(c) = std::char::from_u32(c as u32 + FLAG_MAGIC_NUMBER) {
                flag.push(c);
            } else {
                return None;
            }
        }
        Some(flag)
    } else {
        None
    }
}

pub fn is_country_flag(flag: &str) -> bool {
    FLAG_RE.is_match(flag)
}

pub fn flag_to_code(flag: &str) -> Option<&'static str> {
    let flag = flag.trim();
    if !is_country_flag(flag) {
        return None;
    }
    let mut code = String::new();
    for c in flag.chars() {
        if let Some(c) = std::char::from_u32(c as u32 - FLAG_MAGIC_NUMBER) {
            code.push(c);
        } else {
            return None;
        }
    }
    COUNTRIES_MAP.get(code.as_str()).map(|country| country.code)
}

pub fn name_to_code(name: &str) -> Option<&'static str> {
    let name = name.trim().to_lowercase();

    // exact match lookup
    for country in COUNTRIES.iter() {
        for n in &country.names {
            if n.to_lowercase() == name {
                return Some(country.code);
            }
        }
    }

    // inexact match lookup
    let matches = COUNTRIES.iter().fold(vec![], 
        |mut matches, country| {
            for &n in &country.names {
                if fuzzy_compare(&name, n) {
                    matches.push(country.code)
                }
            }
            matches
        }
    );

    if matches.len() == 1 {
        Some(matches[0])
    } else {
        None
    }   
}

fn fuzzy_compare(input: &str, name: &str) -> bool {
    let name = name.to_lowercase();

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
