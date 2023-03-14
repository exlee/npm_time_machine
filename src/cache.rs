use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::create_dir;
use std::future::Future;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

const CACHE_DIR: &str = ".npm_time_machine_cache";

fn get_cache_file(key: &str) -> PathBuf {
    let cleaned_key = key.replace("/", "||");
    Path::new(CACHE_DIR).join(cleaned_key)
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

    let mut f = std::fs::File::create(cache_file.clone()).expect(&format!("Can't create cache file: {:?}", &cache_file));
    let serialized_string = serde_json::ser::to_string::<T>(data).expect("Can't serialize!");
    f.write_all(&serialized_string.into_bytes())
        .expect("Can't write!");
}

pub fn ensure_cache_dir() {
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
        //println!("Returning from cache");
        cached
    } else {
        println!("Loading info for {key}...");
        let result = closure().await;
        cache_put(key, &result);
        result
    }
}
