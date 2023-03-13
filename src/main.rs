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
#[doc(inline)]
pub use std;

#[derive(Debug)]
struct SemanticVersion {}
struct IncorrectSemanticVersion {}

struct PackageJson {
    dependencies: Vec<String>,
}

fn check_for_git() {}

fn read_package_json() -> PackageJson {
    let dependencies: Vec<String> = vec![
        "express".into(),
        // "emotion".into(),
        // "react".into(),
        // "react-router".into(),
        // "redux".into(),
    ];

    PackageJson { dependencies }
}

use std::collections::hash_map::Entry;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

#[derive(Serialize, Deserialize, Debug)]
struct NPMRegistryJSON {
    time: HashMap<String, String>,
}

pub mod cache {
    use serde::de::DeserializeOwned;
    use serde::{Deserialize, Serialize};
    use std::fs::create_dir;
    use std::future::Future;
    use std::io::prelude::*;
    use std::ops::Index;
    use std::path::{Path, PathBuf};

    const CACHE_DIR: &str = ".npm_time_machine_cache";
    fn get_cache_file(key: &str) -> PathBuf {
        Path::new(CACHE_DIR).join(key)
    }

    fn cache_get<T: DeserializeOwned>(key: &str) -> Option<T> {
        let file_open = std::fs::File::open(get_cache_file(key));
        if file_open.is_err() {
            return None;
        }

        Some(serde_json::de::from_reader::<_, T>(file_open.unwrap()).unwrap())
    }

    fn cache_put<T: ?Sized + Serialize>(key: &str, data: &T) {
        let cache_file = get_cache_file(key);

        let mut f = std::fs::File::create(cache_file).expect("Can't create cache file");
        let serialized_string = serde_json::ser::to_string::<T>(data).expect("Can't serialize!");
        f.write_all(&serialized_string.into_bytes())
            .expect("Can't write!");
    }

    pub async fn ensure_cache_dir() {
        let cache_path = Path::new(CACHE_DIR);
        if !cache_path.exists() {
            create_dir(cache_path).expect("Cannot create cache path!")
        }
    }
    pub async fn cached<T, F, Fut>(key: &str, closure: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
        T: Serialize + DeserializeOwned,
    {
        if let Some(cached) = cache_get(key) {
            println!("Returning from cache");
            cached
        } else {
            println!("Downloading info...");
            let result = closure().await;
            cache_put(key, &result);
            result
        }
    }
}

use semver::Version;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

const SEMAPHORE_COUNT: usize = 4;
#[tokio::main]
async fn main() {
    cache::ensure_cache_dir().await;
    check_for_git();
    let semaphore = Arc::new(Semaphore::new(SEMAPHORE_COUNT));
    let mut set = JoinSet::new();

    let package_json = read_package_json();
    let client = reqwest::Client::new();

    for dependency in package_json.dependencies.iter().cloned() {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let client = client.clone();

        set.spawn(async move {
            let result: NPMRegistryJSON =
                cache::cached::<NPMRegistryJSON, _, _>(&format!("CACHE_{dependency}"), || async {
                    client
                        .get(format!("https://registry.npmjs.org/{dependency}"))
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap()
                })
                .await;

            let mut v: Vec<(Version, OffsetDateTime)> = vec![];

            for (key, value) in result.time.into_iter() {
                let version = semver::Version::parse(&key);
                let time = OffsetDateTime::parse(&value, &Rfc3339);

                if let (Ok(version), Ok(time)) = (version, time) {
                    v.push((version, time));
                }
            }
            println!("{:?}", v);
            println!("{} done", dependency);
            drop(permit);
        });
    }

    while let Some(_) = set.join_next().await {}
}
