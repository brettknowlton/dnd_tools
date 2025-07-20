use std::io;

mod character;
mod file_manager;
mod initiative;
mod dice;
mod input_handler;
mod events;
mod error_handling;
mod tests;

use character::Character;
use file_manager::{load_character_files, save_characters, display_character_info, display_single_character, display_all_characters, delete_character_menu};
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
        println!("\n=== DnD Tools Main Menu ===");
        println!("1. Characters");
        println!("2. Tools");
        println!("3. Exit");
        
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;
        match buffer.trim() {
            "1" => characters_menu(&mut characters),
            "2" => tools_menu(),
            "3" => {
                if exit_menu() {
                    ending = true;
                }
            }
            _ => println!("Invalid input"),
        };
    }
    Ok(())
}

fn characters_menu(characters: &mut Vec<Character>) {
    loop {
        println!("\n=== Characters Menu ===");
        println!("1. Creation");
        println!("2. Display single character");
        println!("3. Display all characters");
        println!("4. Character deletion");
        println!("0. Back to main menu");
        
        let mut buffer = String::new();
        if io::stdin().read_line(&mut buffer).is_err() {
            println!("Failed to read input");
            continue;
        }
        
        match buffer.trim() {
            "1" => {
                let new_c = create_character();
                characters.push(new_c);
                save_characters(characters.clone());
            }
            "2" => display_single_character(characters),
            "3" => display_all_characters(characters),
            "4" => delete_character_menu(characters),
            "0" => break,
            _ => println!("Invalid input"),
        }
    }
}

fn tools_menu() {
    loop {
        println!("\n=== Tools Menu ===");
        println!("1. Initiative tracker");
        println!("2. NPC randomizer");
        println!("3. Dice");
        println!("4. Combat tracker");
        println!("0. Back to main menu");
        
        let mut buffer = String::new();
        if io::stdin().read_line(&mut buffer).is_err() {
            println!("Failed to read input");
            continue;
        }
        
        match buffer.trim() {
            "1" => initiative_tracker_mode(),
            "2" => npc_randomizer_mode(),
            "3" => roll_dice_mode(),
            "4" => combat_tracker_mode(),
            "0" => break,
            _ => println!("Invalid input"),
        }
    }
}

fn exit_menu() -> bool {
    loop {
        println!("\n=== Exit Menu ===");
        println!("1. Save and exit");
        println!("2. Exit without save");
        println!("3. Cancel");
        println!("0. Back to main menu");
        
        let mut buffer = String::new();
        if io::stdin().read_line(&mut buffer).is_err() {
            println!("Failed to read input");
            continue;
        }
        
        match buffer.trim() {
            "1" => {
                println!("Saving and exiting...");
                return true;
            }
            "2" => {
                println!("Exiting without save...");
                return true;
            }
            "3" => {
                println!("Operation cancelled");
                return false;
            }
            "0" => return false,
            _ => println!("Invalid input"),
        }
    }
}


fn npc_randomizer_mode() {
    println!("\n=== NPC Randomizer ===");
    println!("Generating random NPC...");
    
    
    // Generate basic stats
    let ac = (rand::random::<u8>() % 11) + 10; // 10-20
    let hp = (rand::random::<u8>() % 41) + 10; // 10-50
    let speed = ((rand::random::<u8>() % 7) + 2) * 10; // 20-80 in increments of 10
    
    // Generate ability scores using normal distribution approximation
    // Using 3d6 for each stat (normal distribution around 10.5, range 3-18)
    let strength = roll_3d6();
    let dexterity = roll_3d6();
    let constitution = roll_3d6();
    let intelligence = roll_3d6();
    let wisdom = roll_3d6();
    let charisma = roll_3d6();
    
    println!("\n=== Generated NPC ===");
    println!("Name: [Not generated]");
    println!("AC: {}", ac);
    println!("HP: {}", hp);
    println!("Speed: {} feet", speed);
    println!("STR: {}", strength);
    println!("DEX: {}", dexterity);
    println!("CON: {}", constitution);
    println!("INT: {}", intelligence);
    println!("WIS: {}", wisdom);
    println!("CHA: {}", charisma);
    
    println!("\nPress Enter to continue...");
    let mut _buffer = String::new();
    let _ = io::stdin().read_line(&mut _buffer);
}

fn roll_3d6() -> u8 {
    let roll1 = (rand::random::<u8>() % 6) + 1;
    let roll2 = (rand::random::<u8>() % 6) + 1;
    let roll3 = (rand::random::<u8>() % 6) + 1;
    (roll1 + roll2 + roll3).clamp(1, 20)
}

fn combat_tracker_mode() {
    println!("\n=== Combat Tracker ===");
    println!("Starting with Initiative Tracker...");
    
    // First run the initiative tracker
    initiative_tracker_mode();
    
    println!("\nWould you like to enter Combat Mode? (y/n)");
    let mut buffer = String::new();
    if io::stdin().read_line(&mut buffer).is_err() {
        println!("Failed to read input. Exiting combat tracker.");
        return;
    }
    
    if buffer.trim().to_lowercase() == "y" {
        combat_mode();
    }
}

fn combat_mode() {
    println!("\n=== Combat Mode ===");
    println!("Combat tracker functionality is not yet fully implemented.");
    println!("Available commands: next, remove, quit");
    println!("Note: Initiative 0 creatures will be skipped, HP 0 creatures still get turns.");
    
    loop {
        println!("\nCombat > Enter command:");
        let mut buffer = String::new();
        if io::stdin().read_line(&mut buffer).is_err() {
            println!("Failed to read input");
            continue;
        }
        
        let input = buffer.trim().to_lowercase();
        match input.as_str() {
            "next" => println!("Next command - advancing to next combatant..."),
            "remove" => println!("Remove command - removes combatant from loop (doesn't touch files)..."),
            "quit" | "q" => break,
            "help" | "h" => {
                println!("Combat Mode Commands:");
                println!("  next - Advance to next player in initiative order");
                println!("  remove - Remove combatant from combat loop");
                println!("  quit - Exit combat mode");
            }
            _ => println!("Unknown command. Type 'help' for available commands."),
        }
    }
}
