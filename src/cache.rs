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

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::create_dir;
use std::future::Future;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

const CACHE_DIR: &str = ".npm_time_machine_cache";

fn get_cache_file(key: &str) -> PathBuf {
    let cleaned_key = key.replace('/', "||");
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

    let mut f = std::fs::File::create(cache_file.clone())
        .unwrap_or_else(|_| panic!("Can't create cache file: {:?}", &cache_file));
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
    if crate::USE_CACHE.load(Ordering::Relaxed) {
        if let Some(result) = cache_get(key) {
            return result;
        }
    }

    let result = closure().await;
    cache_put(key, &result);
    result
}
