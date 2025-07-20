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