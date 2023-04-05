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

use crate::cache;
use semver::{Comparator, Version};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use time::format_description::well_known::Rfc3339;
use time::{Date, OffsetDateTime};
use tokio::sync::{Semaphore, SemaphorePermit};

type VersionInTime = Vec<(Version, OffsetDateTime)>;
type InternalData = Arc<Mutex<HashMap<String, VersionInTime>>>;

const SEMAPHORE_COUNT: usize = 4;
pub struct Registry {
    versions: InternalData,
    semaphore: Semaphore,
    client: reqwest::Client,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            versions: Arc::new(Mutex::new(HashMap::new())),
            semaphore: Semaphore::new(SEMAPHORE_COUNT),
            client: reqwest::Client::new(),
        }
    }

    pub async fn load(&self, library: &str) {
        let result =
            cache::cached::<VersionInTime, _, _>(&(library.to_owned() + ".vit"), || async {
                let permit: SemaphorePermit = self.get_permit().await;
                let result: Result<NPMRegistryJSON, _> = self
                    .client
                    .get(format!("https://registry.npmjs.org/{library}"))
                    .send()
                    .await
                    // HTTP Connection failed
                    .unwrap()
                    .json()
                    .await;

                drop(permit);
                let Ok(result) = result else {
                    return vec![]
                };

                let mut transformed: VersionInTime = vec![];
                for (key, value) in result.time.into_iter() {
                    let version = Version::parse(&key);
                    let time = OffsetDateTime::parse(&value, &Rfc3339);

                    if let (Ok(version), Ok(time)) = (version, time) {
                        transformed.push((version, time))
                    }
                }
                transformed
            })
            .await;
        let mut store = self.versions.lock().unwrap();
        store.insert(library.into(), result);
    }

    async fn get_permit(&self) -> SemaphorePermit {
        self.semaphore.acquire().await.unwrap()
    }

    fn versions_copy(&self, key: &str) -> Result<VersionInTime, ()> {
        if let Some(vector) = self.versions.lock().unwrap().get(key) {
            Ok(vector.clone())
        } else {
            Err(())
        }
    }

    pub fn get_latest(&self, library: &str, date: Date) -> Option<Version> {
        let mut versions: VersionInTime = self
            .versions_copy(library)
            .expect("Library data not loaded!");
        let mut i = 0;

        while i < versions.len() {
            if !versions[i].0.pre.is_empty() {
                versions.remove(i);
                continue;
            }

            if versions[i].1.date() > date {
                versions.remove(i);
                continue;
            }

            i += 1;
        }

        versions.sort_by(|a, b| match a.0.partial_cmp(&b.0) {
            Some(std::cmp::Ordering::Equal) => a.1.partial_cmp(&b.1).unwrap(),
            result => result.unwrap(),
        });

        Some(versions.last()?.0.clone())
    }

    pub fn get_latest_matching(&self, library: &str, comparator: &Comparator) -> Option<Version> {
        let versions: VersionInTime = self
            .versions_copy(library)
            .expect("Library data not loaded!");
        let mut versions: Vec<Version> = versions.into_iter().map(|(v, _)| v).collect();

        versions.sort();

        let mut i = 0;
        while i < versions.len() {
            if !comparator.matches(&versions[i]) {
                versions.remove(i);
                continue;
            }

            i += 1;
        }

        Some(versions.last()?.clone())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct NPMRegistryJSON {
    time: HashMap<String, String>,
}
