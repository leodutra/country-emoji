
#[macro_use]
extern crate lazy_static;

mod countries;
use countries::COUNTRIES;

fn main() {
    println!("Hello, world! {}", COUNTRIES.get("BR").unwrap().get(0).unwrap());
}
