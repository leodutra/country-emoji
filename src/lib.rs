// NOTE: update flag() whenever we add 2-letter country names
// TODO: improve using https://github.com/bendodson/flag-emoji-from-country-code/blob/master/FlagPlayground.playground

#[macro_use]
extern crate lazy_static;

mod countries;
use countries::{COUNTRIES, COUNTRIES_CODE_MAP, COUNTRIES_FLAG_MAP};

const FLAG_MAGIC_NUMBER: u32 = 127462 - 65;


#[derive(Clone)]
pub(crate) struct Country {
    code: &'static str,
    flag: String,
    names: Vec<&'static str>,
}

impl Country {
    pub(crate) fn new(code: &'static str, names: Vec<&'static str>) -> Self {
        Country {
            code,
            names,
            flag: internal_code_to_flag(code),
        }
    }

    pub(crate) fn name(&self) -> &'static str {
        self.names[0]
    }
}


fn trim_upper(text: &str) -> String {
    text.trim().to_uppercase()
}

pub(crate) fn internal_code_to_flag(code: &'static str) -> String {
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

fn get_by_code(code: &str) -> Option<&'static Country> {
    COUNTRIES_CODE_MAP
        .get(trim_upper(code).as_str())
        .map(|x| *x)
}

fn check_by_flag(flag: &str) -> bool {
    COUNTRIES_CODE_MAP.contains_key(flag.trim())
}

fn get_by_flag(flag: &str) -> Option<&'static Country> {
    COUNTRIES_FLAG_MAP.get(flag.trim()).map(|x| *x)
}

pub fn code(input: &str) -> Option<&'static str> {
    flag_to_code(input).or_else(|| name_to_code(input))
}

pub fn flag(mut input: &str) -> Option<String> {
    if let Some(code) = name_to_code(input) {
        input = code;
    }
    code_to_flag(input)
}

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
    get_by_code(code).map(|country| country.name())
}

pub fn code_to_flag(code: &str) -> Option<String> {
    get_by_code(code).map(|country| country.flag.clone())
}

pub fn is_country_flag(flag: &str) -> bool {
    check_by_flag(flag)
}

pub fn flag_to_code(flag: &str) -> Option<&'static str> {
    get_by_flag(flag).map(|country| country.code)
}

pub fn name_to_code(name: &str) -> Option<&'static str> {
    let name = trim_upper(name);

    // exact match lookup
    for country in COUNTRIES.iter() {
        for n in &country.names {
            if n.to_uppercase() == name {
                return Some(country.code);
            }
        }
    }

    // inexact match lookup
    let matches = COUNTRIES.iter().fold(vec![], |mut matches, country| {
        for &n in &country.names {
            if fuzzy_compare(&name, n) {
                matches.push(country.code)
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
