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
mod npm;
mod pkg_reader;

use clap::Parser;
use semver::Comparator;
use time::Date;
use time::macros::format_description;

fn check_for_git() {}

pub fn date_from_str(value: &str) -> Result<Date, time::error::Parse> {
    let format = format_description!("[day padding:zero]-[month padding:zero repr:numerical]-[year]");
    Date::parse(value, &format)
}

#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(value_parser=crate::date_from_str)]
    date: Date
}

use std::path::Path;
use std::sync::Arc;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    cache::ensure_cache_dir();

    let args = CliArgs::parse();
    let file = Path::new("./sample_data/package.json");
    let reader = Arc::new(pkg_reader::PkgReader::from_path(file.into()));

    println!("{:?}", args);
    check_for_git();
    let registry = Arc::new(npm::Registry::new());

    let mut task_set = JoinSet::new();

    for dependency in reader.dependencies().iter().cloned() {
        let registry = registry.clone();
        task_set.spawn(async move {
            registry.load(&dependency).await;
            dependency
        });
    }

    // Waiting for the data to load
    while let Some(_) = task_set.join_next().await {}

    let mut task_set = JoinSet::new();
    for dependency in reader.dependencies().iter().cloned() {
        let registry = registry.clone();
        let reader = reader.clone();
        let date = args.date.clone();
        task_set.spawn(async move {
            let comparator: Comparator = reader.comparator(&dependency);
            let Some(latest_at_date) = registry.get_latest(&dependency, date) else { return };
            let Some(latest_matching) = registry.get_latest_matching(&dependency, &comparator) else { return };
            let comparision = latest_at_date.cmp(&latest_matching);

            println!("{dependency}: {:?} --> {:?} ({comparision:?})", latest_at_date.to_string(), latest_matching.to_string());
        });
    }

    while let Some(_) = task_set.join_next().await {}
}
