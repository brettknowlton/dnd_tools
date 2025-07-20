use std::io;

mod character;
mod file_manager;
mod initiative;
mod dice;
mod input_handler;
mod events;
mod error_handling;
mod tests;

use file_manager::{load_character_files, save_characters, display_character_info};
use initiative::initiative_tracker_mode;
use dice::roll_dice_mode;
use input_handler::create_character;
use events::Data;


fn main() -> io::Result<()> {
    println!("Welcome to DnD tools!");
    let mut characters = load_character_files();
    println!("Loaded {} character sheets:", characters.len());
    for character_sheet in &characters {
        println!("{:?}\n", character_sheet);
    }

    let _events = Data::new();

    let mut ending = false;
    while !ending {
        println!("What would you like to do?");
        println!("1. Create a new character");
        println!("2. Display character info");
        println!("3. Roll Dice");
        println!("4. Initiative Tracker");
        println!("0. Exit");
        
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;
        match buffer.trim() {
            "1" => {
                let new_c = create_character();
                characters.push(new_c);
                save_characters(characters.clone());
            }
            "2" => display_character_info(),
            "3" => roll_dice_mode(),
            "4" => initiative_tracker_mode(),
            "0" => ending = true,
            _ => println!("Invalid input"),
        };
    }
    Ok(())
}


