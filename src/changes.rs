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

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use semver::Version;

type Inner = HashMap<String, Version>;
pub struct Changes(Arc<Mutex<Inner>>);

impl Changes {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Inner::new())))
    }

    pub fn insert(&self, key: String, value: semver::Version) {
        let my_clone = self.clone();
        let mut mutable = my_clone.0.lock().unwrap();
        mutable.insert(key, value);
    }
}

impl IntoIterator for Changes {
    type Item = (String, Version);
    type IntoIter = std::collections::hash_map::IntoIter<String, Version>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.lock().unwrap().clone().into_iter()
    }
}

impl Clone for Changes {
    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }

    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
