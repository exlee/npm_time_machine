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
mod changes;

use clap::Parser;
use semver::Comparator;
use time::Date;
use time::macros::format_description;
use changes::Changes;

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

use std::{path::Path, io::Write};
use std::sync::Arc;
use tokio::task::JoinSet;

use std::collections::HashMap;
use semver::Version;
use std::sync::Mutex;
use serde_json::{ json, Value };


#[tokio::main]
async fn main() {
    cache::ensure_cache_dir();

    let args = CliArgs::parse();
    let changes: Changes = Changes::new();

    let file = Path::new("./sample_data/package.json");
    let reader = Arc::new(pkg_reader::PkgReader::from_path(file.into()));

    //println!("{:?}", args);
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
        let changes = changes.clone();
        task_set.spawn(async move {
            let comparator: Comparator = reader.comparator(&dependency);
            let Some(latest_at_date) = registry.get_latest(&dependency, date) else { return };
            let Some(latest_matching) = registry.get_latest_matching(&dependency, &comparator) else { return };

            if std::cmp::Ordering::Greater == latest_at_date.cmp(&latest_matching) {
                //println!("Matching!");
                changes.insert(dependency.into(), latest_at_date);
            }
        });
    }

    while let Some(_) = task_set.join_next().await {}

    let mut json = reader.json();
    let dependencies = json.get_mut("dependencies").unwrap();

    for (key, value) in Changes::into_inner(changes).into_iter() {
        *dependencies.get_mut(key).unwrap() = Value::from(value.to_string());
    };

    let mut handle = std::fs::File::create("./output.json").expect("Can't open file for writing");
    let pretty_string = serde_json::to_string_pretty(&json).unwrap();

    handle.write(pretty_string.as_bytes()).unwrap();

}
