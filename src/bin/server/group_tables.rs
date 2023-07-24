use std::{sync::{Mutex, Arc}, collections::HashMap};

use crate::group::Group;


pub struct GroupTables(Mutex<HashMap<Arc<String>, Arc<Group>>>);

impl GroupTables {
    
    pub fn new() -> GroupTables{
        GroupTables(Mutex::new(HashMap::new()))
    }

    pub fn get(&self, name: &String) -> Option<Arc<Group>>{
        self.0.lock().unwrap().get(name).cloned()
    }

    pub fn get_or_create(&self, name: Arc<String>)  -> Arc<Group> {
        self.0.lock().unwrap().entry(name.clone()).or_insert_with(|| Arc::new(Group::new(name))).clone()
    }
}