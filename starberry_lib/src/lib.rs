use percent_encoding::{percent_decode, percent_encode, NON_ALPHANUMERIC}; 
use rand::Rng; 


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

#[cfg(feature = "ende")]
pub mod ende; 

#[cfg(feature = "url_encoding")]
pub mod url_encoding; 

#[cfg(feature = "compression")] 
pub mod compression; 

