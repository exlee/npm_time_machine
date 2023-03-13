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

use clap::Parser;
use time::Date;
use time::macros::format_description;

struct PackageJson {
    dependencies: Vec<String>,
}

fn check_for_git() {}

fn read_package_json() -> PackageJson {
    let dependencies: Vec<String> = vec![
        "express".into(),
        //"emotion".into(),
        //"react".into(),
        // "react-router".into(),
        // "redux".into(),
    ];

    PackageJson { dependencies }
}

pub fn date_from_str(value: &str) -> Result<Date, time::error::Parse> {
    let format = format_description!("[day padding:zero]-[month padding:zero repr:numerical]-[year]");
    Date::parse(value, &format)
}

#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(value_parser=crate::date_from_str)]
    date: Date
}

use std::sync::Arc;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    cache::ensure_cache_dir().await;

    let args = CliArgs::parse();
    println!("{:?}", args);
    check_for_git();
    let registry = Arc::new(npm::Registry::new());
    let package_json = read_package_json();

    let mut task_set = JoinSet::new();

    for dependency in package_json.dependencies.iter().cloned() {
        let registry = registry.clone();
        task_set.spawn(async move {
            registry.load(&dependency).await;
            dependency
        });
    }

    // Waiting for the data to load
    while let Some(_) = task_set.join_next().await {}

    let mut task_set = JoinSet::new();

    for dependency in package_json.dependencies.iter().cloned() {
        let registry = registry.clone();
        let date = args.date.clone();
        task_set.spawn(async move {
            registry.get_latest(dependency, date).await
        });
    }

    while let Some(r) = task_set.join_next().await {
        println!("{:?}", r);
    }
}
