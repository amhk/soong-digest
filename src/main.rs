use flate2::read::GzDecoder;
use std::io::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

mod ansi;
mod error;
mod item;
mod warning;

use crate::item::Item;

#[derive(StructOpt, Debug)]
#[structopt(name = "soong-digest")]
struct Opt {
    #[structopt(long = "errors", parse(from_os_str))]
    /// Path to errors file
    ///
    /// Typically $ANDROID_ROOT/out/build.log
    errors: Option<PathBuf>,

    #[structopt(long = "warnings", parse(from_os_str))]
    /// Path to warnings file
    ///
    /// Typically $ANDROID_ROOT/out/verbose.log.gz
    warnings: Option<PathBuf>,
}

fn display_items(items: &[Item]) {
    for item in items {
        println!("{:?}", item);
    }
}

fn main() {
    let opt = Opt::from_args();

    if opt.errors.is_some() {
        let contents = std::fs::read_to_string(opt.errors.unwrap()).expect("failed to read file");
        let items = error::parse(&contents).expect("failed to parse file");
        display_items(&items);
    }

    if opt.warnings.is_some() {
        let raw = std::fs::read(opt.warnings.unwrap()).expect("failed to read file");
        let mut decoder = GzDecoder::new(&*raw);
        let mut contents = String::new();
        decoder
            .read_to_string(&mut contents)
            .expect("failed to decode file");
        let items = warning::parse(&contents).expect("failed to parse file");
        display_items(&items);
    }
}
