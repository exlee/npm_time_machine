use std::{sync::{Arc, Mutex}, collections::HashMap};

use semver::Version;

type Inner = HashMap<String,Version>;
pub struct Changes(Arc<Mutex<Inner>>);


impl Changes {
    pub fn new() -> Self {
        return Self(Arc::new(Mutex::new(Inner::new())))
    }

    pub fn insert(&self, key: String, value: semver::Version) {
        let my_clone = self.clone();
        let mut mutable = my_clone.0.lock().unwrap();
        mutable.insert(key, value);
    }

}

impl IntoIterator for Changes {
    type Item = (String,Version);
    type IntoIter = std::collections::hash_map::IntoIter<String, Version>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.lock().unwrap().clone().into_iter()
    }
}

impl Clone for Changes {
    fn clone_from(&mut self, source: &Self)
    {
        *self = source.clone()
    }

    fn clone(&self) -> Self {
        Self(self.0.clone())
    }


}
