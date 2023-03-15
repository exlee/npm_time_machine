use semver::Comparator;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct PkgReader {
    json: Value,
    dependencies: Vec<String>,
    comparators: HashMap<String, Comparator>,
}

impl PkgReader {
    pub fn from_path(file: PathBuf) -> Self {
        let handle = std::fs::File::open(file).expect("Can't open package file");
        let reader = std::io::BufReader::new(handle);
        let json: Value = serde_json::from_reader(reader).expect("Can't parse to JSON");

        let (dependencies, comparators) = Self::process(&json);

        Self {
            json,
            dependencies,
            comparators,
        }
    }

    pub fn json(&self) -> Value {
        self.json.clone()
    }

    fn process(json: &Value) -> (Vec<String>, HashMap<String, Comparator>) {
        let mut deps: Vec<String> = vec![];
        let mut comps: HashMap<String, Comparator> = HashMap::new();
        let Some(dep_value) = json.get("dependencies") else {
            panic!("Package json doesn't have `dependencies' entry");
        };

        let Some(dep_map) = dep_value.as_object() else {
            panic!("`dependencies' malformed");
        };

        for (key, value) in dep_map.into_iter() {
            let version_string = value.as_str().expect("Version not a string");
            let req = Comparator::parse(version_string).unwrap();

            deps.push(key.into());
            comps.insert(key.into(), req);
        }

        (deps, comps)
    }

    pub fn dependencies(&self) -> Vec<String> {
        self.dependencies.clone()
    }

    pub fn comparator(&self, library: &str) -> Comparator {
        self.comparators.get(library).unwrap().clone()
    }
}
