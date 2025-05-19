use std::collections::HashMap;
use once_cell::sync::Lazy;
use starberry_lib::{decode_url_owned, encode_url_owned};

#[derive(Debug, Clone)] 
pub struct UrlEncodedForm{ 
    pub data: HashMap<String, String>  
} 

impl UrlEncodedForm{ 
    /// Creates a new UrlEncodedForm with an empty HashMap. 
    pub fn new() -> Self { 
        Self { data: HashMap::new() } 
    } 

    pub fn parse(body: Vec<u8>) -> Self {
        let form_data = String::from_utf8_lossy(&body).to_string();
        let mut form_map = HashMap::new();
        for pair in form_data.split('&') {
            let parts: Vec<&str> = pair.split('=').collect();
            if parts.len() == 2 {
                form_map.insert(decode_url_owned(parts[0]), decode_url_owned(parts[1]));
            }
        }
        return UrlEncodedForm { data: form_map }; 
    } 

    pub fn to_string(&self) -> String {
        let mut form_data = String::new();
        for (key, value) in &self.data {
            if !form_data.is_empty() {
                form_data.push('&');
            }
            form_data.push_str(&format!("{}={}", encode_url_owned(key), encode_url_owned(value)));
        }
        form_data
    } 

    /// Inserts a key-value pair into the UrlEncodedForm. 
    pub fn insert(&mut self, key: String, value: String) { 
        self.data.insert(key, value); 
    } 

    /// Gets the value from the UrlEncodedForm. 
    pub fn get(&self, key: &str) -> Option<&String> { 
        self.data.get(key) 
    } 

    pub fn get_or_default(&self, key: &str) -> &String { 
        if let Some(value) = self.data.get(key) { 
            return value; 
        } 
        static EMPTY: Lazy<String> = Lazy::new(|| "".to_string()); 
        &EMPTY 
    } 

    /// Gets all values from the UrlEncodedForm. 
    pub fn get_all(&self) -> &HashMap<String, String> { 
        &self.data 
    } 
} 

impl From<HashMap<String, String>> for UrlEncodedForm { 
    fn from(data: HashMap<String, String>) -> Self { 
        Self { data } 
    } 
} 

/// Represents a multipart form data. 
#[derive(Debug, Clone)] 
pub struct MultiForm{ 
    data: HashMap<String, MultiFormField> 
} 

/// Represents a field in a multipart form.
#[derive(Debug, Clone)]
pub enum MultiFormField {
    Text(String),
    File(Vec<MultiFormFieldFile>)
} 

/// Represents a file in a multipart form. 
#[derive(Debug, Clone)]
pub struct MultiFormFieldFile {
    filename: Option<String>,
    content_type: Option<String>, 
    data: Vec<u8>,
} 

impl From<HashMap<String, MultiFormField>> for MultiForm { 
    fn from(data: HashMap<String, MultiFormField>) -> Self { 
        Self { data } 
    } 
} 

impl MultiForm{ 
    /// Creates a new MultiForm with an empty HashMap. 
    pub fn new() -> Self { 
        Self { data: HashMap::new() } 
    } 
    
    /// Parses a multipart form data body into a HashMap.
    ///
    /// # Arguments
    ///
    /// * `body` - The raw bytes of the multipart form data body
    /// * `boundary` - The boundary string specified in the Content-Type header
    ///
    /// # Returns
    ///
    /// A HashMap where keys are field names and values are parsed form fields
    ///
    /// # Examples
    ///
    /// ```
    /// use starberry_core::http::form::MultiForm;  
    /// let boundary = "boundary123";
    /// let body = concat!(
    ///     "--boundary123\r\n",
    ///     "Content-Disposition: form-data; name=\"field1\"\r\n\r\n",
    ///     "value1\r\n",
    ///     "--boundary123\r\n",
    ///     "Content-Disposition: form-data; name=\"file1\"; filename=\"example.txt\"\r\n",
    ///     "Content-Type: text/plain\r\n\r\n",
    ///     "file content here\r\n",
    ///     "--boundary123--\r\n"
    /// ).as_bytes().to_vec();
    ///
    /// let form = MultiForm::parse(body, boundary.to_string()); 
    /// assert_eq!(form.len(), 2);
    /// assert!(form.contains_key("field1"));
    /// assert!(form.contains_key("file1"));
    /// // Test the file content and filename
    /// assert_eq!(form.get("file1").unwrap().filename(), Some("example.txt".to_string()));
    /// ```
    pub fn parse(body: Vec<u8>, boundary: String) -> Self {
        /// Finds a subsequence within a larger sequence of bytes.
        fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
            haystack
                .windows(needle.len())
                .position(|window| window == needle)
        }

        /// Extracts the field name from the Content-Disposition header.
        fn extract_field_name(headers: &str) -> Option<String> {
            // Simple regex to extract name="value" from Content-Disposition
            let re = regex::Regex::new(r#"Content-Disposition:.*?name="([^"]+)""#).unwrap();
            re.captures(headers)
                .and_then(|cap| cap.get(1))
                .map(|m| m.as_str().to_string())
        }

        /// Extracts the filename from the Content-Disposition header if present.
        fn extract_filename(headers: &str) -> Option<String> {
            let re = regex::Regex::new(r#"Content-Disposition:.*?filename="([^"]+)""#).unwrap();
            re.captures(headers)
                .and_then(|cap| cap.get(1))
                .map(|m| m.as_str().to_string())
        }

        /// Extracts the content type from the Content-Type header if present.
        fn extract_content_type(headers: &str) -> Option<String> {
            let re = regex::Regex::new(r#"Content-Type:\s*(.+?)(?:\r\n|\r|\n|$)"#).unwrap();
            re.captures(headers)
                .and_then(|cap| cap.get(1))
                .map(|m| m.as_str().trim().to_string())
        }

        let mut form_map: HashMap<String, MultiFormField> = HashMap::new();

        // The boundary in the body is prefixed with "--"
        let boundary = format!("--{}", boundary);
        let boundary_bytes = boundary.as_bytes();
        let end_boundary = format!("{}--", boundary);
        let end_boundary_bytes = end_boundary.as_bytes();

        // Split the body by boundaries
        let mut parts: Vec<&[u8]> = Vec::new();
        let mut start_idx = 0;

        while let Some(idx) = find_subsequence(&body[start_idx..], boundary_bytes) {
            // Skip the first boundary or add the part if not the first
            if start_idx > 0 {
                parts.push(&body[start_idx..start_idx + idx - 2]); // -2 to remove trailing CRLF
            }

            // Move past this boundary
            start_idx += idx + boundary_bytes.len();

            // Check if this is the end boundary
            if start_idx < body.len()
                && body.len() - start_idx >= 2
                && body[start_idx..start_idx + 2] == [b'-', b'-']
            {
                break; // End boundary found
            }
        }

        // Process each part
        for part in parts {
            if part.len() < 4 {
                // Minimum size for valid part
                continue;
            }

            // Find headers and content separation (double CRLF)
            if let Some(header_end) = find_subsequence(part, b"\r\n\r\n") {
                let headers = &part[..header_end];
                let content = &part[header_end + 4..]; // +4 to skip the double CRLF

                // Parse headers as UTF-8 string
                if let Ok(headers_str) = std::str::from_utf8(headers) {
                    let name = extract_field_name(headers_str);
                    let filename = extract_filename(headers_str);
                    let content_type = extract_content_type(headers_str);

                    if let Some(field_name) = name {
                        if let Some(filename) = filename {
                            match form_map.get_mut(&field_name) {
                                Some(field) => {
                                    field.insert_file(MultiFormFieldFile::new(
                                        Some(filename),
                                        content_type,
                                        content.to_vec(),
                                    ));
                                }
                                None => {
                                    form_map.insert(
                                        field_name.clone(),
                                        MultiFormField::new_file(MultiFormFieldFile::new(
                                            Some(filename),
                                            content_type,
                                            content.to_vec(),
                                        )),
                                    );
                                }
                            }
                        } else {
                            // This is a text field - try to convert to UTF-8
                            if let Ok(text_value) = std::str::from_utf8(content) {
                                form_map.insert(
                                    field_name,
                                    MultiFormField::Text(text_value.to_string()),
                                );
                            } else {
                                // Fallback for non-UTF-8 field content
                                form_map.insert(
                                    field_name.clone(),
                                    MultiFormField::new_file(MultiFormFieldFile::new(
                                        None,
                                        content_type,
                                        content.to_vec(),
                                    )),
                                );
                            }
                        }
                    }
                }
            }
        }

        form_map.into() 
    } 

    /// Change a MultiForm into a string. 
    pub fn to_string(&self, boundary: &String) -> String { 
        let mut form_data = String::new();
        for (key, field) in &self.data {
            form_data.push_str(&format!("--{}\r\n", boundary));
            match field {
                MultiFormField::Text(value) => {
                    form_data.push_str(&format!(
                        "Content-Disposition: form-data; name=\"{}\"\r\n\r\n{}\r\n",
                        key, value
                    ));
                }
                MultiFormField::File(files) => {
                    for file in files {
                        form_data.push_str(&format!(
                            "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n",
                            key,
                            file.filename.as_ref().unwrap_or(&"".to_string())
                        ));
                        if let Some(content_type) = &file.content_type {
                            form_data.push_str(&format!("Content-Type: {}\r\n", content_type));
                        }
                        form_data.push_str("\r\n");
                        form_data.push_str(std::str::from_utf8(&file.data).unwrap_or(""));
                        form_data.push_str("\r\n");
                    }
                }
            }
        }
        form_data.push_str(&format!("--{}--\r\n", boundary));
        form_data 
    }

    /// Inserts a field into the MultiForm. 
    pub fn insert(&mut self, key: String, value: MultiFormField) { 
        self.data.insert(key, value); 
    } 

    /// Gets the field from the MultiForm. 
    pub fn get(&self, key: &str) -> Option<&MultiFormField> { 
        self.data.get(key) 
    } 

    /// Gets all fields from the MultiForm. 
    pub fn get_all(&self) -> &HashMap<String, MultiFormField> { 
        &self.data 
    } 

    /// Gets the files from the MultiForm. 
    pub fn get_text(&self, key: &str) -> Option<&String> { 
        if let Some(field) = self.data.get(key) { 
            if let MultiFormField::Text(value) = field { 
                return Some(value); 
            } 
        } 
        None 
    } 

    pub fn get_text_or_default(&self, key: &str) -> String { 
        if let Some(field) = self.data.get(key) { 
            if let MultiFormField::Text(value) = field { 
                return value.clone(); 
            } 
        } 
        "".to_string() 
    } 

    /// Gets the files from the MultiForm. 
    pub fn get_files(&self, key: &str) -> Option<&Vec<MultiFormFieldFile>> { 
        if let Some(field) = self.data.get(key) { 
            if let MultiFormField::File(files) = field { 
                return Some(files); 
            } 
        } 
        None 
    } 

    /// Gets the files from the MultiForm. 
    /// This function returns an empty vector if the field is not found or if it is not a file. 
    pub fn get_files_or_default(&self, key: &str) -> &Vec<MultiFormFieldFile> { 
        if let Some(field) = self.data.get(key) { 
            if let MultiFormField::File(files) = field { 
                return files; 
            } 
        } 
        static EMPTY: Lazy<Vec<MultiFormFieldFile>> = Lazy::new(|| Vec::new()); 
        &EMPTY 
    } 

    /// Get the first file from the MultiForm. 
    pub fn get_first_file(&self, key: &str) -> Option<&MultiFormFieldFile> { 
        if let Some(field) = self.data.get(key) { 
            if let MultiFormField::File(files) = field { 
                return files.first(); 
            } 
        } 
        None 
    } 

    /// Get the first file from the MultiForm. 
    /// This function returns the first file as a MultiFormFieldFile. 
    pub fn get_first_file_or_default(&self, key: &str) -> &MultiFormFieldFile { 
        if let Some(field) = self.get_first_file(key) { 
            return field; 
        } 
        static EMPTY: Lazy<MultiFormFieldFile> = Lazy::new(|| MultiFormFieldFile::default()); 
        &EMPTY 
    } 

    /// Get the first file content from the MultiForm. 
    /// This function returns the first file content as a byte slice. 
    pub fn get_first_file_content(&self, key: &str) -> Option<&[u8]> { 
        if let Some(field) = self.data.get(key) { 
            if let MultiFormField::File(files) = field { 
                return files.first().map(|file| file.data.as_slice()); 
            } 
        } 
        None 
    } 

    /// Get the first file content from the MultiForm. 
    /// This function returns the first file content as a byte vector. 
    /// This function returns an empty vector if the field is not found or if it is not a file. 
    pub fn get_first_file_content_or_default(&self, key: &str) -> &[u8] { 
        if let Some(content) = self.get_first_file_content(key) { 
            return content; 
        } 
        static EMPTY: Lazy<Vec<u8>> = Lazy::new(|| Vec::new()); 
        &EMPTY 
    }
}

impl MultiFormField { 
    pub fn new_text(value: String) -> Self {
        Self::Text(value) 
    } 
    
    pub fn new_file(files: MultiFormFieldFile) -> Self {
        Self::File(vec![files])  
    } 

    /// Creates a new MultiFormField with a file. 
    /// This function takes a filename, content type, and data as parameters. 
    /// It returns a MultiFormField::File variant. 
    /// When the Field is Text type, it will change it into a File type. 
    pub fn insert_file(&mut self, file: MultiFormFieldFile) {
        if let Self::File(files) = self {
            files.push(file); 
        } else {
            *self = Self::File(vec![file]); 
        }
    }    

    /// Gets the files value from the MultiFormField. 
    pub fn get_files(&self) -> Option<&Vec<MultiFormFieldFile>> {
        if let Self::File(files) = self {
            Some(files) 
        } else {
            None 
        } 
    } 
}

impl Default for MultiFormField { 
    /// Creates a new MultiFormField with an empty string. 
    fn default() -> Self { 
        Self::Text("".to_string()) 
    } 
} 

impl MultiFormFieldFile{ 
    pub fn new(filename: Option<String>, content_type: Option<String>, data: Vec<u8>) -> Self { 
        Self { filename, content_type, data } 
    } 

    pub fn filename(&self) -> Option<String> { 
        self.filename.clone() 
    } 

    pub fn content_type(&self) -> Option<String> { 
        self.content_type.clone() 
    } 

    pub fn data(&self) -> &[u8] { 
        &self.data 
    } 
} 

impl Default for MultiFormFieldFile { 
    fn default() -> Self { 
        Self { filename: None, content_type: None, data: Vec::new() } 
    } 
} 

