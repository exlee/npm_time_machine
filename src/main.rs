// Copyright 2023 Przemysław Alexander Kamiński
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod cache;
mod changes;
mod error;
mod npm;
mod npm_time_machine;
mod pkg_reader;
mod printer;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use clap::Parser;
use time::macros::format_description;
use time::Date;

use error::AppError;

pub static USE_CACHE: AtomicBool = AtomicBool::new(true);
pub static VERBOSE: AtomicBool = AtomicBool::new(true);

pub fn date_from_str(value: &str) -> Result<Date, time::error::Parse> {
    let format =
        format_description!("[day padding:zero]-[month padding:zero repr:numerical]-[year]");
    Date::parse(value, &format)
}

/// NPM Time Machine - Move package.json through the time!
///
/// Utility which locks package.json into the latest state at given time (but not older than input).
///
/// Example:
/// For package.json with React 0.0.1:
///
/// npm_time_machine 27-09-2017 ->  React 16.0.0
/// npm_time_machine 21-10-2020 ->  React 17.0.0
///
/// for package.json with React 18.0.0:
///
/// npm_time_machine 27-09-2017 -x- NO CHANGE
#[derive(Parser, Debug, Clone)]
#[command(name = "npm_time_machine", verbatim_doc_comment, arg_required_else_help(true))]
pub struct CliArgs {
    /// Target date (format: DD-MM-YYYY)
    #[arg(value_parser=crate::date_from_str)]
    date: Date,
    #[arg(help = "input file", short = 'f', default_value = "package.json")]
    input_file: PathBuf,
    #[arg(help = "output file", short = 'o', default_value = "package.json.out")]
    output_file: PathBuf,
    /// Don't use / reload cache
    #[arg(long)]
    no_cache: bool,
    /// Silent mode
    #[arg(long)]
    silent: bool,
    /// Dry run - show changes only
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() {
    let now = Instant::now();

    cache::ensure_cache_dir();

    let args = CliArgs::parse();
    printer::print("Processing...\n");
    USE_CACHE.swap(!args.no_cache, Ordering::Relaxed);
    VERBOSE.swap(!args.silent, Ordering::Relaxed);

    match npm_time_machine::run(args.clone()).await {
        Err(AppError::NoPackageFile) => eprintln!(
            "Error: Package input file ({}) couldn't be found. Exiting.",
            args.input_file.display()
        ),
        Err(AppError::PackageFileNotJson) => eprintln!(
            "Error: Package input file ({}) doesn't seem to be valid JSON. Exiting.",
            args.input_file.display()
        ),
        _ => printer::print(&format!("\nDone. Took {} seconds.", now.elapsed().as_secs_f32())),
    }

    if args.dry_run {
        printer::print("\ndry-run - no files were written.\n");
    }
}
