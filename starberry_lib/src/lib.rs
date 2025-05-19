use percent_encoding::{percent_decode, percent_encode, NON_ALPHANUMERIC}; 
use rand::Rng; 

/// Encodes a string for URL safety and returns an owned `String`
/// 
/// # Example
/// ```
/// let encoded = encode_url_owned("Hello World!");
/// assert_eq!(encoded, "Hello%20World%21");
/// ```
pub fn encode_url_owned(input: &str) -> String {
    percent_encode(input.as_bytes(), NON_ALPHANUMERIC).to_string()
}

/// Encodes a string in place for URL safety
/// 
/// # Example
/// ```
/// let mut s = String::from("Hello World!");
/// encode_url(&mut s);
/// assert_eq!(s, "Hello%20World%21");
/// ```
pub fn encode_url(input: &mut String) {
    let encoded = encode_url_owned(input);
    *input = encoded;
} 

/// Decodes a URL-encoded string and returns an owned `String`.
///
/// # Arguments
///
/// * `input` - A URL-encoded string as a `&str`.
///
/// # Returns
///
/// A new `String` containing the decoded value.
pub fn decode_url_owned(input: &str) -> String {
    percent_decode(input.as_bytes())
        .decode_utf8_lossy()
        .into_owned() 
}

/// Decodes a URL-encoded string in place by updating the provided `String`.
///
/// # Arguments
///
/// * `input` - A mutable reference to a `String` holding a URL-encoded value.
///   After the call, it will contain the decoded version.
pub fn decode_url(input: &mut String) {
    let decoded = decode_url_owned(input);
    *input = decoded;
} 

/// Generates a random string of the specified length using printable ASCII characters. 
pub fn random_string(length: usize) -> String {
    let mut rng = rand::rng();
    let bytes: Vec<u8> = (0..length).map(|_| rng.random_range(33..127)).collect();
    String::from_utf8(bytes).unwrap()
} 

pub fn random_alphanumeric_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

