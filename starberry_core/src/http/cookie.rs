use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)] 
pub struct CookieMap(pub HashMap<String, Cookie>); 

impl CookieMap { 
    pub fn new() -> Self { 
        Self(HashMap::new()) 
    } 

    /// Parses Cookie header into a Cookie Map 
    pub fn parse<T: Into<String>>(cookies: T) -> Self { 
        let mut cookie_map = CookieMap::new(); 
        let cookies = cookies.into(); 
        for cookie in cookies.split(';') {
            let parts: Vec<&str> = cookie.split('=').collect();
            if parts.len() == 2 {
                cookie_map.set(
                    parts[0].trim(), 
                    Cookie::new(parts[1].trim()));
            }
        } 
        cookie_map
    } 

    /// Parses multiple Set-Cookie headers into a CookieMap.
    ///
    /// # Arguments
    ///
    /// * `set_cookies` - A collection of Set-Cookie header values
    ///
    /// # Returns
    ///
    /// A CookieMap containing the parsed cookies.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::cookie::CookieMap;
    ///
    /// let set_cookies = vec![
    ///     "sessionId=abc123; Path=/; Secure",
    ///     "theme=dark; Path=/settings"
    /// ];
    ///
    /// let cookies = CookieMap::parse_set_cookies(&set_cookies);
    /// assert_eq!(cookies.get("sessionId").unwrap().value, "abc123");
    /// assert_eq!(cookies.get("theme").unwrap().value, "dark");
    /// assert_eq!(cookies.get("theme").unwrap().get_path(), Some("/settings".to_string()));
    /// ```
    pub fn parse_set_cookies<'a, I, T>(set_cookies: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let mut cookie_map = CookieMap::new();
        
        for cookie_str in set_cookies {
            let (name, cookie) = Cookie::parse_set_cookie(cookie_str.as_ref());
            if !name.is_empty() {
                cookie_map.set(name, cookie);
            }
        }
        
        cookie_map
    } 

    pub fn get<T: AsRef<str>>(&self, key: T) -> Option<&Cookie> { 
        self.0.get(key.as_ref()) 
    } 

    pub fn set<T: Into<String>>(&mut self, key: T, value: Cookie) { 
        self.0.insert(key.into(), value); 
    } 

    pub fn remove<T: AsRef<str>>(&mut self, key: T) -> Option<Cookie> { 
        self.0.remove(key.as_ref()) 
    } 

    pub fn clear(&mut self) { 
        self.0.clear(); 
    } 

    pub fn response(&self) -> String { 
        let mut result = String::new(); 
        for (key, value) in &self.0 { 
            result.push_str(&format!("Set-Cookie: {}={};", key, value.response())); 
        } 
        result  
    } 

    pub fn request(&self) -> String { 
        let mut result = String::new(); 
        result.push_str("Cookie: "); 
        for (key, value) in &self.0 { 
            result.push_str(&format!("{}={}; ", key, value.request())); 
        } 
        result  
    } 
} 

impl Default for CookieMap { 
    fn default() -> Self { 
        Self::new() 
    } 
} 

impl From<HashMap<String, String>> for CookieMap { 
    fn from(map: HashMap<String, String>) -> Self { 
        let mut cookie_map = CookieMap::new(); 
        for (key, value) in map { 
            cookie_map.set(key, Cookie::new(value)); 
        } 
        cookie_map 
    }  
} 

impl IntoIterator for CookieMap {
    type Item = (String, Cookie);
    type IntoIter = std::collections::hash_map::IntoIter<String, Cookie>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
} 

#[derive(Debug, Clone, PartialEq)] 
pub struct Cookie{ 
    pub value: String, 
    pub path: Option<String>, 
    pub domain: Option<String>, 
    pub expires: Option<String>, 
    pub max_age: Option<String>, 
    pub secure: Option<bool>, 
    pub http_only: Option<bool>, 
} 

impl Cookie{ 
    /// Creates a new CookieResponse with the given name and value. 
    /// This function initializes the cookie with default values for path, domain, expires, max_age, secure, and http_only. 
    /// It returns a CookieResponse instance. 
    /// # Examples 
    /// ```rust 
    /// use starberry_core::http::http_value::CookieResponse; 
    /// let cookie = CookieResponse::new("session_id", 123456).domain("example.com".to_string()).path("/".to_string()).expires("Wed, 21 Oct 2025 07:28:00 GMT".to_string()).secure(true).http_only(true); 
    /// ``` 
    pub fn new<T: ToString>(value: T) -> Self { 
        Self { 
            value: value.to_string(), 
            path: None, 
            domain: None, 
            expires: None, 
            max_age: None, 
            secure: None, 
            http_only: None, 
        } 
    } 

    /// Parses a Set-Cookie header value into a cookie name and Cookie object.
    ///
    /// # Arguments
    ///
    /// * `set_cookie_str` - A string slice containing the Set-Cookie header value
    ///
    /// # Returns
    ///
    /// A tuple containing the cookie name and a Cookie object with parsed attributes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::cookie::Cookie;
    ///
    /// let set_cookie = "sessionId=abc123; Path=/; Domain=example.com; Secure; HttpOnly";
    /// let (name, cookie) = Cookie::parse_set_cookie(set_cookie);
    ///
    /// assert_eq!(name, "sessionId");
    /// assert_eq!(cookie.value, "abc123");
    /// assert_eq!(cookie.get_path(), Some("/".to_string()));
    /// assert_eq!(cookie.get_domain(), Some("example.com".to_string()));
    /// assert_eq!(cookie.get_secure(), Some(true));
    /// assert_eq!(cookie.get_http_only(), Some(true));
    /// ```
    pub fn parse_set_cookie(set_cookie_str: &str) -> (String, Self) {
        // Split into name=value and attributes
        let mut parts = set_cookie_str.splitn(2, '=');
        
        // Get the name
        let name = parts.next()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
            
        // Get value and attributes
        let value_and_attrs = parts.next()
            .unwrap_or("")
            .trim();
            
        // Split the rest into value and attributes
        let attrs_parts: Vec<&str> = value_and_attrs.split(';').collect();
        
        // Create cookie with the value (first part)
        let value = if !attrs_parts.is_empty() { 
            attrs_parts[0].trim().to_string() 
        } else { 
            String::new() 
        };
        
        let mut cookie = Cookie::new(value);
        
        // Parse attributes (skip the first part which is the value)
        for attr in attrs_parts.iter().skip(1) {
            let attr = attr.trim();
            
            if attr.eq_ignore_ascii_case("Secure") {
                cookie.set_secure(true);
                continue;
            }
            if attr.eq_ignore_ascii_case("HttpOnly") {
                cookie.set_http_only(true);
                continue;
            }
            
            // Parse key=value attributes
            let attr_parts: Vec<&str> = attr.splitn(2, '=').collect();
            if attr_parts.len() == 2 {
                let attr_name = attr_parts[0].trim();
                let attr_value = attr_parts[1].trim();
                
                match attr_name.to_lowercase().as_str() {
                    "path" => cookie.set_path(attr_value),
                    "domain" => cookie.set_domain(attr_value),
                    "expires" => cookie.set_expires(attr_value),
                    "max-age" => cookie.set_max_age(attr_value),
                    _ => {} // Ignore unknown attributes
                }
            }
        }
        
        (name, cookie)
    } 

    pub fn get_value(&self) -> &str { 
        &self.value  
    } 

    pub fn set_value<T: ToString>(&mut self, value: T) { 
        self.value = value.to_string(); 
    } 

    pub fn path<T: ToString>(self, path: T) -> Self { 
        Self { path: Some(path.to_string()), ..self } 
    } 

    pub fn get_path(&self) -> Option<String> { 
        self.path.clone() 
    } 

    pub fn set_path<T: ToString> (&mut self, path: T) { 
        self.path = Some(path.to_string()); 
    } 

    pub fn clear_path(&mut self) { 
        self.path = None; 
    } 

    pub fn domain<T: ToString>(self, domain: T) -> Self { 
        Self { domain: Some(domain.to_string()), ..self } 
    } 

    pub fn get_domain(&self) -> Option<String> { 
        self.domain.clone() 
    } 

    pub fn set_domain<T: ToString> (&mut self, domain: T) { 
        self.domain = Some(domain.to_string()); 
    } 

    pub fn clear_domain(&mut self) { 
        self.domain = None; 
    } 

    pub fn expires<T: ToString> (self, expires: T) -> Self { 
        Self { expires: Some(expires.to_string()), ..self } 
    } 

    pub fn get_expires(&self) -> Option<String> { 
        self.expires.clone() 
    } 

    pub fn set_expires<T: ToString> (&mut self, expires: T) { 
        self.expires = Some(expires.to_string()); 
    } 

    pub fn clear_expires(&mut self) { 
        self.expires = None; 
    } 

    pub fn max_age<T: ToString> (self, max_age: T) -> Self { 
        Self { max_age: Some(max_age.to_string()), ..self } 
    } 

    pub fn get_max_age(&self) -> Option<String> { 
        self.max_age.clone() 
    } 

    pub fn set_max_age<T: ToString> (&mut self, max_age: T) { 
        self.max_age = Some(max_age.to_string()); 
    } 

    pub fn clear_max_age(&mut self) { 
        self.max_age = None; 
    } 

    pub fn secure(self, secure: bool) -> Self { 
        Self { secure: Some(secure), ..self } 
    } 

    pub fn get_secure(&self) -> Option<bool> { 
        self.secure.clone() 
    } 

    pub fn set_secure(&mut self, secure: bool) { 
        self.secure = Some(secure); 
    } 

    pub fn clear_secure(&mut self) { 
        self.secure = None; 
    } 

    pub fn http_only(self, http_only: bool) -> Self { 
        Self { http_only: Some(http_only), ..self } 
    } 

    pub fn get_http_only(&self) -> Option<bool> { 
        self.http_only.clone() 
    } 

    pub fn set_http_only(&mut self, http_only: bool) { 
        self.http_only = Some(http_only); 
    } 

    pub fn clear_http_only(&mut self) { 
        self.http_only = None; 
    } 

    /// Returns a string formatted for a Set-Cookie header including all attributes.
    ///
    /// # Returns
    ///
    /// A string suitable for use in a Set-Cookie header.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use starberry_core::http::cookie::Cookie;
    ///
    /// let cookie = Cookie::new("abc123").path("/").secure(true);
    /// assert_eq!(cookie.to_string(), "abc123; Path=/; Secure");
    /// ```
    pub fn to_string(&self) -> String { 
        let mut result = format!("{}", self.value.to_string()); 
        if let Some(ref path) = self.path { 
            result.push_str(&format!("; Path={}", path)); 
        } 
        if let Some(ref domain) = self.domain { 
            result.push_str(&format!("; Domain={}", domain)); 
        } 
        if let Some(ref expires) = self.expires { 
            result.push_str(&format!("; Expires={}", expires)); 
        } 
        if let Some(ref max_age) = self.max_age { 
            result.push_str(&format!("; Max-Age={}", max_age)); 
        } 
        if let Some(ref secure) = self.secure { 
            if *secure { 
                result.push_str("; Secure"); 
            } 
        } 
        if let Some(ref http_only) = self.http_only { 
            if *http_only { 
                result.push_str("; HttpOnly"); 
            } 
        } 
        result 
    } 

    pub fn response(&self) -> String { 
        format!("{}", self.to_string()) 
    } 

    pub fn request(&self) -> String { 
        format!("{}", self.value) 
    } 
} 

impl std::fmt::Display for Cookie { 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        write!(f, "{}", self.to_string()) 
    } 
}  

