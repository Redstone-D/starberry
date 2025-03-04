use std::collections::HashMap; 
use std::fs; 

fn render_template(template: &str, data: &HashMap<String, String>) -> String {
    let content = fs::read_to_string(template).unwrap(); 
    let mut rendered = content.clone(); 
    
    rendered
} 