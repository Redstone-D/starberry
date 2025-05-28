use super::http_value::{HttpContentType, HttpMethod};

#[derive(Debug, Clone)] 
pub struct MaxBodySize(usize); 

#[derive(Debug, Clone)] 
pub struct AllowedMethods(Vec<HttpMethod>); 

#[derive(Debug, Clone)]  
pub struct AllowedContentTypes(Vec<HttpContentType>); 

impl MaxBodySize {
    pub fn new(size: usize) -> Self {
        Self(size)
    }

    pub fn get(&self) -> usize {
        self.0
    }

    pub fn set(&mut self, size: usize) {
        self.0 = size;
    } 

    pub fn check(&self, size: usize) -> bool {
        size <= self.0
    } 
}

impl AllowedMethods {
    pub fn new(methods: Vec<HttpMethod>) -> Self {
        Self(methods)
    }

    pub fn get(&self) -> &[HttpMethod] {
        &self.0
    }

    pub fn set(&mut self, methods: Vec<HttpMethod>) {
        self.0 = methods;
    }

    pub fn add(&mut self, method: HttpMethod) {
        if !self.0.contains(&method) {
            self.0.push(method);
        }
    }

    pub fn remove(&mut self, method: HttpMethod) {
        self.0.retain(|m| *m != method);
    }

    pub fn reset(&mut self) {
        self.0.clear();
    } 

    pub fn check(&self, method: &HttpMethod) -> bool {
        self.0.contains(method)
    } 
}

impl AllowedContentTypes {
    pub fn new(content_types: Vec<HttpContentType>) -> Self {
        Self(content_types)
    }

    pub fn get(&self) -> &[HttpContentType] {
        &self.0
    }

    pub fn set(&mut self, content_types: Vec<HttpContentType>) {
        self.0 = content_types;
    }

    pub fn add(&mut self, content_type: HttpContentType) {
        if !self.0.contains(&content_type) {
            self.0.push(content_type);
        }
    }

    pub fn remove(&mut self, content_type: HttpContentType) {
        self.0.retain(|ct| *ct != content_type);
    }

    pub fn reset(&mut self) {
        self.0.clear();
    } 

    pub fn check(&self, content_type: &HttpContentType) -> bool {
        self.0.contains(content_type)
    } 
} 
