use std::{sync::{Arc, Mutex}, collections::HashMap};

use semver::Version;

type Inner = HashMap<String,Version>;
pub struct Changes(Arc<Mutex<Inner>>);


impl Changes {
    pub fn new() -> Self {
        return Self(Arc::new(Mutex::new(Inner::new())))
    }

    pub fn into_inner(obj: Self) -> Inner {
        let mutex = Arc::try_unwrap(obj.0).unwrap();
        mutex.into_inner().unwrap()

    }
    pub fn insert(&self, key: String, value: semver::Version) {
        let my_clone = self.clone();
        let mut mutable = my_clone.0.lock().unwrap();
        mutable.insert(key, value);
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
