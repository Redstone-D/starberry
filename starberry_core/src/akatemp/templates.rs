use std::collections::HashMap; 
use std::fs; 
use std::io::Read; 
use super::parse; 

fn render_template(template: &str, data: &HashMap<String, String>) -> String {
    let file = fs::File::open(template).unwrap(); 
    let mut buffer = Vec::new(); 
    file.read_to_end(&mut buffer).unwrap();
    let parsed_template = parse::parse_template(&buffer); 
    rendered
} 
