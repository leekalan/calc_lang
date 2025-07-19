#![feature(never_type)]

pub mod console;
pub mod raw_token;
pub mod run;
pub mod tokens;

use console::console;

pub fn main() {
    console();
}
