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
use crate::changes::Changes;
use crate::npm::Registry;
use crate::pkg_reader::PkgReader;

use crate::error::AppError;
use crate::CliArgs;

use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use semver::Comparator;
use serde_json::Value;
use tokio::task::JoinSet;

struct TimeMachine {
    changes: Changes,
    registry: Arc<Registry>,
    reader: Arc<PkgReader>,
    args: Arc<CliArgs>,
}

pub async fn run(args: CliArgs) -> Result<(), AppError> {
    cache::ensure_cache_dir();

    let machine = TimeMachine {
        changes: Changes::new(),
        registry: Arc::new(Registry::new()),
        reader: Arc::new(PkgReader::from_path(args.input_file.clone())?),
        args: Arc::new(args),
    };

    machine.load_registry().await;
    machine.find_changes().await;

    if !machine.args.dry_run {
        machine.write_json();
    }

    Ok(())
}

impl TimeMachine {
    pub async fn load_registry(&self) {
        let mut task_set = JoinSet::new();
        let dependencies = self.reader.dependencies();

        for dependency in dependencies.iter().cloned() {
            let registry = self.registry.clone();
            task_set.spawn(async move {
                registry.load(&dependency).await;
                dependency
            });
        }
        while (task_set.join_next().await).is_some() {}
    }

    pub async fn find_changes(&self) {
        let mut task_set = JoinSet::new();
        for dependency in self.reader.dependencies().iter().cloned() {
            let registry = self.registry.clone();
            let reader = self.reader.clone();
            let date = self.args.date;
            let changes = self.changes.clone();

            task_set.spawn(async move {
                let comparator: Comparator = reader.comparator(&dependency);
                let Some(latest_at_date) = registry.get_latest(&dependency, date) else { return };
                let Some(latest_matching) = registry.get_latest_matching(&dependency, &comparator) else { return };

                if std::cmp::Ordering::Greater == latest_at_date.cmp(&latest_matching) {
                    changes.insert(dependency, latest_at_date);
                }
            });
        }
        while (task_set.join_next().await).is_some() {}
    }

    pub fn write_json(&self) {
        let mut json = self.reader.json();
        let dependencies = json.get_mut("dependencies").unwrap();
        for (key, value) in self.changes.clone().into_iter() {
            *dependencies.get_mut(key).unwrap() = Value::from(value.to_string());
        }

        let mut handle = File::create(&self.args.output_file).expect("Can't open file for writing");
        let pretty_string = serde_json::to_string_pretty(&json).unwrap();

        handle.write_all(pretty_string.as_bytes()).unwrap();
    }
}
