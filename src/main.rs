// package.json loader
// package.json modifier

// serializer for npm data
// cache
// npm query call

// params --no-cache
// -f <...> for package file
// -o <...> outputfile
// git warn
// --no-git
mod cache;
mod changes;
mod npm;
mod pkg_reader;
mod npm_time_machine;
mod error;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use clap::Parser;
use time::macros::format_description;
use time::Date;

use error::AppError;

pub static USE_CACHE: AtomicBool = AtomicBool::new(true);

pub fn date_from_str(value: &str) -> Result<Date, time::error::Parse> {
    let format =
        format_description!("[day padding:zero]-[month padding:zero repr:numerical]-[year]");
    Date::parse(value, &format)
}

/// NPM Time Machine - Move package.json through the time!
///
/// This utility allows to lock package.json into the latest state
/// at given time (but not older than input).
///
/// E.g. with package.json set to React 0.0.1
/// npm_time_machine 27-09-2017 -> React 16.0.0
/// npm_time_machine 21-10-2020 -> React 17.0.0
///
/// but.. for React 18.0.0
/// npm_time_machine 27-09-2017 -x- NO CHANGE
#[derive(Parser, Debug, Clone)]
#[command(name="npm_time_machine")]
pub struct CliArgs {
    /// Date for which to move (format: DD-MM-YYYY)
    #[arg(value_parser=crate::date_from_str)]
    date: Date,
    #[arg(help="input file", short='f', default_value="package.json")]
    input_file: PathBuf,
    #[arg(help="output file", short='o', default_value="package.json.out")]
    output_file: PathBuf,
    /// Don't use / reload cache
    #[arg(long)]
    no_cache: bool,
}

#[tokio::main]
async fn main() {
    let now = Instant::now();
    println!("Processing...\n");
    cache::ensure_cache_dir();

    let args = CliArgs::parse();
    USE_CACHE.swap(!args.no_cache, Ordering::Relaxed);

    match npm_time_machine::run(args.clone()).await {
        Err(AppError::NoPackageFile) => eprintln!("Error: Package input file ({}) couldn't be found. Exiting.", args.input_file.display()),
        Err(AppError::PackageFileNotJson) => eprintln!("Error: Package input file ({}) doesn't seem to be valid JSON. Exiting.", args.input_file.display()),
        _ => println!("Done. Took {} seconds.", now.elapsed().as_secs_f32())
    }
}
