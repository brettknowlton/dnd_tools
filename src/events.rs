use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Data {
    pub data: HashMap<char, Vec<String>>
}

impl Data {
    pub fn new() -> Self {
        Data {
            data: HashMap::new()
        }
    }
}

pub fn add_event(buffer: &str) {
    let mut buffer = buffer.trim().to_string();
    if buffer.is_empty() {
        return;
    }
    
    // Remove first character (the 'e' command)
    buffer.remove(0);

    match buffer.trim().chars().next() {
        Some('a') => {
            println!("Event management - add functionality coming soon");
        }
        Some('r') => {
            remove_event_from_file(&buffer);
        }
        Some('l') => {
            load_events();
        }
        _ => println!("Invalid event command. Use 'ea' to add, 'er' to remove, 'el' to load"),
    }

    if buffer.trim().is_empty() {
        return;
    }

    let parts: Vec<&str> = buffer.split_whitespace().collect();
    if parts.len() >= 3 {
        let event = parts[0];
        let time = parts[1];
        let desc = parts[2..].join(" ");
        println!("Event: {}\nTime: {}\nDescription: {}", event, time, desc);
    }
}

fn remove_event_from_file(_buffer: &str) {
    println!("Remove event functionality coming soon");
}

fn load_events() {
    println!("Load events functionality coming soon");
}