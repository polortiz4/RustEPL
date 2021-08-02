extern crate clap;
use std::env;
use rust_epl::Config;

fn main() {
    let config = Config::parse_cli();
    println!("{:?}", config);
}

