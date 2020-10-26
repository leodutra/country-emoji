use country_emoji::{code, code_to_flag, flag, flag_to_code, name};

fn main() {
    // println!("Hello, world! {}", COUNTRIES_MAP.get("BR").unwrap().name());
    println!("Hello, world! {} != {}", code_to_flag("BR").unwrap(), "BR");
    println!(
        "Hello, world! {} == {}",
        flag_to_code(code_to_flag("BR").unwrap().as_str()).unwrap(),
        "BR"
    );
    println!(
        "Hello, world! {} == {}",
        code("Republic of Moldova").unwrap(),
        "MD"
    );
    println!(
        "Hello, world! {} == {}",
        flag("Republic of Moldova").unwrap(),
        "🇲🇩"
    );
    println!("Hello, world! {} == {}", flag("UK").unwrap(), "🇬🇧");
    println!(
        "Hello, world! {} == {}",
        name("🇬🇧").unwrap(),
        "United Kingdom"
    );
    println!(
        "Hello, world! {} == {}",
        name("GB").unwrap(),
        "United Kingdom"
    );
    // println!(
    //     "{}",
    //     CODE_RE.is_match("U1")
    // )
    // let flag = code_to_flag(Some("AD")).unwrap();
    // println!("flag = {}", flag);
    // flag_to_code(&flag);
    // let mut chars = flag.chars();
    // println!("0x{:x} 0x{:x}", chars.next().unwrap() as u32, chars.next().unwrap() as u32)
}
