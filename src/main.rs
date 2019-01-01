mod ansi;
mod error;
mod item;

use std::fs::File;
use std::io::prelude::*;

fn main() {
    let path = match ::std::env::args().nth(1) {
        Some(path) => path,
        None => panic!("missing argument"),
    };
    let mut file = File::open(path).expect("failed to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("failed to read file");
    let items = error::parse(&contents).expect("failed to parse file");
    for item in items {
        println!("{:?}", item);
    }
}
