use std::collections::HashMap;

use starberry_core::http::cookie::Cookie;
use starberry_macro::middleware;
use starberry_core::app::middleware::AsyncMiddleware; 
use starberry_core::http::context::HttpReqCtx;
use akari::Value;  

use crate::session::session_counter;   

pub struct CSessionRW(HashMap::<String, Value>, bool); 

impl CSessionRW { 
    pub fn new() -> Self {
        CSessionRW(HashMap::new(), false)
    } 

    pub fn from_hash(map: HashMap<String, Value>) -> Self {
        CSessionRW(map, false)
    } 

    pub fn insert(&mut self, key: String, value: Value) {
        self.0.insert(key, value); 
        self.1 = true; // Mark as modified 
    } 

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key) 
    } 

    pub fn remove(&mut self, key: &str) -> Option<Value> {
        let removed = self.0.remove(key);
        if removed.is_some() {
            self.1 = true; // Mark as modified 
        }
        removed
    } 

    pub fn is_modified(&self) -> bool {
        self.1
    } 

    pub fn into_tuple(self) -> (Value, bool) {
        (Value::Dict(self.0), self.1)
    } 
}

impl Default for CSessionRW { 
    fn default() -> Self {
        CSessionRW(HashMap::new(), false)
    }
} 

#[middleware(HttpReqCtx)] 
pub async fn CookieSession(){ 

    let session_id: u64 = req.get_cookie_or_default("session_id")
        .get_value()
        .parse()
        .unwrap_or_else(|_| {
            session_counter::generate_session_id() 
        });  

    println!("Session ID: {}", req.get_cookie_or_default("session_cont")); 

    let session = CSessionRW::from_hash(
        if let Value::Dict(map) = Value::from_json(
            req.get_cookie_or_default("session_cont")
                .get_value() 
            ).unwrap_or(Value::None) { map } else { 
                HashMap::new() 
            } 
        ); 
    
    req.params.set(session); 
    let mut req = next(req).await; // Continue middleware chain 

    let (session, is_modified) = req
            .params
            .take::<CSessionRW>() 
            .unwrap_or_default() 
            .into_tuple(); 

    println!("Cookie Session: {}", session); 

    if is_modified { 
        req.response = req.response.add_cookie(
            "session_id", 
            Cookie::new(session_id.to_string()) 
                .path("/") 
        ).add_cookie(
            "session_cont", 
            Cookie::new(session.into_json()) 
                .path("/") 
        ); // Set cookie with session ID 
    } 

    req 
} 
 
