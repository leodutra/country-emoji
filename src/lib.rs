#[macro_use]
extern crate lazy_static;

mod countries;

use countries::COUNTRIES_MAP;
// use regex::Regex;

const FLAG_MAGIC_NUMBER: u32 = 127397;

// lazy_static! {
//     static ref FLAG_RE: Regex = Regex::new(r"^[\u{0001F1E6}-\u{0001F1FF}]{2}$").unwrap();
// }

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

pub fn code(input: &str) -> Option<&'static str> {
    flag_to_code(input).or_else(|| name_to_code(input))
}

// pub fn is_country_flag(flag: &str) -> bool {
//     FLAG_RE.is_match(flag)
// }

pub fn flag_to_code(flag: &str) -> Option<&'static str> {
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
    None
}
