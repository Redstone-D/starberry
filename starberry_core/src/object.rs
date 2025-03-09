use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Object {
    Numerical(f64),
    Boolean(bool), 
    Str(String),
    List(Vec<Object>),
    Dictionary(HashMap<String, Object>), // Key type changed to String
}

impl Object {
    pub fn new<T: Into<Object>>(value: T) -> Self {
        value.into()
    }
    
    pub fn type_of(&self) -> String {
        match self {
            Object::Numerical(_) => "num".to_string(),
            Object::Boolean(_) => "bool".to_string(),
            Object::Str(_) => "str".to_string(),
            Object::List(_) => "vec".to_string(),
            Object::Dictionary(_) => "dict".to_string(),
        }
    }
    
    pub fn into_json(&self) -> String {
        match self {
            Object::Numerical(n) => n.to_string(),
            Object::Boolean(b) => b.to_string(),
            Object::Str(s) => format!("\"{}\"", s),
            Object::List(l) => {
                let mut result = String::new();
                for item in l {
                    result.push_str(&format!("{}, ", item.into_json()));
                }
                if result.len() >= 2 { result.truncate(result.len() - 2); }
                format!("[{}]", result)
            }
            Object::Dictionary(d) => {
                let mut result = String::new();
                for (key, value) in d {
                    result.push_str(&format!("\"{}\": {}, ", key, value.into_json()));
                }
                if result.len() >= 2 { result.truncate(result.len() - 2); }
                format!("{{{}}}", result)
            }
        }
    }
    
    pub fn from_json(json: &str) -> Result<Self, String> {
        let mut parser = Parser::new(json);
        let value = parser.parse_value()?;
        parser.skip_whitespace();
        if parser.pos != json.len() {
            return Err("Extra characters after JSON value".to_string());
        }
        Ok(value)
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Numerical(n) => write!(f, "{}", n),
            Object::Boolean(b) => write!(f, "{}", b),
            Object::Str(s) => write!(f, "\"{}\"", s),
            Object::List(l) => {
                let mut result = String::new();
                for item in l { result.push_str(&format!("{}, ", item)); }
                if result.len() >= 2 { result.truncate(result.len() - 2); }
                write!(f, "[{}]", result)
            }
            Object::Dictionary(d) => {
                let mut result = String::new();
                for (key, value) in d {
                    result.push_str(&format!("{} {} = {}, ", value.type_of(), key, value));
                }
                if result.len() >= 2 { result.truncate(result.len() - 2); }
                write!(f, "{{{}}}", result)
            }
        }
    }
}

// From implementations
impl From<i8> for Object { fn from(n: i8) -> Self { Object::Numerical(n as f64) } }
impl From<i16> for Object { fn from(n: i16) -> Self { Object::Numerical(n as f64) } }
impl From<i32> for Object { fn from(n: i32) -> Self { Object::Numerical(n as f64) } }
impl From<i64> for Object { fn from(n: i64) -> Self { Object::Numerical(n as f64) } }
impl From<i128> for Object { fn from(n: i128) -> Self { Object::Numerical(n as f64) } }
impl From<isize> for Object { fn from(n: isize) -> Self { Object::Numerical(n as f64) } }
impl From<u8> for Object { fn from(n: u8) -> Self { Object::Numerical(n as f64) } }
impl From<u16> for Object { fn from(n: u16) -> Self { Object::Numerical(n as f64) } }
impl From<u32> for Object { fn from(n: u32) -> Self { Object::Numerical(n as f64) } }
impl From<u64> for Object { fn from(n: u64) -> Self { Object::Numerical(n as f64) } }
impl From<u128> for Object { fn from(n: u128) -> Self { Object::Numerical(n as f64) } }
impl From<usize> for Object { fn from(n: usize) -> Self { Object::Numerical(n as f64) } }
impl From<f32> for Object { fn from(n: f32) -> Self { Object::Numerical(n as f64) } }
impl From<f64> for Object { fn from(n: f64) -> Self { Object::Numerical(n) } }
impl From<char> for Object { fn from(c: char) -> Self { Object::Str(c.to_string()) } }
impl From<bool> for Object { fn from(b: bool) -> Self { Object::Boolean(b) } }
impl From<&str> for Object { fn from(s: &str) -> Self { Object::Str(s.to_string()) } }
impl From<String> for Object { fn from(s: String) -> Self { Object::Str(s) } }
impl From<Vec<Object>> for Object { fn from(vec: Vec<Object>) -> Self { Object::List(vec) } }
impl From<HashMap<String, Object>> for Object { fn from(map: HashMap<String, Object>) -> Self { Object::Dictionary(map) } }

// Recursive-descent JSON parser
struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser { input, pos: 0 }
    }
    
    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }
    
    fn next(&mut self) -> Option<char> {
        if let Some(ch) = self.peek() {
            self.pos += ch.len_utf8();
            Some(ch)
        } else {
            None
        }
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() { self.next(); } else { break; }
        }
    }
    
    fn parse_value(&mut self) -> Result<Object, String> {
        self.skip_whitespace();
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string().map(Object::Str),
            Some(ch) if ch == 't' || ch == 'f' => self.parse_boolean().map(Object::Boolean),
            Some(ch) if ch.is_digit(10) || ch == '-' => self.parse_number().map(Object::Numerical),
            _ => Err(format!("Unexpected character at position {}: {:?}", self.pos, self.peek())),
        }
    }
    
    fn parse_object(&mut self) -> Result<Object, String> {
        let mut map = HashMap::new();
        if self.next() != Some('{') {
            return Err(format!("Expected '{{' at position {}", self.pos));
        }
        self.skip_whitespace();
        if let Some('}') = self.peek() {
            self.next();
            return Ok(Object::Dictionary(map));
        }
        loop {
            self.skip_whitespace();
            if self.peek() != Some('"') {
                return Err(format!("Expected '\"' at position {} for object key", self.pos));
            }
            let key = self.parse_string()?;
            self.skip_whitespace();
            if self.next() != Some(':') {
                return Err(format!("Expected ':' after key at position {}", self.pos));
            }
            self.skip_whitespace();
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => { self.next(); },
                Some('}') => { self.next(); break; },
                _ => return Err(format!("Expected ',' or '}}' at position {}", self.pos)),
            }
        }
        Ok(Object::Dictionary(map))
    }
    
    fn parse_array(&mut self) -> Result<Object, String> {
        let mut vec = Vec::new();
        if self.next() != Some('[') {
            return Err(format!("Expected '[' at position {}", self.pos));
        }
        self.skip_whitespace();
        if let Some(']') = self.peek() {
            self.next();
            return Ok(Object::List(vec));
        }
        loop {
            self.skip_whitespace();
            let value = self.parse_value()?;
            vec.push(value);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => { self.next(); },
                Some(']') => { self.next(); break; },
                _ => return Err(format!("Expected ',' or ']' at position {}", self.pos)),
            }
        }
        Ok(Object::List(vec))
    }
    
    fn parse_string(&mut self) -> Result<String, String> {
        let mut result = String::new();
        if self.next() != Some('"') {
            return Err("Expected '\"' at beginning of string".to_string());
        }
        while let Some(ch) = self.next() {
            if ch == '"' { return Ok(result); }
            if ch == '\\' {
                if let Some(esc) = self.next() {
                    match esc {
                        '"'  => result.push('"'),
                        '\\' => result.push('\\'),
                        '/'  => result.push('/'),
                        'b'  => result.push('\x08'),
                        'f'  => result.push('\x0C'),
                        'n'  => result.push('\n'),
                        'r'  => result.push('\r'),
                        't'  => result.push('\t'),
                        _    => return Err(format!("Invalid escape sequence: \\{}", esc)),
                    }
                } else {
                    return Err("Incomplete escape sequence".to_string());
                }
            } else {
                result.push(ch);
            }
        }
        Err("Unterminated string literal".to_string())
    }
    
    fn parse_number(&mut self) -> Result<f64, String> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_digit(10) || ch == '.' || ch == '-' || ch == 'e' || ch == 'E' || ch == '+' {
                self.next();
            } else {
                break;
            }
        }
        let number_str = &self.input[start..self.pos];
        number_str.parse::<f64>().map_err(|_| format!("Invalid number: {}", number_str))
    }
    
    fn parse_boolean(&mut self) -> Result<bool, String> {
        if self.input[self.pos..].starts_with("true") {
            self.pos += 4;
            Ok(true)
        } else if self.input[self.pos..].starts_with("false") {
            self.pos += 5;
            Ok(false)
        } else {
            Err(format!("Invalid boolean value at position {}", self.pos))
        }
    }
} 

/// A macro to create an Object from a literal or expression. 
/// It can handle dictionaries, lists, booleans, strings, and numeric values. 
/// # Example 
/// ```rust 
/// use starberry_core::object::Object; 
/// use starberry_core::object; 
/// let num_obj = object!(3); 
/// assert_eq!(num_obj, Object::Numerical(3.0)); 
/// ```
/// ```rust 
/// use starberry_core::object::Object;  
/// use std::collections::HashMap; 
/// use starberry_core::object; 
/// let list_obj = object!(["aaa", "bbb"]); 
/// assert_eq!(list_obj, Object::List(vec![Object::Str("aaa".to_string()), Object::Str("bbb".to_string())]));  
/// ``` 
/// ```rust 
/// use starberry_core::object::Object; 
/// use std::collections::HashMap; 
/// use starberry_core::object; 
/// let obj_obj = object!({c: "p", b: ["aaa", "bbb"], u: 32});  
/// assert_eq!(obj_obj, Object::Dictionary(HashMap::from([
///     ("c".to_string(), Object::Str("p".to_string())),
///     ("b".to_string(), Object::List(vec![Object::Str("aaa".to_string()), Object::Str("bbb".to_string())])), 
///     ("u".to_string(), Object::Numerical(32.0)),
/// ]))); 
/// ```
 
#[macro_export]
macro_rules! object {
    // Dictionary: keys become Strings now.
    ({ $( $key:ident : $value:tt ),* $(,)? }) => {{
        let mut map = ::std::collections::HashMap::new();
        $(
            map.insert(stringify!($key).to_string(), object!($value));
        )*
        Object::Dictionary(map)
    }};
    // List
    ([ $( $elem:tt ),* $(,)? ]) => {{
        let mut vec = Vec::new();
        $(
            vec.push(object!($elem));
        )*
        Object::List(vec)
    }};
    // Booleans
    (true) => {
        Object::new(true)
    };
    (false) => {
        Object::new(false)
    };
    // String literals
    ($e:literal) => {
        Object::new($e)
    };
    // Fallback for expressions (like numbers)
    ($e:expr) => {
        Object::new($e)
    };
}  

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_from_json_object() {
        let json = r#"{"a": 1, "b": true, "c": "hello"}"#;
        let obj = Object::from_json(json).expect("Failed to parse JSON");
        let mut expected_map = HashMap::new();
        expected_map.insert("a".to_string(), Object::Numerical(1.0));
        expected_map.insert("b".to_string(), Object::Boolean(true));
        expected_map.insert("c".to_string(), Object::Str("hello".to_string()));
        assert_eq!(obj, Object::Dictionary(expected_map));
    }
    
    #[test]
    fn test_from_json_array() {
        let json = r#"[1, 2, 3]"#;
        let obj = Object::from_json(json).expect("Failed to parse JSON");
        assert_eq!(obj, Object::List(vec![
            Object::Numerical(1.0),
            Object::Numerical(2.0),
            Object::Numerical(3.0)
        ]));
    }
    
    #[test]
    fn test_from_json_nested() {
        let json = r#"{"a": [true, false], "b": {"nested": "value"}}"#;
        let obj = Object::from_json(json).expect("Failed to parse JSON");
        // Further assertions can be added here.
    }
}

 