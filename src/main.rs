use std::process;
use rust_epl::Config;

fn main() {
    let config = Config::parse_cli();
    // println!("{:?}", config);

    if let Err(e) = rust_epl::run(config) {
        println!("Error: {}", e);
        process::exit(1);
    }
}

