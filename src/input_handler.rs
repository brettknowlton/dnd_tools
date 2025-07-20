use crate::character::Character;
use crate::error_handling::{Result, AppError, validate_character_name, validate_numeric_input};
use std::{io, collections::HashMap};

fn read_user_input(prompt: &str) -> Result<String> {
    println!("{}", prompt);
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(buffer.trim().to_string())
}

pub fn create_character() -> Character {
    println!("Creating a new character");
    
    let name = loop {
        match read_user_input("Enter the character's name:") {
            Ok(input) => {
                match validate_character_name(&input) {
                    Ok(_) => break input,
                    Err(e) => {
                        println!("Error: {}. Please try again.", e);
                        continue;
                    }
                }
            }
            Err(e) => {
                println!("Error reading input: {}. Using default name.", e);
                break "Unknown".to_string();
            }
        }
    };

    let mut character = Character::new(&name);
    println!("Character {} created!", name);

    loop {
        match read_user_input("Would you like to add more information to the character sheet?\n1. Yes\n2. No") {
            Ok(input) => {
                match input.as_str() {
                    "1" => {
                        println!("Adding more information to the character sheet");
                        character = data_entry(character);
                        break;
                    }
                    "2" => break,
                    _ => println!("Invalid input, please enter 1 or 2"),
                }
            }
            Err(e) => {
                println!("Error reading input: {}. Skipping additional information.", e);
                break;
            }
        }
    }

    character
}

pub fn data_entry(mut character: Character) -> Character {
    let data = character.as_vec();
    let stats = character.get_ordered_stats();
    let mut changes = HashMap::new();
    
    println!("Enter new values for character stats (press Enter to keep current value):");
    
    // Loop over each item in data, show what the current value is, and ask for an overwrite value
    for (index, item) in data.iter().enumerate() {
        if index >= stats.len() {
            break; // Safety check
        }
        
        let stat = &stats[index];
        println!("\nCurrent {}: {}", stat, item);
        
        match read_user_input(&format!("New value for {} (or press Enter to keep current):", stat)) {
            Ok(new_value) => {
                if !new_value.is_empty() {
                    // Extract the key from the stat string (everything before the colon)
                    if let Some(colon_pos) = stat.find(':') {
                        let key = stat[..colon_pos].to_lowercase().replace(' ', "_");
                        
                        // Validate numeric inputs
                        let is_valid = match key.as_str() {
                            "level" => validate_numeric_input(&new_value, &key, Some(1), Some(20)).is_ok(),
                            "ac" => validate_numeric_input(&new_value, &key, Some(1), Some(30)).is_ok(),
                            "hp" | "max_hp" | "temp_hp" => validate_numeric_input(&new_value, &key, Some(0), Some(255)).is_ok(),
                            "speed" => validate_numeric_input(&new_value, &key, Some(0), Some(100)).is_ok(),
                            "intelligence" | "wisdom" | "charisma" | "strength" | "dexterity" | "constitution" => {
                                validate_numeric_input(&new_value, &key, Some(1), Some(30)).is_ok()
                            }
                            "passive_perception" | "initiative" | "proficiency_bonus" => {
                                validate_numeric_input(&new_value, &key, Some(0), Some(50)).is_ok()
                            }
                            _ => true, // Non-numeric fields like name and description
                        };
                        
                        if is_valid {
                            println!("Updated {} from {} to {}", stat, item, new_value);
                            changes.insert(key, new_value);
                        } else {
                            println!("Invalid value for {}. Keeping current value.", stat);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error reading input for {}: {}. Keeping current value.", stat, e);
            }
        }
    }
    
    character.apply_hash_changes(changes)
}