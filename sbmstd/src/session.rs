use std::collections::HashMap;
use std::sync::RwLock;
use lazy_static::lazy_static; 

#[derive(Debug)]
struct SessionCont {
    expiry_time: u64,
    data: HashMap<String, String>,
}

lazy_static! {
    static ref SESSION_STORE: RwLock<HashMap<String, &'static SessionCont>> = RwLock::new(HashMap::new());
} 

#[middleware] 
