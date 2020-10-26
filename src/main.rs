use country_emoji::{code_to_flag, flag_to_code};

fn main() {
    // println!("Hello, world! {}", COUNTRIES_MAP.get("BR").unwrap().name());
    println!("Hello, world! {} != {}", code_to_flag("BR").unwrap(), "BR");
    println!(
        "Hello, world! {} == {}",
        flag_to_code(code_to_flag("BR").unwrap().as_str()).unwrap(),
        "BR"
    );
    // let flag = code_to_flag(Some("AD")).unwrap();
    // println!("flag = {}", flag);
    // flag_to_code(&flag);
    // let mut chars = flag.chars();
    // println!("0x{:x} 0x{:x}", chars.next().unwrap() as u32, chars.next().unwrap() as u32)
}
