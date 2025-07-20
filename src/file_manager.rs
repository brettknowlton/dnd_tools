use crate::character::Character;
use std::{fs, io::Write, path::Path};

pub fn load_character_files() -> Vec<Character> {
    let mut characters = Vec::new();
    if let Ok(paths) = fs::read_dir("characters") {
        for path in paths {
            if let Ok(path) = path {
                if let Ok(character_sheet) = fs::read_to_string(path.path()) {
                    if let Ok(character) = ron::de::from_str::<Character>(&character_sheet) {
                        characters.push(character);
                    }
                }
            }
        }
    }
    characters
}

pub fn save_characters(characters: Vec<Character>) {
    for character in characters {
        save_character(character.name.clone(), character);
    }
}

pub fn save_character(name: String, data: Character) {
    println!("Saving character sheet for {}", name);

    let path = format!("characters/{}.txt", name);
    if let Ok(mut file) = fs::File::create(path) {
        if let Ok(serialized) = ron::ser::to_string_pretty(&data, ron::ser::PrettyConfig::default()) {
            if file.write(serialized.as_bytes()).is_ok() {
                println!("Character sheet saved!");
            } else {
                println!("Failed to write character data to file");
            }
        } else {
            println!("Failed to serialize character data");
        }
    } else {
        println!("Failed to create character file");
    }
}

pub fn display_character_info() {
    println!("Enter the name of the character you would like to load:");

    let mut buffer = String::new();
    if std::io::stdin().read_line(&mut buffer).is_ok() {
        let name = buffer.trim();
        println!("Loading character sheet for {}", name);

        let path = format!("characters/{}.txt", name);
        match fs::read_to_string(Path::new(&path)) {
            Ok(character_sheet) => {
                println!("Read: {}", character_sheet);
                println!("Finished loading character sheet");
            }
            Err(e) => println!("Failed to read character sheet: {}", e),
        }
    } else {
        println!("Failed to read input");
    }
}

pub fn display_single_character(characters: &[Character]) {
    if characters.is_empty() {
        println!("No characters available.");
        return;
    }
    
    println!("\nSelect a character:");
    for (i, character) in characters.iter().enumerate() {
        println!("{}. {}", i + 1, character.name);
    }
    
    let mut buffer = String::new();
    if std::io::stdin().read_line(&mut buffer).is_ok() {
        if let Ok(choice) = buffer.trim().parse::<usize>() {
            if choice > 0 && choice <= characters.len() {
                let character = &characters[choice - 1];
                println!("\n=== Character Sheet ===");
                for stat in character.get_ordered_stats() {
                    println!("{}", stat);
                }
            } else {
                println!("Invalid selection.");
            }
        } else {
            println!("Invalid input. Please enter a number.");
        }
    } else {
        println!("Failed to read input");
    }
}

pub fn display_all_characters(characters: &[Character]) {
    if characters.is_empty() {
        println!("No characters available.");
        return;
    }
    
    println!("\n=== All Characters ===");
    for (i, character) in characters.iter().enumerate() {
        println!("\n--- Character {} ---", i + 1);
        for stat in character.get_ordered_stats() {
            println!("{}", stat);
        }
    }
}

pub fn delete_character_menu(characters: &mut Vec<Character>) {
    if characters.is_empty() {
        println!("No characters available to delete.");
        return;
    }
    
    println!("\nSelect a character to delete:");
    for (i, character) in characters.iter().enumerate() {
        println!("{}. {}", i + 1, character.name);
    }
    
    let mut buffer = String::new();
    if std::io::stdin().read_line(&mut buffer).is_ok() {
        if let Ok(choice) = buffer.trim().parse::<usize>() {
            if choice > 0 && choice <= characters.len() {
                let character = characters.remove(choice - 1);
                
                // Delete the character file
                let path = format!("characters/{}.txt", character.name);
                if let Err(e) = fs::remove_file(&path) {
                    println!("Warning: Could not delete character file {}: {}", path, e);
                }
                
                println!("Character '{}' deleted successfully.", character.name);
                save_characters(characters.clone());
            } else {
                println!("Invalid selection.");
            }
        } else {
            println!("Invalid input. Please enter a number.");
        }
    } else {
        println!("Failed to read input");
    }
}