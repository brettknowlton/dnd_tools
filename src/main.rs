use std::io;

mod character;
mod file_manager;
mod initiative;
mod dice;
mod input_handler;
mod events;
mod error_handling;
mod combat;
mod tests;

use character::Character;
use file_manager::{load_character_files, save_characters, display_single_character, display_all_characters, delete_character_menu};
use initiative::initiative_tracker_mode;
use dice::{roll_dice_mode, roll_dice};
use input_handler::create_character;
use events::Data;
use combat::{enhanced_initiative_setup, CombatTracker, StatusEffect};


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
    println!("\n(Note: passive_perception and cards array excluded as requested)");
    
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
    println!("  🎭 status [add|remove] [self|name] <status> - Manage status effects");
    println!("  🎲 save [ability] [self|name] - Make saving throw (e.g., save wis Gandalf)");
    println!("  ➡️  next|continue - Advance to next combatant");
    println!("  🗑️  remove <name> - Remove combatant from combat");
    println!("  💾 save <npc_name> - Save NPC to npcs/ directory");
    println!("  🔍 show - Display current initiative order");
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
                if let Some(next_combatant) = combat_tracker.next_turn() {
                    println!("\n🎯 It's {}'s turn!", next_combatant.name);
                    next_combatant.display_stats();
                } else {
                    println!("❌ No combatants available for turns");
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
            "show" => {
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
                println!("  status [add|remove] [self|name] <status> - Manage status effects");
                println!("  save [ability] [self|name] - Make saving throw (e.g., save wis Gandalf)");
                println!("  save <npc_name> - Save NPC stats to npcs/ directory");
                println!("  next|continue - Advance to next combatant");
                println!("  remove <name> - Remove combatant from combat loop");
                println!("  show - Display current initiative order");
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
        
        // Roll d20 for attack
        match roll_dice("1d20") {
            Ok((rolls, total)) => {
                let attack_roll = rolls[0] as i32;
                let hit = attack_roll >= target_ac;
                
                println!("\n⚔️  Attack Roll: {} (d20: {})", total, attack_roll);
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
    if args.len() < 3 {
        println!("Usage: status [add|remove] [self|name] <status_name>");
        return;
    }
    
    let action = args[0].to_lowercase();
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
            println!("❌ Invalid action '{}'. Use 'add' or 'remove'", action);
        }
    }
}
