use std::collections::HashMap;

pub fn parse_template(buffer: &[u8], data: & mut HashMap<String, String>) -> Vec<u8> { 
    let command_positions = get_command_position(buffer); 
    let mut output = Vec::new(); 
    let mut last_start = 0; 

    for (start, end) in command_positions.iter() { 

        // Add the content before the command
        output.extend_from_slice(&buffer[last_start..*start]); 

        // Extract the command 
        output.extend_from_slice(execute_command(&buffer[*start + 2..*end - 1], data)); 

        // Update the start position for the next iteration 
        last_start = *end; 

    } 
    output 
} 

/// Parses a template buffer and returns a vector of tuples representing the start and end indices of each match of the command 
/// pattern "-[...]-". 
pub fn get_command_position(buffer: &[u8]) -> Vec<(usize, usize)> { 
    let mut matches = Vec::new(); 
    let mut i = 0; 

    while i < buffer.len() { 

        let start_offset = buffer[i..]
            .windows(2)
            .position(|window| window == b"-["); 
        
        let start = match start_offset {
            Some(offset) => i + offset,
            None => break,
        }; 

        let search_start = start + 2; 
        if search_start >= buffer.len() {
            break; 
        } 

        let end_offset = buffer[search_start..]
            .windows(2)
            .position(|window| window == b"]-"); 

        let end = match end_offset {
            Some(offset) => search_start + offset,
            None => break,
        }; 

        matches.push((start, end + 1)); 
        i = end + 2; 
    } 
    
    matches 
} 

pub fn execute_command(command: &[u8], data: &mut HashMap<String, String>) -> Vec<u8> { 
    Vec::<u8>::new() 
}   