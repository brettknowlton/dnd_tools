use serde::{Deserialize, Serialize};
use std::io;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitiativeEntry {
    pub name: String,
    pub initiative: i32,
    pub is_player: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiativeTracker {
    entries: Vec<InitiativeEntry>,
    current_turn: usize,
}

impl InitiativeTracker {
    pub fn new() -> Self {
        InitiativeTracker {
            entries: Vec::new(),
            current_turn: 0,
        }
    }

    pub fn add_entry(&mut self, name: String, initiative: i32, is_player: bool) {
        let entry = InitiativeEntry {
            name,
            initiative,
            is_player,
        };
        self.entries.push(entry);
        self.sort_by_initiative();
    }

    fn sort_by_initiative(&mut self) {
        self.entries.sort_by(|a, b| b.initiative.cmp(&a.initiative));
        self.current_turn = 0;
    }

    pub fn next_turn(&mut self) -> Option<&InitiativeEntry> {
        if self.entries.is_empty() {
            return None;
        }
        let current = &self.entries[self.current_turn];
        self.current_turn = (self.current_turn + 1) % self.entries.len();
        Some(current)
    }

    pub fn display(&self) {
        println!("Initiative Order:");
        for (i, entry) in self.entries.iter().enumerate() {
            let marker = if i == self.current_turn { ">>> " } else { "    " };
            let player_type = if entry.is_player { "(Player)" } else { "(NPC)" };
            println!("{}Initiative {}: {} {}", marker, entry.initiative, entry.name, player_type);
        }
    }

    pub fn remove_entry(&mut self, name: &str) -> bool {
        if let Some(pos) = self.entries.iter().position(|entry| entry.name == name) {
            self.entries.remove(pos);
            if self.current_turn >= self.entries.len() && !self.entries.is_empty() {
                self.current_turn = 0;
            }
            true
        } else {
            false
        }
    }
    
    pub fn get_entries(&self) -> &Vec<InitiativeEntry> {
        &self.entries
    }
}

pub fn initiative_tracker_mode() {
    let mut tracker = InitiativeTracker::new();
    let mut ending = false;
    
    println!("Welcome to the Initiative Tracker!");
    println!("Commands: add, remove, next, display, clear, quit, help");
    
    while !ending {
        println!("\nInitiative Tracker > Enter command:");
        let mut buffer = String::new();
        if io::stdin().read_line(&mut buffer).is_err() {
            println!("Failed to read input");
            continue;
        }
        
        let input = buffer.trim().to_lowercase();
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        match parts.get(0) {
            Some(&"add") => {
                if parts.len() >= 3 {
                    let name = parts[1].to_string();
                    if let Ok(initiative) = parts[2].parse::<i32>() {
                        let is_player = parts.get(3).map_or(true, |&s| s == "player");
                        tracker.add_entry(name, initiative, is_player);
                        println!("Added to initiative tracker!");
                        tracker.display();
                    } else {
                        println!("Invalid initiative value. Please enter a number.");
                    }
                } else {
                    println!("Usage: add <name> <initiative> [player|npc]");
                    println!("Example: add Gandalf 18 player");
                }
            }
            Some(&"remove") => {
                if parts.len() >= 2 {
                    let name = parts[1];
                    if tracker.remove_entry(name) {
                        println!("Removed {} from initiative tracker", name);
                        tracker.display();
                    } else {
                        println!("Could not find {} in initiative tracker", name);
                    }
                } else {
                    println!("Usage: remove <name>");
                }
            }
            Some(&"next") => {
                if let Some(current) = tracker.next_turn() {
                    println!("Current turn: {} (Initiative: {})", current.name, current.initiative);
                    tracker.display();
                } else {
                    println!("No entries in initiative tracker. Use 'add' to add some!");
                }
            }
            Some(&"display") => {
                tracker.display();
            }
            Some(&"clear") => {
                tracker = InitiativeTracker::new();
                println!("Initiative tracker cleared!");
            }
            Some(&"quit") | Some(&"q") => {
                ending = true;
            }
            Some(&"help") | Some(&"h") => {
                println!("Commands:");
                println!("  add <name> <initiative> [player|npc] - Add entry to tracker");
                println!("  remove <name> - Remove entry from tracker");
                println!("  next - Advance to next turn");
                println!("  display - Show current initiative order");
                println!("  clear - Clear all entries");
                println!("  quit - Exit initiative tracker");
            }
            _ => {
                println!("Unknown command. Type 'help' for available commands.");
            }
        }
    }
}