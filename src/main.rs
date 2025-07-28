use std::io::{self, Write};
use std::process;
use crate::search::{DndSearchClient, SearchCategory, SearchResult};

mod character;
mod file_manager;
mod initiative;
mod dice;
mod input_handler;
mod events;
mod error_handling;
mod combat;
mod tests;
mod races_classes;
mod search;
mod tui;

fn clear_console() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap_or(());
}

/// Check if input is a universal exit command and exit the program if so
fn check_universal_exit(input: &str) {
    let trimmed = input.trim();
    if trimmed.to_uppercase() == "EXIT" || trimmed.to_uppercase() == "QUIT" {
        println!("\n🚪 Universal EXIT command detected - terminating program...");
        println!("Goodbye! 👋");
        process::exit(0);
    }
}

use character::Character;
use file_manager::{load_character_files, save_characters, display_single_character, display_all_characters, delete_character_menu};
use initiative::initiative_tracker_mode;
use dice::{roll_dice_mode};
use input_handler::create_character;
use events::Data;
use combat::{enhanced_initiative_setup, CombatTracker, StatusEffect, Combatant};


fn main() -> io::Result<()> {
    println!("Welcome to DnD tools!");
    let characters = load_character_files();
    println!("Loaded {} character sheets.", characters.len());

    let _events = Data::new();

    // Initialize TUI
    let app = tui::App::new(characters);
    
    match tui::run_tui(app) {
        Ok(final_app) => {
            // Save any character changes before exiting
            save_characters(final_app.characters);
            println!("Goodbye! 👋");
        }
        Err(e) => {
            eprintln!("Error running TUI: {}", e);
            // Fall back to CLI mode if TUI fails
            println!("Falling back to CLI mode...");
            run_cli_mode(load_character_files())?;
        }
    }
    
    Ok(())
}

fn run_cli_mode(mut characters: Vec<Character>) -> io::Result<()> {
    println!("Running in CLI mode...");
    
    let mut ending = false;
    while !ending {
        println!("\n=== DnD Tools Main Menu ===");
        println!("1. Characters");
        println!("2. Tools");
        println!("3. Exit");
        
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;
        
        // Check for universal exit command
        check_universal_exit(&buffer);
        
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
        println!("3. Encounter randomizer");
        println!("4. Dice");
        println!("5. Combat tracker");
        println!("6. Search D&D 5e API");
        println!("0. Back to main menu");
        
        let mut buffer = String::new();
        if io::stdin().read_line(&mut buffer).is_err() {
            println!("Failed to read input");
            continue;
        }
        
        match buffer.trim() {
            "1" => initiative_tracker_mode(),
            "2" => npc_randomizer_mode(),
            "3" => encounter_randomizer_mode(),
            "4" => roll_dice_mode(),
            "5" => combat_tracker_mode(),
            "6" => search_mode(),
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


pub fn npc_randomizer_mode() {
    println!("\n=== NPC Generator ===");
    
    // Ask for manual or generated stats
    println!("Would you like to:");
    println!("1. Generate all stats randomly");
    println!("2. Enter stats manually");
    println!("3. Generate with custom race/class");
    
    let mut buffer = String::new();
    if io::stdin().read_line(&mut buffer).is_err() {
        println!("Failed to read input, defaulting to random generation");
        generate_random_npc();
        return;
    }
    
    match buffer.trim() {
        "1" => generate_random_npc(),
        "2" => generate_manual_npc(), 
        "3" => generate_custom_npc(),
        _ => {
            println!("Invalid choice, defaulting to random generation");
            generate_random_npc();
        }
    }
}

fn generate_random_npc() {
    use crate::races_classes::{get_random_race, get_random_class};
    
    println!("\n=== Generating Random NPC ===");
    
    // Generate race and class
    let race = get_random_race();
    let class = get_random_class();
    
    // Generate basic stats
    let ac = (rand::random::<u8>() % 11) + 10; // 10-20
    let hp = (rand::random::<u8>() % 41) + 10; // 10-50
    let speed = ((rand::random::<u8>() % 7) + 2) * 10; // 20-80 in increments of 10
    
    // Generate ability scores using 3d6 for each stat
    let strength = roll_3d6();
    let dexterity = roll_3d6();
    let constitution = roll_3d6();
    let intelligence = roll_3d6();
    let wisdom = roll_3d6();
    let charisma = roll_3d6();
    
    println!("\n╔═══════════════════════════════════════╗");
    println!("║            Generated NPC              ║");
    println!("╠═══════════════════════════════════════╣");
    println!("║ Race: {:<31} ║", race);
    println!("║ Class: {:<30} ║", class);
    println!("║ AC: {:<33} ║", ac);
    println!("║ HP: {:<33} ║", hp);
    println!("║ Speed: {} feet{:<21} ║", speed, "");
    println!("║                                       ║");
    println!("║ Ability Scores:                       ║");
    println!("║   STR: {:<29} ║", strength);
    println!("║   DEX: {:<29} ║", dexterity);
    println!("║   CON: {:<29} ║", constitution);
    println!("║   INT: {:<29} ║", intelligence);
    println!("║   WIS: {:<29} ║", wisdom);
    println!("║   CHA: {:<29} ║", charisma);
    println!("╚═══════════════════════════════════════╝");
    
    // Ask if they want to save this NPC
    println!("\nSave this NPC? (y/n): ");
    let mut save_input = String::new();
    if io::stdin().read_line(&mut save_input).is_ok() && save_input.trim().to_lowercase() == "y" {
        save_generated_npc(&race, &class, ac, hp, speed, strength, dexterity, constitution, intelligence, wisdom, charisma);
    }
    
    println!("\nPress Enter to continue...");
    let mut _buffer = String::new();
    let _ = io::stdin().read_line(&mut _buffer);
}

fn generate_manual_npc() {
    use crate::races_classes::{list_races, list_classes};
    
    println!("\n=== Manual NPC Creation ===");
    
    // Get name
    println!("NPC Name: ");
    let mut name = String::new();
    if io::stdin().read_line(&mut name).is_err() {
        println!("Failed to read name, using default");
        name = "Unknown NPC".to_string();
    }
    let name = name.trim().to_string();
    
    // Show race options
    let races = list_races();
    println!("\nAvailable Races:");
    for (i, race) in races.iter().enumerate() {
        print!("{:<12} ", race);
        if (i + 1) % 6 == 0 { println!(); }
    }
    println!("\nRace (or press Enter for random): ");
    let mut race_input = String::new();
    let race = if io::stdin().read_line(&mut race_input).is_ok() {
        let input = race_input.trim();
        if input.is_empty() {
            crate::races_classes::get_random_race()
        } else {
            input.to_string()
        }
    } else {
        crate::races_classes::get_random_race()
    };
    
    // Show class options  
    let classes = list_classes();
    println!("\nAvailable Classes:");
    for (i, class) in classes.iter().enumerate() {
        print!("{:<12} ", class);
        if (i + 1) % 4 == 0 { println!(); }
    }
    println!("\nClass (or press Enter for random): ");
    let mut class_input = String::new();
    let class = if io::stdin().read_line(&mut class_input).is_ok() {
        let input = class_input.trim();
        if input.is_empty() {
            crate::races_classes::get_random_class()
        } else {
            input.to_string()
        }
    } else {
        crate::races_classes::get_random_class()
    };
    
    // Get other stats manually
    let ac = prompt_for_number("AC (10-25)", 10, 25).unwrap_or(12);
    let hp = prompt_for_number("HP (1-200)", 1, 200).unwrap_or(20);
    let speed = prompt_for_number("Speed (10-120)", 10, 120).unwrap_or(30);
    
    println!("\nAbility Scores (3-18, or press Enter to roll 3d6):");
    let strength = prompt_for_ability_score("Strength").unwrap_or_else(|| roll_3d6());
    let dexterity = prompt_for_ability_score("Dexterity").unwrap_or_else(|| roll_3d6());
    let constitution = prompt_for_ability_score("Constitution").unwrap_or_else(|| roll_3d6());
    let intelligence = prompt_for_ability_score("Intelligence").unwrap_or_else(|| roll_3d6());
    let wisdom = prompt_for_ability_score("Wisdom").unwrap_or_else(|| roll_3d6());
    let charisma = prompt_for_ability_score("Charisma").unwrap_or_else(|| roll_3d6());
    
    // Display the created NPC
    println!("\n╔═══════════════════════════════════════╗");
    println!("║            Created NPC                ║");
    println!("╠═══════════════════════════════════════╣");
    println!("║ Name: {:<31} ║", name);
    println!("║ Race: {:<31} ║", race);
    println!("║ Class: {:<30} ║", class);
    println!("║ AC: {:<33} ║", ac);
    println!("║ HP: {:<33} ║", hp);
    println!("║ Speed: {} feet{:<21} ║", speed, "");
    println!("║                                       ║");
    println!("║ Ability Scores:                       ║");
    println!("║   STR: {:<29} ║", strength);
    println!("║   DEX: {:<29} ║", dexterity);
    println!("║   CON: {:<29} ║", constitution);
    println!("║   INT: {:<29} ║", intelligence);
    println!("║   WIS: {:<29} ║", wisdom);
    println!("║   CHA: {:<29} ║", charisma);
    println!("╚═══════════════════════════════════════╝");
    
    // Save the NPC
    save_generated_npc(&race, &class, ac, hp, speed, strength, dexterity, constitution, intelligence, wisdom, charisma);
    
    println!("\nPress Enter to continue...");
    let mut _buffer = String::new();
    let _ = io::stdin().read_line(&mut _buffer);
}

fn generate_custom_npc() {
    use crate::races_classes::{list_races, list_classes};
    
    println!("\n=== Custom NPC Generation ===");
    
    // Get race selection
    let races = list_races();
    println!("Available Races:");
    for (i, race) in races.iter().enumerate() {
        print!("{:<12} ", race);
        if (i + 1) % 6 == 0 { println!(); }
    }
    println!("\nSelect race (or press Enter for random): ");
    let mut race_input = String::new();
    let race = if io::stdin().read_line(&mut race_input).is_ok() {
        let input = race_input.trim();
        if input.is_empty() {
            crate::races_classes::get_random_race()
        } else {
            input.to_string()
        }
    } else {
        crate::races_classes::get_random_race()
    };
    
    // Get class selection
    let classes = list_classes();
    println!("\nAvailable Classes:");
    for (i, class) in classes.iter().enumerate() {
        print!("{:<12} ", class);
        if (i + 1) % 4 == 0 { println!(); }
    }
    println!("\nSelect class (or press Enter for random): ");
    let mut class_input = String::new();
    let class = if io::stdin().read_line(&mut class_input).is_ok() {
        let input = class_input.trim();
        if input.is_empty() {
            crate::races_classes::get_random_class()
        } else {
            input.to_string()
        }
    } else {
        crate::races_classes::get_random_class()
    };
    
    // Generate other stats randomly
    let ac = (rand::random::<u8>() % 11) + 10;
    let hp = (rand::random::<u8>() % 41) + 10;
    let speed = ((rand::random::<u8>() % 7) + 2) * 10;
    
    let strength = roll_3d6();
    let dexterity = roll_3d6();
    let constitution = roll_3d6();
    let intelligence = roll_3d6();
    let wisdom = roll_3d6();
    let charisma = roll_3d6();
    
    println!("\n╔═══════════════════════════════════════╗");
    println!("║       Custom Generated NPC            ║");
    println!("╠═══════════════════════════════════════╣");
    println!("║ Race: {:<31} ║", race);
    println!("║ Class: {:<30} ║", class);
    println!("║ AC: {:<33} ║", ac);
    println!("║ HP: {:<33} ║", hp);
    println!("║ Speed: {} feet{:<21} ║", speed, "");
    println!("║                                       ║");
    println!("║ Ability Scores:                       ║");
    println!("║   STR: {:<29} ║", strength);
    println!("║   DEX: {:<29} ║", dexterity);
    println!("║   CON: {:<29} ║", constitution);
    println!("║   INT: {:<29} ║", intelligence);
    println!("║   WIS: {:<29} ║", wisdom);
    println!("║   CHA: {:<29} ║", charisma);
    println!("╚═══════════════════════════════════════╝");
    
    // Ask if they want to save this NPC
    println!("\nSave this NPC? (y/n): ");
    let mut save_input = String::new();
    if io::stdin().read_line(&mut save_input).is_ok() && save_input.trim().to_lowercase() == "y" {
        save_generated_npc(&race, &class, ac, hp, speed, strength, dexterity, constitution, intelligence, wisdom, charisma);
    }
    
    println!("\nPress Enter to continue...");
    let mut _buffer = String::new();
    let _ = io::stdin().read_line(&mut _buffer);
}

fn prompt_for_number(prompt: &str, min: u8, max: u8) -> Option<u8> {
    println!("{} ({}-{}): ", prompt, min, max);
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        if let Ok(num) = input.trim().parse::<u8>() {
            if num >= min && num <= max {
                return Some(num);
            }
        }
    }
    None
}

fn prompt_for_ability_score(ability: &str) -> Option<u8> {
    println!("{} (3-18, or Enter to roll 3d6): ", ability);
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None; // Will trigger random roll
        }
        if let Ok(score) = trimmed.parse::<u8>() {
            if score >= 3 && score <= 18 {
                return Some(score);
            }
        }
    }
    None
}

fn save_generated_npc(race: &str, class: &str, ac: u8, hp: u8, speed: u8, str: u8, dex: u8, con: u8, int: u8, wis: u8, cha: u8) {
    use std::fs;
    
    println!("Enter NPC name to save: ");
    let mut name_input = String::new();
    if io::stdin().read_line(&mut name_input).is_err() {
        println!("Failed to read name, not saving");
        return;
    }
    
    let name = name_input.trim();
    if name.is_empty() {
        println!("No name provided, not saving");
        return;
    }
    
    // Create npcs directory if it doesn't exist
    if let Err(e) = fs::create_dir_all("npcs") {
        println!("Failed to create npcs directory: {}", e);
        return;
    }
    
    let path = format!("npcs/{}.txt", name);
    
    let npc_data = format!(
        "Name: {}\nRace: {}\nClass: {}\nAC: {}\nHP: {}\nSpeed: {}\nSTR: {}\nDEX: {}\nCON: {}\nINT: {}\nWIS: {}\nCHA: {}",
        name, race, class, ac, hp, speed, str, dex, con, int, wis, cha
    );
    
    match fs::write(&path, npc_data) {
        Ok(_) => println!("✅ Saved NPC '{}' to {}", name, path),
        Err(e) => println!("❌ Failed to save NPC: {}", e),
    }
}

fn roll_3d6() -> u8 {
    let roll1 = (rand::random::<u8>() % 6) + 1;
    let roll2 = (rand::random::<u8>() % 6) + 1;
    let roll3 = (rand::random::<u8>() % 6) + 1;
    (roll1 + roll2 + roll3).clamp(1, 20)
}

pub fn encounter_randomizer_mode() {
    println!("\n⚔️  === Encounter Randomizer === ⚔️");
    println!("Generate balanced encounters based on Challenge Rating (CR)");
    println!("═══════════════════════════════════════════════════════════");
    
    // Ask for Combat Rating
    println!("\nEnter Combat Rating (CR):");
    println!("Examples: 0 (CR 1/4), 1 (CR 1), 5 (CR 5), 10 (CR 10)");
    println!("Or fractional: 0.125 (CR 1/8), 0.25 (CR 1/4), 0.5 (CR 1/2)");
    print!("CR: ");
    io::stdout().flush().unwrap();
    
    let mut cr_input = String::new();
    if io::stdin().read_line(&mut cr_input).is_err() {
        println!("Failed to read input, defaulting to CR 1");
        generate_encounter_by_cr(1.0);
        return;
    }
    
    let cr = match cr_input.trim().parse::<f64>() {
        Ok(val) if val >= 0.0 && val <= 30.0 => val,
        _ => {
            println!("Invalid CR, defaulting to CR 1");
            1.0
        }
    };
    
    generate_encounter_by_cr(cr);
}

fn generate_encounter_by_cr(cr: f64) {
    use crate::races_classes::{get_random_race, get_random_class};
    
    println!("\n🎲 Generating encounter for CR {}...", format_cr(cr));
    
    // Ask for number of creatures
    println!("\nHow many creatures in this encounter? (1-8): ");
    let mut num_input = String::new();
    if io::stdin().read_line(&mut num_input).is_err() {
        println!("Failed to read input, generating 1 creature");
    }
    
    let num_creatures = match num_input.trim().parse::<usize>() {
        Ok(n) if n >= 1 && n <= 8 => n,
        _ => {
            println!("Invalid number, generating 1 creature");
            1
        }
    };
    
    // Calculate individual creature CR for balanced encounter
    let individual_cr = if num_creatures == 1 {
        cr
    } else {
        // For multiple creatures, reduce individual CR
        let multiplier = match num_creatures {
            2 => 0.7,
            3..=4 => 0.5,
            5..=6 => 0.4,
            _ => 0.3,
        };
        (cr * multiplier).max(0.125)
    };
    
    println!("\n🎯 Creating {} creature(s) at individual CR {}", 
             num_creatures, format_cr(individual_cr));
    println!("═══════════════════════════════════════════════════════════");
    
    let mut encounter = Vec::new();
    
    for i in 1..=num_creatures {
        let creature = generate_creature_by_cr(individual_cr, i);
        encounter.push(creature.clone());
        
        display_creature(&creature);
        println!();
    }
    
    // Show encounter summary
    println!("📋 === Encounter Summary ===");
    println!("Total Challenge Rating: {}", format_cr(cr));
    println!("Creatures: {}", num_creatures);
    for (i, creature) in encounter.iter().enumerate() {
        println!("  {}. {} (CR {}, HP: {}, AC: {})", 
                 i + 1, creature.name, format_cr(creature.cr), creature.hp, creature.ac);
    }
    
    // Ask if they want to save the encounter
    println!("\nSave this encounter? (y/n): ");
    let mut save_input = String::new();
    if io::stdin().read_line(&mut save_input).is_ok() && save_input.trim().to_lowercase() == "y" {
        save_encounter(&encounter, cr);
    }
    
    println!("\nPress Enter to continue...");
    let mut _buffer = String::new();
    let _ = io::stdin().read_line(&mut _buffer);
}

#[derive(Clone)]
struct EncounterCreature {
    name: String,
    race: String,
    class: String,
    cr: f64,
    hp: i32,
    ac: i32,
    speed: i32,
    strength: u8,
    dexterity: u8,
    constitution: u8,
    intelligence: u8,
    wisdom: u8,
    charisma: u8,
    proficiency_bonus: i32,
}

fn generate_creature_by_cr(cr: f64, creature_num: usize) -> EncounterCreature {
    use crate::races_classes::{get_random_race, get_random_class};
    
    let race = get_random_race();
    let class = get_random_class();
    let name = format!("{} {} #{}", race, class, creature_num);
    
    // Calculate stats based on CR
    let (base_hp, base_ac, prof_bonus) = match cr {
        x if x < 0.25 => (7, 11, 2),   // CR 1/8, 1/4
        x if x < 0.5 => (22, 12, 2),   // CR 1/4
        x if x < 1.0 => (32, 13, 2),   // CR 1/2
        x if x < 2.0 => (58, 13, 2),   // CR 1
        x if x < 3.0 => (71, 13, 2),   // CR 2
        x if x < 4.0 => (84, 14, 2),   // CR 3
        x if x < 5.0 => (97, 14, 2),   // CR 4
        x if x < 6.0 => (110, 15, 3),  // CR 5
        x if x < 7.0 => (123, 15, 3),  // CR 6
        x if x < 8.0 => (136, 15, 3),  // CR 7
        x if x < 9.0 => (149, 16, 3),  // CR 8
        x if x < 10.0 => (162, 16, 4), // CR 9
        x if x < 11.0 => (175, 17, 4), // CR 10
        x if x < 13.0 => (195, 17, 4), // CR 11-12
        x if x < 15.0 => (225, 18, 5), // CR 13-14
        x if x < 17.0 => (255, 18, 5), // CR 15-16
        x if x < 21.0 => (285, 19, 6), // CR 17-20
        _ => (400, 20, 7),              // CR 21+
    };
    
    // Add some randomization to base stats (±20%)
    let hp_variance = (base_hp as f64 * 0.2) as i32;
    let hp = (base_hp + (rand::random::<i32>() % (hp_variance * 2 + 1)) - hp_variance).max(1);
    
    let ac_variance = if base_ac < 15 { 1 } else { 2 };
    let ac = base_ac + (rand::random::<i32>() % (ac_variance * 2 + 1)) - ac_variance;
    
    let speed = (((rand::random::<u8>() % 4) + 3) * 10) as i32; // 30-60 feet
    
    // Generate ability scores scaled by CR
    let ability_bonus = match cr {
        x if x < 1.0 => 0,
        x if x < 5.0 => 1,
        x if x < 10.0 => 2,
        x if x < 15.0 => 3,
        x if x < 20.0 => 4,
        _ => 5,
    };
    
    let strength = (roll_3d6() + ability_bonus as u8).min(20);
    let dexterity = (roll_3d6() + ability_bonus as u8).min(20);
    let constitution = (roll_3d6() + ability_bonus as u8).min(20);
    let intelligence = (roll_3d6() + ability_bonus as u8).min(20);
    let wisdom = (roll_3d6() + ability_bonus as u8).min(20);
    let charisma = (roll_3d6() + ability_bonus as u8).min(20);
    
    EncounterCreature {
        name,
        race,
        class,
        cr,
        hp,
        ac,
        speed,
        strength,
        dexterity,
        constitution,
        intelligence,
        wisdom,
        charisma,
        proficiency_bonus: prof_bonus,
    }
}

fn display_creature(creature: &EncounterCreature) {
    println!("╔═══════════════════════════════════════╗");
    println!("║ {:<37} ║", creature.name);
    println!("╠═══════════════════════════════════════╣");
    println!("║ CR: {:<33} ║", format_cr(creature.cr));
    println!("║ Race: {:<31} ║", creature.race);
    println!("║ Class: {:<30} ║", creature.class);
    println!("║ HP: {:<33} ║", creature.hp);
    println!("║ AC: {:<33} ║", creature.ac);
    println!("║ Speed: {} feet{:<21} ║", creature.speed, "");
    println!("║ Proficiency: +{:<26} ║", creature.proficiency_bonus);
    println!("║                                       ║");
    println!("║ Ability Scores:                       ║");
    println!("║   STR: {:<29} ║", creature.strength);
    println!("║   DEX: {:<29} ║", creature.dexterity);
    println!("║   CON: {:<29} ║", creature.constitution);
    println!("║   INT: {:<29} ║", creature.intelligence);
    println!("║   WIS: {:<29} ║", creature.wisdom);
    println!("║   CHA: {:<29} ║", creature.charisma);
    println!("╚═══════════════════════════════════════╝");
}

pub fn format_cr(cr: f64) -> String {
    if cr < 1.0 {
        match cr {
            x if (x - 0.125).abs() < 0.001 => "1/8".to_string(),
            x if (x - 0.25).abs() < 0.001 => "1/4".to_string(),
            x if (x - 0.5).abs() < 0.001 => "1/2".to_string(),
            _ => format!("{:.3}", cr),
        }
    } else if cr.fract() == 0.0 {
        format!("{}", cr as i32)
    } else {
        format!("{:.1}", cr)
    }
}

fn save_encounter(encounter: &[EncounterCreature], total_cr: f64) {
    use std::fs;
    
    println!("Enter encounter name to save: ");
    let mut name_input = String::new();
    if io::stdin().read_line(&mut name_input).is_err() {
        println!("Failed to read name, not saving");
        return;
    }
    
    let name = name_input.trim();
    if name.is_empty() {
        println!("No name provided, not saving");
        return;
    }
    
    // Create encounters directory if it doesn't exist
    if let Err(e) = fs::create_dir_all("encounters") {
        println!("Failed to create encounters directory: {}", e);
        return;
    }
    
    let path = format!("encounters/{}_CR{}.txt", name, format_cr(total_cr));
    
    let mut encounter_data = format!("Encounter: {}\nTotal CR: {}\nCreatures: {}\n\n", 
                                     name, format_cr(total_cr), encounter.len());
    
    for (i, creature) in encounter.iter().enumerate() {
        encounter_data.push_str(&format!(
            "=== Creature {} ===\n\
            Name: {}\n\
            CR: {}\n\
            Race: {}\n\
            Class: {}\n\
            HP: {}\n\
            AC: {}\n\
            Speed: {}\n\
            Proficiency: +{}\n\
            STR: {}\n\
            DEX: {}\n\
            CON: {}\n\
            INT: {}\n\
            WIS: {}\n\
            CHA: {}\n\n",
            i + 1, creature.name, format_cr(creature.cr), creature.race, creature.class,
            creature.hp, creature.ac, creature.speed, creature.proficiency_bonus,
            creature.strength, creature.dexterity, creature.constitution,
            creature.intelligence, creature.wisdom, creature.charisma
        ));
    }
    
    match fs::write(&path, encounter_data) {
        Ok(_) => println!("✅ Saved encounter '{}' to {}", name, path),
        Err(e) => println!("❌ Failed to save encounter: {}", e),
    }
}

pub fn combat_tracker_mode() {
    println!("\n⚔️  Enhanced Combat Tracker ⚔️");
    println!("Starting with Initiative setup...\n");
    
    // Set up initiative with enhanced features
    let mut combat_tracker = enhanced_initiative_setup();
    
    if combat_tracker.combatants.is_empty() {
        println!("❌ No combatants added. Exiting combat tracker.");
        return;
    }
    
    combat_tracker.display_initiative_order();
    
    println!("\n🚀 Ready to begin combat? (y/n)");
    let mut buffer = String::new();
    if io::stdin().read_line(&mut buffer).is_err() {
        println!("Failed to read input. Exiting combat tracker.");
        return;
    }
    
    if buffer.trim().to_lowercase() == "y" || buffer.trim().to_lowercase() == "yes" {
        // Ask for current HP for all combatants
        println!("\n💖 Please confirm current HP for all combatants:");
        for combatant in &mut combat_tracker.combatants {
            println!("Current HP for {} (max: {}): ", combatant.name, combatant.max_hp);
            let mut hp_input = String::new();
            if io::stdin().read_line(&mut hp_input).is_ok() {
                if let Ok(hp) = hp_input.trim().parse::<i32>() {
                    combatant.current_hp = hp;
                    println!("✅ Set {}'s HP to {}", combatant.name, hp);
                } else {
                    println!("Invalid input, keeping current HP: {}", combatant.current_hp);
                }
            }
        }
        
        enhanced_combat_mode(combat_tracker);
    }
}

fn enhanced_combat_mode(mut combat_tracker: CombatTracker) {
    println!("\n⚔️  COMBAT MODE ACTIVATED ⚔️");
    println!("═══════════════════════════════════════════════════════════");
    println!("Available commands:");
    println!("  📊 stats [name] - Show character stats");
    println!("  ⚔️  attack <target> - Roll attack vs target's AC");
    println!("  🎭 status [add|remove|list] [self|name] <status> - Manage status effects");
    println!("  🎲 save [ability] [self|name] - Make saving throw (e.g., save wis Gandalf)");
    println!("  🔍 search <query> - Search D&D 5e API (returns to combat after)");
    println!("  ➡️  next|continue - Advance to next combatant");
    println!("  ⬅️  back - Go back to previous combatant's turn");
    println!("  ➕ insert <name> - Add new combatant mid-fight");
    println!("  🗑️  remove <name> - Remove combatant from combat");
    println!("  💾 save <npc_name> - Save NPC to npcs/ directory");
    println!("  🔍 show|list - Display current initiative order");
    println!("  ❓ help - Show this help");
    println!("  🚪 quit - Exit combat mode (auto-saves characters)");
    println!("═══════════════════════════════════════════════════════════");
    
    // Start the first turn
    if let Some(current_combatant) = combat_tracker.next_turn() {
        println!("\n🎯 Starting combat with {}", current_combatant.name);
        current_combatant.display_stats();
    }
    
    loop {
        println!("\nCombat > Enter command:");
        let mut buffer = String::new();
        if io::stdin().read_line(&mut buffer).is_err() {
            println!("Failed to read input");
            continue;
        }
        
        let input = buffer.trim();
        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts.get(0).map(|s| s.to_lowercase()).unwrap_or_default();
        
        match command.as_str() {
            "stats" => {
                if let Some(name) = parts.get(1) {
                    if let Some(combatant) = combat_tracker.get_combatant(name) {
                        combatant.display_stats();
                    } else {
                        println!("❌ Combatant '{}' not found", name);
                    }
                } else {
                    println!("Usage: stats <name>");
                }
            }
            "search" => {
                if let Some(_query) = parts.get(1) {
                    let full_query = parts[1..].join(" ");
                    handle_search_in_combat(&full_query);
                } else {
                    println!("Usage: search <query>");
                    println!("Example: search fireball");
                }
            }
            "attack" => {
                if let Some(target_name) = parts.get(1) {
                    handle_attack_command(&mut combat_tracker, target_name);
                } else {
                    println!("Usage: attack <target>");
                }
            }
            "status" => {
                handle_status_command(&mut combat_tracker, &parts[1..]);
            }
            "next" | "continue" => {
                clear_console();
                if let Some(next_combatant) = combat_tracker.next_turn() {
                    println!("\n🎯 It's {}'s turn!", next_combatant.name);
                    next_combatant.display_stats();
                } else {
                    println!("❌ No combatants available for turns");
                }
            }
            "back" => {
                if combat_tracker.previous_turn() {
                    clear_console();
                    if let Some(prev_combatant) = combat_tracker.get_current_combatant() {
                        println!("\n⬅️  Going back to {}'s turn!", prev_combatant.name);
                        prev_combatant.display_stats();
                    }
                } else {
                    println!("❌ Cannot go back further");
                }
            }
            "insert" => {
                if let Some(name) = parts.get(1) {
                    handle_insert_combatant(&mut combat_tracker, name);
                } else {
                    println!("Usage: insert <combatant_name>");
                }
            }
            "remove" => {
                if let Some(name) = parts.get(1) {
                    if combat_tracker.remove_combatant(name) {
                        println!("✅ Removed {} from combat", name);
                        combat_tracker.display_initiative_order();
                    } else {
                        println!("❌ Could not find {} in combat", name);
                    }
                } else {
                    println!("Usage: remove <name>");
                }
            }
            "save" => {
                if parts.len() >= 2 {
                    // Check if this is a saving throw or NPC save
                    let potential_ability = parts[1].to_lowercase();
                    if ["str", "dex", "con", "wis", "int", "cha", "strength", "dexterity", "constitution", "wisdom", "intelligence", "charisma"].contains(&potential_ability.as_str()) {
                        // This is a saving throw command
                        let ability = parts[1];
                        let target_name = if parts.len() >= 3 {
                            parts[2].to_string()
                        } else {
                            // Default to current player
                            if let Some(current) = combat_tracker.combatants.get(combat_tracker.current_turn) {
                                current.name.clone()
                            } else {
                                println!("❌ No current combatant for saving throw");
                                continue;
                            }
                        };
                        
                        let actual_target = if target_name.to_lowercase() == "self" {
                            if let Some(current) = combat_tracker.combatants.get(combat_tracker.current_turn) {
                                current.name.clone()
                            } else {
                                println!("❌ Cannot determine current combatant for 'self'");
                                continue;
                            }
                        } else {
                            target_name
                        };
                        
                        match combat_tracker.make_saving_throw(&actual_target, ability) {
                            Ok(result) => println!("{}", result),
                            Err(e) => println!("❌ {}", e),
                        }
                    } else {
                        // This is an NPC save command
                        let npc_name = parts[1];
                        if let Err(e) = combat_tracker.save_npc(npc_name) {
                            println!("❌ Failed to save NPC: {}", e);
                        }
                    }
                } else {
                    println!("Usage: save [ability] [self|name] for saving throws, or save <npc_name> for NPC saving");
                    println!("Examples: save wis Gandalf, save dex self, save Orc");
                }
            }
            "show" | "list" => {
                combat_tracker.display_initiative_order();
            }
            "quit" | "q" => {
                println!("💀 Exiting combat mode...");
                combat_tracker.save_characters_on_exit();
                break;
            }
            "help" | "h" => {
                println!("Combat Mode Commands:");
                println!("  stats [name] - Show character stats");
                println!("  attack <target> - Roll d20 attack vs target's AC");
                println!("  status [add|remove|list] [self|name] <status> - Manage status effects");
                println!("  search <query> - Search D&D 5e API (returns to combat after)");
                println!("  save [ability] [self|name] - Make saving throw (e.g., save wis Gandalf)");
                println!("  save <npc_name> - Save NPC stats to npcs/ directory");
                println!("  next|continue - Advance to next combatant");
                println!("  back - Go back to previous combatant's turn");
                println!("  insert <name> - Add new combatant mid-fight");
                println!("  remove <name> - Remove combatant from combat loop");
                println!("  show|list - Display current initiative order");
                println!("  quit - Exit combat mode (auto-saves player characters)");
            }
            _ => {
                println!("❌ Unknown command '{}'. Type 'help' for available commands.", 
                         parts.get(0).unwrap_or(&""));
            }
        }
    }
}

fn handle_attack_command(combat_tracker: &mut CombatTracker, target_name: &str) {
    if let Some(target) = combat_tracker.get_combatant(target_name) {
        let target_ac = target.ac;
        
        // Roll d20 for attack with critical announcements
        match dice::roll_dice_with_crits("1d20") {
            Ok((rolls, total, crit_message)) => {
                let attack_roll = rolls[0] as i32;
                let hit = attack_roll >= target_ac;
                
                println!("\n⚔️  Attack Roll: {} (d20: {})", total, attack_roll);
                
                // Display critical message if applicable
                if let Some(message) = crit_message {
                    println!("{}", message);
                }
                
                println!("🎯 Target AC: {}", target_ac);
                
                if hit {
                    println!("💥 HIT! The attack connects!");
                    println!("🎲 Enter damage amount (or type 'roll' to use dice mode):");
                    
                    let mut damage_input = String::new();
                    if std::io::stdin().read_line(&mut damage_input).is_ok() {
                        let damage_input = damage_input.trim();
                        
                        if damage_input.to_lowercase() == "roll" {
                            println!("💡 Use the dice mode in another terminal or enter damage manually.");
                            println!("Enter damage amount:");
                            let mut manual_damage = String::new();
                            if std::io::stdin().read_line(&mut manual_damage).is_ok() {
                                if let Ok(damage) = manual_damage.trim().parse::<i32>() {
                                    match combat_tracker.apply_damage(target_name, damage) {
                                        Ok(result) => println!("{}", result),
                                        Err(e) => println!("❌ {}", e),
                                    }
                                } else {
                                    println!("❌ Invalid damage amount");
                                }
                            }
                        } else if let Ok(damage) = damage_input.parse::<i32>() {
                            match combat_tracker.apply_damage(target_name, damage) {
                                Ok(result) => println!("{}", result),
                                Err(e) => println!("❌ {}", e),
                            }
                        } else {
                            println!("❌ Invalid damage amount");
                        }
                    }
                } else {
                    println!("🛡️  MISS! The attack fails to connect.");
                }
            }
            Err(e) => println!("❌ Error rolling attack: {}", e),
        }
    } else {
        println!("❌ Target '{}' not found in combat", target_name);
    }
}

fn handle_status_command(combat_tracker: &mut CombatTracker, args: &[&str]) {
    if args.is_empty() {
        println!("Usage: status [add|remove|list] [self|name] <status_name>");
        return;
    }
    
    let action = args[0].to_lowercase();
    
    // Handle status list command
    if action == "list" {
        if args.len() >= 2 {
            let target = args[1];
            let target_name = if target.to_lowercase() == "self" {
                if let Some(current) = combat_tracker.combatants.get(combat_tracker.current_turn) {
                    current.name.clone()
                } else {
                    println!("❌ Cannot determine current combatant for 'self'");
                    return;
                }
            } else {
                target.to_string()
            };
            
            if let Some(combatant) = combat_tracker.get_combatant(&target_name) {
                if combatant.status_effects.is_empty() {
                    println!("📋 {} has no status effects", target_name);
                } else {
                    println!("📋 Status effects for {}:", target_name);
                    for status in &combatant.status_effects {
                        let duration_str = match status.duration {
                            Some(d) => format!(" ({} rounds remaining)", d),
                            None => " (permanent)".to_string(),
                        };
                        println!("  • {}{}", status.name, duration_str);
                    }
                }
            } else {
                println!("❌ Combatant '{}' not found", target_name);
            }
        } else {
            // List status effects for all combatants
            println!("📋 Status Effects Summary:");
            for combatant in &combat_tracker.combatants {
                if !combatant.status_effects.is_empty() {
                    println!("  {}: {}", combatant.name, 
                        combatant.status_effects.iter()
                            .map(|s| s.name.as_str()).collect::<Vec<_>>().join(", "));
                }
            }
        }
        return;
    }
    
    if args.len() < 3 {
        println!("Usage: status [add|remove] [self|name] <status_name>");
        return;
    }
    
    let target = args[1];
    let status_name = args[2..].join(" ");
    
    // For now, we'll determine "self" based on current turn
    let target_name = if target.to_lowercase() == "self" {
        // Get current combatant name
        if let Some(current) = combat_tracker.combatants.get(combat_tracker.current_turn) {
            current.name.clone()
        } else {
            println!("❌ Cannot determine current combatant for 'self'");
            return;
        }
    } else {
        target.to_string()
    };
    
    match action.as_str() {
        "add" => {
            if let Some(combatant) = combat_tracker.get_combatant_mut(&target_name) {
                let status = StatusEffect {
                    name: status_name.clone(),
                    description: None,
                    duration: None, // Could be enhanced to ask for duration
                };
                combatant.add_status(status);
                println!("✅ Added status '{}' to {}", status_name, target_name);
            } else {
                println!("❌ Combatant '{}' not found", target_name);
            }
        }
        "remove" => {
            if let Some(combatant) = combat_tracker.get_combatant_mut(&target_name) {
                if combatant.remove_status(&status_name) {
                    println!("✅ Removed status '{}' from {}", status_name, target_name);
                } else {
                    println!("❌ Status '{}' not found on {}", status_name, target_name);
                }
            } else {
                println!("❌ Combatant '{}' not found", target_name);
            }
        }
        _ => {
            println!("❌ Invalid action '{}'. Use 'add', 'remove', or 'list'", action);
        }
    }
}

fn handle_insert_combatant(combat_tracker: &mut CombatTracker, name: &str) {
    println!("\n➕ Inserting new combatant: {}", name);
    
    // Check if character already exists in saved characters
    let existing_characters = load_character_files();
    if let Some(character) = existing_characters.iter().find(|c| c.name.eq_ignore_ascii_case(name)) {
        println!("📝 Found existing character: {}", character.name);
        
        // Get initiative
        let dex_mod = character.get_dexterity_modifier();
        let dex_mod_str = if dex_mod >= 0 { format!("+{}", dex_mod) } else { dex_mod.to_string() };
        
        println!("Initiative for {} (DEX modifier: {}): ", character.name, dex_mod_str);
        let mut init_input = String::new();
        if io::stdin().read_line(&mut init_input).is_ok() {
            let input = init_input.trim();
            
            if input.is_empty() {
                // Auto-roll initiative
                match dice::roll_dice_with_crits("1d20") {
                    Ok((rolls, base_roll, crit_message)) => {
                        let initiative = base_roll as i32 + dex_mod as i32;
                        let mut message = format!("🎲 Rolled {} (d20: {}, DEX: {}) = {}", 
                                initiative, rolls[0], dex_mod_str, initiative);
                        
                        if let Some(crit) = crit_message {
                            message.push_str(&format!("\n{}", crit));
                        }
                        println!("{}", message);
                        
                        let combatant = Combatant::from_character(character.clone(), initiative);
                        combat_tracker.add_combatant(combatant);
                        println!("✅ Added {} to combat with initiative {}", character.name, initiative);
                    }
                    Err(e) => println!("❌ Error rolling initiative: {}", e),
                }
            } else if let Ok(initiative) = input.parse::<i32>() {
                let combatant = Combatant::from_character(character.clone(), initiative);
                combat_tracker.add_combatant(combatant);
                println!("✅ Added {} to combat with initiative {}", character.name, initiative);
            } else {
                println!("❌ Invalid initiative value");
            }
        }
    } else {
        // Create new NPC
        println!("📝 Creating new NPC: {}", name);
        
        print!("HP: ");
        io::stdout().flush().unwrap();
        let mut hp_input = String::new();
        io::stdin().read_line(&mut hp_input).expect("Failed to read HP");
        let hp = hp_input.trim().parse::<i32>().unwrap_or(10);
        
        print!("AC: ");
        io::stdout().flush().unwrap();
        let mut ac_input = String::new();
        io::stdin().read_line(&mut ac_input).expect("Failed to read AC");
        let ac = ac_input.trim().parse::<i32>().unwrap_or(10);
        
        print!("Initiative: ");
        io::stdout().flush().unwrap();
        let mut init_input = String::new();
        io::stdin().read_line(&mut init_input).expect("Failed to read initiative");
        let initiative = init_input.trim().parse::<i32>().unwrap_or(0);
        
        let combatant = Combatant::new_npc(name.to_string(), hp, ac, initiative);
        combat_tracker.add_combatant(combatant);
        println!("✅ Added {} to combat as NPC!", name);
    }
    
    combat_tracker.display_initiative_order();
}

pub fn search_mode() {
    println!("\n🔍 D&D 5e Wikidot Search Tool 🔍");
    println!("═══════════════════════════════════════════════════════════");
    println!("Search for spells, classes, equipment, monsters, and races");
    println!("Powered by http://dnd5e.wikidot.com - Live data from the web!");
    println!("═══════════════════════════════════════════════════════════");
    
    // Create runtime for async operations
    let rt = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(e) => {
            println!("❌ Failed to create async runtime: {}", e);
            println!("Search functionality unavailable.");
            return;
        }
    };
    
    let client = DndSearchClient::new();
    
    println!("🌐 Online mode - connecting to Wikidot D&D 5e site");
    
    // Test network connectivity
    println!("🔄 Testing API connectivity...");
    rt.block_on(async {
        test_api_connectivity(&client).await;
    });
    
    loop {
        println!("\n--- Search Menu ---");
        println!("Commands:");
        println!("  search <query> - Search all categories");
        println!("  search <category> <query> - Search specific category");
        println!("  categories - List available categories");
        println!("  help - Show detailed help");
        println!("  back - Return to tools menu");
        println!("  EXIT - Quit program immediately");
        println!();
        print!("Search > ");
        io::stdout().flush().unwrap_or(());
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Failed to read input");
            continue;
        }
        
        let input = input.trim();
        
        // Check for universal exit command
        check_universal_exit(input);
        
        if input.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0].to_lowercase();
        
        match command.as_str() {
            "search" => {
                if parts.len() < 2 {
                    println!("Usage: search <query> or search <category> <query>");
                    continue;
                }
                
                let (category, query) = if parts.len() == 2 {
                    // search <query>
                    (None, parts[1].to_string())
                } else {
                    // search <category> <query>
                    let potential_category = SearchCategory::from_str(parts[1]);
                    if potential_category.is_some() {
                        (potential_category, parts[2..].join(" "))
                    } else {
                        // Treat first argument as part of query
                        (None, parts[1..].join(" "))
                    }
                };
                
                rt.block_on(async {
                    handle_search_command(&client, &query, category).await;
                });
            },
            "categories" => {
                println!("\nAvailable Categories:");
                println!("  • spells - Magic spells");
                println!("  • classes - Character classes");
                println!("  • equipment (or items/gear) - Weapons, armor, and gear");
                println!("  • monsters (or creatures) - Monsters and NPCs");
                println!("  • races - Character races");
                println!("\nExample usage:");
                println!("  search fireball");
                println!("  search spell fireball");
                println!("  search equipment longsword");
            },
            "help" => {
                show_search_help();
            },
            "back" => {
                println!("Returning to tools menu...");
                break;
            },
            _ => {
                // Try to interpret the entire input as a search query
                rt.block_on(async {
                    handle_search_command(&client, input, None).await;
                });
            }
        }
    }
}

async fn handle_search_command(client: &DndSearchClient, query: &str, category: Option<SearchCategory>) {
    println!("🔍 Searching for '{}'...", query);
    
    match client.search(query, category).await {
        Ok(results) => {
            if results.is_empty() {
                // No exact match found, get suggestions
                println!("❌ No exact match found for '{}'", query);
                
                let suggestions = client.get_suggestions(query, category).await;
                
                if suggestions.is_empty() {
                    println!("🔍 No similar items found either.");
                    if let Some(_cat) = category {
                        println!("💡 Try searching in a different category or check your spelling");
                    } else {
                        println!("💡 Try specifying a category: search <category> <query>");
                        println!("   Example: search spell {}", query);
                    }
                } else {
                    println!("🔍 Here are some suggestions that might be what you're looking for:");
                    println!("   (These are the closest matches found)");
                    println!();
                    
                    for (i, suggestion) in suggestions.iter().enumerate() {
                        println!("  {}. {} 📝", i + 1, suggestion);
                    }
                    
                    println!();
                    println!("💡 Would you like to search for one of these suggestions?");
                    println!("   Enter the number of your choice (1-{}), or press Enter to skip:", suggestions.len());
                    print!("Choice > ");
                    io::stdout().flush().unwrap_or(());
                    
                    let mut choice_input = String::new();
                    if io::stdin().read_line(&mut choice_input).is_ok() {
                        let choice_input = choice_input.trim();
                        
                        // Check for universal exit command
                        check_universal_exit(choice_input);
                        
                        if !choice_input.is_empty() {
                            if let Ok(choice) = choice_input.parse::<usize>() {
                                if choice > 0 && choice <= suggestions.len() {
                                    let selected = &suggestions[choice - 1];
                                    println!("🔍 Searching for '{}'...", selected);
                                    
                                    // Search again with the selected suggestion
                                    match client.search(selected, category).await {
                                        Ok(suggestion_results) => {
                                            if suggestion_results.is_empty() {
                                                println!("❌ No detailed results found for '{}'", selected);
                                            } else {
                                                display_search_results(&suggestion_results);
                                            }
                                        },
                                        Err(e) => {
                                            println!("❌ Error searching for suggestion: {}", e);
                                        }
                                    }
                                } else {
                                    println!("❌ Invalid choice. Please select a number between 1 and {}", suggestions.len());
                                }
                            } else {
                                println!("❌ Invalid input. Please enter a number or press Enter to skip.");
                            }
                        } else {
                            println!("👍 Skipping suggestions.");
                        }
                    }
                }
            } else {
                display_search_results(&results);
            }
        },
        Err(e) => {
            println!("❌ Search failed: {}", e);
            println!("💡 This might be due to network issues. The search will fall back to cached data.");
            
            // Still try to show suggestions even if the main search failed
            println!("🔍 Checking for suggestions in cached data...");
            let suggestions = client.get_suggestions(query, category).await;
            
            if !suggestions.is_empty() {
                println!("📝 Found these similar items in cached data:");
                for (i, suggestion) in suggestions.iter().enumerate() {
                    println!("  {}. {}", i + 1, suggestion);
                }
                
                println!("\nTry searching for one of these when the network is available.");
            }
        }
    }
}

fn display_search_results(results: &[SearchResult]) {
    println!("✅ Found {} result(s):", results.len());
    
    for (i, result) in results.iter().enumerate() {
        if results.len() > 1 {
            println!("\n--- Result {} ---", i + 1);
        }
        result.display();
    }
    
    if results.len() > 1 {
        println!("\n📋 Summary:");
        for (i, result) in results.iter().enumerate() {
            println!("  {}. {} ({})", i + 1, result.name(), result.index());
        }
    }
    
    println!("\nPress Enter to continue...");
    let mut _buffer = String::new();
    let _ = io::stdin().read_line(&mut _buffer);
}

fn show_search_help() {
    println!("\n📖 D&D 5e Wikidot Search Help 📖");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("BASIC USAGE:");
    println!("  search <query>              - Search all categories");
    println!("  search <category> <query>   - Search specific category");
    println!();
    println!("CATEGORIES:");
    println!("  spells      - Magic spells (e.g., fireball, cure wounds)");
    println!("  classes     - Character classes (e.g., fighter, wizard)");
    println!("  equipment   - Items, weapons, armor (e.g., longsword, leather armor)");
    println!("  monsters    - Creatures and NPCs (e.g., goblin, dragon)");
    println!("  races       - Character races (e.g., elf, dwarf)");
    println!();
    println!("EXAMPLES:");
    println!("  search fireball");
    println!("  search spell magic missile");
    println!("  search class paladin");
    println!("  search equipment chain mail");
    println!("  search monster troll");
    println!("  search race halfling");
    println!();
    println!("FEATURES:");
    println!("  • Live data fetching from dnd5e.wikidot.com");
    println!("  • Complete page content displayed with nice formatting");
    println!("  • Smart query variations for better match finding");
    println!("  • Case-insensitive search with flexible input parsing");
    println!("  • Universal EXIT command works from any prompt");
    println!();
    println!("INPUT REQUIREMENTS:");
    println!("  • All commands must end with pressing Enter (newline)");
    println!("  • This prevents the program from hanging on input prompts");
    println!();
    println!("NETWORK:");
    println!("  The tool fetches live data from the D&D 5e Wikidot community site.");
    println!("  Internet connection is required for search functionality.");
    println!("  All content is sourced from the community-maintained wiki.");
    println!();
    println!("═══════════════════════════════════════════════════════════");
}

async fn test_api_connectivity(client: &DndSearchClient) {
    // Test basic connectivity to Wikidot
    let test_url = "http://dnd5e.wikidot.com/spell:fireball";
    
    match reqwest::Client::new()
        .get(test_url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await 
    {
        Ok(response) => {
            if response.status().is_success() {
                println!("✅ Wikidot connectivity test successful! Online features available.");
            } else {
                println!("⚠️ Wikidot responded but with status: {} - limited online functionality", response.status());
            }
        },
        Err(e) => {
            println!("❌ Wikidot connectivity test failed: {}", e);
            
            if e.is_timeout() {
                println!("💡 Timeout error - the site might be slow or unreachable");
            } else if e.is_connect() {
                println!("💡 Connection error - check network connectivity");
            } else if e.is_request() {
                println!("💡 Request error - there might be an issue with the request format");
            }
        }
    }
}

fn handle_search_in_combat(query: &str) {
    println!("\n🔍 Searching for '{}' in D&D 5e database...", query);
    
    // Create runtime for async operations
    let rt = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(e) => {
            println!("❌ Failed to create async runtime: {}", e);
            println!("Search functionality unavailable.");
            return;
        }
    };
    
    let client = DndSearchClient::new();
    
    rt.block_on(async {
        match client.search(query, None).await {
            Ok(results) => {
                if results.is_empty() {
                    println!("❌ No exact match found for '{}'", query);
                    
                    let suggestions = client.get_suggestions(query, None).await;
                    if !suggestions.is_empty() {
                        println!("🔍 Similar items found:");
                        for (i, suggestion) in suggestions.iter().take(3).enumerate() {
                            println!("  {}. {}", i + 1, suggestion);
                        }
                    }
                } else {
                    display_search_results(&results);
                }
            },
            Err(e) => {
                println!("❌ Search failed: {}", e);
            }
        }
    });
    
    println!("\n📋 Returning to combat...");
    println!("Press Enter to continue combat...");
    let mut _buffer = String::new();
    let _ = io::stdin().read_line(&mut _buffer);
}
