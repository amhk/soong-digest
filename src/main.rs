use flate2::read::GzDecoder;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;
use termcolor::ColorChoice;

mod ansi;
mod error;
mod item;
mod output;
mod warning;

use crate::output::{display_items, OutputFormat};

fn try_parse_color_choice(s: &str) -> Result<ColorChoice, &str> {
    match s {
        "auto" => Ok(ColorChoice::Auto),
        "always" => Ok(ColorChoice::Always),
        "never" => Ok(ColorChoice::Never),
        _ => Err("unknown value"),
    }
}

fn try_parse_output_format(s: &str) -> Result<OutputFormat, &str> {
    match s {
        "full" => Ok(OutputFormat::Full),
        "cfile" => Ok(OutputFormat::Cfile),
        _ => Err("unknown value"),
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "soong-digest")]
struct Opt {
    #[structopt(long = "errors", parse(from_os_str))]
    /// Path to errors file
    ///
    /// Typically $ANDROID_ROOT/out/build.log.
    errors: Option<PathBuf>,

    #[structopt(long = "warnings", parse(from_os_str))]
    /// Path to warnings file
    ///
    /// Typically $ANDROID_ROOT/out/verbose.log.gz.
    warnings: Option<PathBuf>,

    #[structopt(
        long = "color",
        default_value = "auto",
        parse(try_from_str = "try_parse_color_choice")
    )]
    /// If and when to use color
    ///
    /// Valid values are: auto, always, never.
    color_choice: ColorChoice,

    #[structopt(
        long = "format",
        default_value = "full",
        parse(try_from_str = "try_parse_output_format")
    )]
    /// Output format template
    ///
    /// Valid values are: full, cfile
    output_format: OutputFormat,
}

fn try_main() -> Result<usize, String> {
    let opt = Opt::from_args();
    let mut total = 0;

    if opt.errors.is_some() {
        let contents = std::fs::read_to_string(opt.errors.unwrap()).expect("failed to read file");
        let iter = error::parse(&contents).expect("failed to parse file");
        total += display_items(iter, opt.color_choice).expect("failed to display errors");
    }

    if opt.warnings.is_some() {
        let raw = std::fs::read(opt.warnings.unwrap()).expect("failed to read file");
        let mut decoder = GzDecoder::new(&*raw);
        let mut contents = String::new();
        decoder
            .read_to_string(&mut contents)
            .expect("failed to decode file");
        let iter = warning::parse(&contents).expect("failed to parse file");
        total += display_items(iter, opt.color_choice).expect("failed to display warnings");
    }

    Ok(total)
}

fn main() {
    exit(match try_main() {
        Ok(n) if n < 0xff => n as i32,
        Ok(_) => 0xff,
        Err(e) => {
            eprintln!("{:?}", e);
            1
        }
    });
}
