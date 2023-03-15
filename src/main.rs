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

use clap::Parser;
use time::macros::format_description;
use time::Date;

fn check_for_git() {}

pub fn date_from_str(value: &str) -> Result<Date, time::error::Parse> {
    let format =
        format_description!("[day padding:zero]-[month padding:zero repr:numerical]-[year]");
    Date::parse(value, &format)
}

use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct CliArgs {
    #[arg(value_parser=crate::date_from_str)]
    date: Date,
    input_file: PathBuf,
    output_file: PathBuf,
}

#[tokio::main]
async fn main() {
    cache::ensure_cache_dir();
    check_for_git();

    let args = CliArgs::parse();

    npm_time_machine::run(args).await;
}
