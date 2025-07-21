use crate::character::Character;
use crate::file_manager::load_character_files;
use serde::{Deserialize, Serialize};
use std::{fs, io::{self, Write}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEffect {
    pub name: String,
    pub description: Option<String>,
    pub duration: Option<i32>, // rounds remaining, None for permanent
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Combatant {
    pub name: String,
    pub character_data: Option<Character>, // None for quick NPCs
    pub current_hp: i32,
    pub max_hp: i32,
    pub temp_hp: i32,
    pub ac: i32,
    pub initiative: i32,
    pub is_player: bool,
    pub status_effects: Vec<StatusEffect>,
}

impl Combatant {
    pub fn from_character(character: Character, initiative: i32) -> Self {
        let current_hp = character.hp.unwrap_or(10) as i32;
        let max_hp = character.max_hp.unwrap_or(current_hp as u8) as i32;
        let temp_hp = character.temp_hp.unwrap_or(0) as i32;
        let ac = character.ac.unwrap_or(10) as i32;

        Combatant {
            name: character.name.clone(),
            character_data: Some(character),
            current_hp,
            max_hp,
            temp_hp,
            ac,
            initiative,
            is_player: true,
            status_effects: Vec::new(),
        }
    }

    pub fn new_npc(name: String, hp: i32, ac: i32, initiative: i32) -> Self {
        Combatant {
            name,
            character_data: None,
            current_hp: hp,
            max_hp: hp,
            temp_hp: 0,
            ac,
            initiative,
            is_player: false,
            status_effects: Vec::new(),
        }
    }

    pub fn add_status(&mut self, status: StatusEffect) {
        // Remove existing status with same name
        self.status_effects.retain(|s| s.name != status.name);
        self.status_effects.push(status);
    }

    pub fn remove_status(&mut self, status_name: &str) -> bool {
        let original_len = self.status_effects.len();
        self.status_effects.retain(|s| s.name != status_name);
        self.status_effects.len() != original_len
    }

    pub fn display_stats(&self) {
        // Enhanced 3-column display
        println!("\n╔═══════════════════════════════════════════════════════════════╗");
        println!("║                    {} ({})", 
                 format!("{:<25}", self.name), 
                 if self.is_player { "Player" } else { "NPC" });
        println!("╠═══════════════════════════════════════════════════════════════╣");
        
        // Combat Stats Column
        println!("║ Combat Stats          │ Health Stats          │ Other Stats        ║");
        println!("║ AC: {:<17} │ HP: {}/{:<13} │                    ║", 
                 self.ac, self.current_hp, self.max_hp);
        println!("║ Initiative: {:<10} │ Temp HP: {:<12} │                    ║", 
                 self.initiative, self.temp_hp);

        // If we have character data, show more stats
        if let Some(character) = &self.character_data {
            println!("║                       │                       │                    ║");
            println!("║ Ability Scores        │ Saves & Skills        │ Other              ║");
            
            // Display ability scores with modifiers in proper order
            let str_score = character.stre.unwrap_or(10);
            let str_mod = character.get_strength_modifier();
            let dex_score = character.dext.unwrap_or(10);
            let dex_mod = character.get_dexterity_modifier();
            let con_score = character.cons.unwrap_or(10);
            let con_mod = character.get_constitution_modifier();
            let wis_score = character.wisd.unwrap_or(10);
            let wis_mod = character.get_wisdom_modifier();
            let int_score = character.intl.unwrap_or(10);
            let int_mod = character.get_intelligence_modifier();
            let cha_score = character.chas.unwrap_or(10);
            let cha_mod = character.get_charisma_modifier();

            println!("║ STR: {} ({:+2})       │ Prof Bonus: {:<10} │ Level: {:<12} ║", 
                     str_score, str_mod,
                     character.prof_bonus.unwrap_or(2),
                     character.level.unwrap_or(1));
            println!("║ DEX: {} ({:+2})       │ Passive Perc: {:<8} │ Speed: {:<12} ║", 
                     dex_score, dex_mod,
                     character.passive_perception.unwrap_or_else(|| character.calculate_passive_perception()),
                     character.speed.unwrap_or(30));
            println!("║ CON: {} ({:+2})       │                       │                    ║", 
                     con_score, con_mod);
            println!("║ WIS: {} ({:+2})       │                       │                    ║", 
                     wis_score, wis_mod);
            println!("║ INT: {} ({:+2})       │                       │                    ║", 
                     int_score, int_mod);
            println!("║ CHA: {} ({:+2})       │                       │                    ║", 
                     cha_score, cha_mod);
        }

        // Status effects
        if !self.status_effects.is_empty() {
            println!("║                       │                       │                    ║");
            println!("║ Status Effects:                                                   ║");
            for status in &self.status_effects {
                let duration_str = match status.duration {
                    Some(d) => format!("({} rounds)", d),
                    None => "(permanent)".to_string(),
                };
                println!("║ • {:<20} {:<35} ║", status.name, duration_str);
            }
        }

        println!("╚═══════════════════════════════════════════════════════════════╝");
    }
}

#[derive(Debug)]
pub struct CombatTracker {
    pub combatants: Vec<Combatant>,
    pub current_turn: usize,
    pub round_number: i32,
}

impl CombatTracker {
    pub fn new() -> Self {
        CombatTracker {
            combatants: Vec::new(),
            current_turn: 0,
            round_number: 1,
        }
    }

    pub fn add_combatant(&mut self, combatant: Combatant) {
        self.combatants.push(combatant);
        self.sort_by_initiative();
    }

    fn sort_by_initiative(&mut self) {
        self.combatants.sort_by(|a, b| b.initiative.cmp(&a.initiative));
        self.current_turn = 0;
    }

    pub fn next_turn(&mut self) -> Option<&mut Combatant> {
        if self.combatants.is_empty() {
            return None;
        }

        // Skip combatants with initiative 0
        let mut attempts = 0;
        while attempts < self.combatants.len() {
            let combatant = &self.combatants[self.current_turn];
            
            if combatant.initiative > 0 {
                let current_turn = self.current_turn;
                self.current_turn = (self.current_turn + 1) % self.combatants.len();
                
                // If we've looped back to the beginning, increment round
                if self.current_turn == 0 {
                    self.round_number += 1;
                    println!("\n🔄 Starting Round {}", self.round_number);
                }
                
                return Some(&mut self.combatants[current_turn]);
            }
            
            self.current_turn = (self.current_turn + 1) % self.combatants.len();
            attempts += 1;
            
            // Check if we've completed a round
            if self.current_turn == 0 {
                self.round_number += 1;
                println!("\n🔄 Starting Round {}", self.round_number);
            }
        }

        None
    }

    pub fn get_combatant_mut(&mut self, name: &str) -> Option<&mut Combatant> {
        self.combatants.iter_mut().find(|c| c.name.eq_ignore_ascii_case(name))
    }

    pub fn get_combatant(&self, name: &str) -> Option<&Combatant> {
        self.combatants.iter().find(|c| c.name.eq_ignore_ascii_case(name))
    }

    pub fn remove_combatant(&mut self, name: &str) -> bool {
        if let Some(pos) = self.combatants.iter().position(|c| c.name.eq_ignore_ascii_case(name)) {
            self.combatants.remove(pos);
            if self.current_turn >= self.combatants.len() && !self.combatants.is_empty() {
                self.current_turn = 0;
            }
            true
        } else {
            false
        }
    }

    pub fn apply_damage(&mut self, target_name: &str, damage: i32) -> Result<String, String> {
        if let Some(target) = self.get_combatant_mut(target_name) {
            // Apply damage to temp HP first, then regular HP
            if target.temp_hp > 0 {
                if damage <= target.temp_hp {
                    target.temp_hp -= damage;
                    return Ok(format!("💛 {} takes {} damage to temporary HP (Temp HP: {}/{})", 
                             target_name, damage, target.temp_hp, target.current_hp));
                } else {
                    let temp_damage = target.temp_hp;
                    let remaining_damage = damage - temp_damage;
                    target.temp_hp = 0;
                    target.current_hp = (target.current_hp - remaining_damage).max(0);
                    return Ok(format!("💛❤️ {} takes {} damage ({} to temp HP, {} to HP). HP: {}/{}, Temp: 0", 
                             target_name, damage, temp_damage, remaining_damage, 
                             target.current_hp, target.max_hp));
                }
            } else {
                target.current_hp = (target.current_hp - damage).max(0);
                let status = if target.current_hp == 0 {
                    "💀 DOWN!"
                } else if target.current_hp <= target.max_hp / 4 {
                    "🩸 Bloodied"
                } else {
                    ""
                };
                
                return Ok(format!("❤️ {} takes {} damage. HP: {}/{} {}", 
                         target_name, damage, target.current_hp, target.max_hp, status));
            }
        } else {
            Err(format!("Target '{}' not found in combat", target_name))
        }
    }

    pub fn make_saving_throw(&self, combatant_name: &str, ability: &str) -> Result<String, String> {
        use crate::character::AbilityScore;
        use crate::dice::roll_dice_with_crits;

        if let Some(combatant) = self.get_combatant(combatant_name) {
            let ability_type = match ability.to_lowercase().as_str() {
                "str" | "strength" => AbilityScore::Strength,
                "dex" | "dexterity" => AbilityScore::Dexterity,
                "con" | "constitution" => AbilityScore::Constitution,
                "wis" | "wisdom" => AbilityScore::Wisdom,
                "int" | "intelligence" => AbilityScore::Intelligence,
                "cha" | "charisma" => AbilityScore::Charisma,
                _ => return Err(format!("Invalid ability score: {}. Use str, dex, con, wis, int, or cha", ability)),
            };

            let modifier = if let Some(character_data) = &combatant.character_data {
                character_data.get_ability_modifier(ability_type)
            } else {
                // For NPCs without character data, assume average stats (10-11, modifier 0)
                0
            };

            match roll_dice_with_crits("1d20") {
                Ok((rolls, base_roll, crit_message)) => {
                    let total = base_roll as i32 + modifier as i32;
                    let modifier_str = if modifier >= 0 {
                        format!("+{}", modifier)
                    } else {
                        modifier.to_string()
                    };

                    let mut result = format!("🎲 {} makes a {} saving throw: {} (d20: {}, modifier: {}) = {}", 
                              combatant_name, ability_type.name(), total, rolls[0], modifier_str, total);
                    
                    if let Some(message) = crit_message {
                        result.push_str(&format!("\n{}", message));
                    }
                    
                    Ok(result)
                }
                Err(e) => Err(format!("Error rolling d20: {}", e)),
            }
        } else {
            Err(format!("Combatant '{}' not found in combat", combatant_name))
        }
    }

    pub fn save_characters_on_exit(&self) {
        use crate::file_manager::save_character;
        
        println!("💾 Auto-saving player characters...");
        let mut saved_count = 0;
        
        for combatant in &self.combatants {
            if combatant.is_player {
                if let Some(character_data) = &combatant.character_data {
                    // Update character HP from combat
                    let mut updated_character = character_data.clone();
                    updated_character.hp = Some(combatant.current_hp as u8);
                    updated_character.temp_hp = Some(combatant.temp_hp as u8);
                    
                    save_character(updated_character.name.clone(), updated_character);
                    saved_count += 1;
                }
            }
        }
        
        if saved_count > 0 {
            println!("✅ Saved {} player character(s)", saved_count);
        }
    }

    pub fn display_initiative_order(&self) {
        println!("\n📋 Initiative Order (Round {}):", self.round_number);
        println!("═══════════════════════════════════════════════════════════");
        
        for (i, combatant) in self.combatants.iter().enumerate() {
            let marker = if i == self.current_turn { ">>> " } else { "    " };
            let hp_display = format!("{}/{}", combatant.current_hp, combatant.max_hp);
            let status_info = if combatant.status_effects.is_empty() {
                String::new()
            } else {
                format!(" [{}]", combatant.status_effects.iter()
                    .map(|s| s.name.as_str()).collect::<Vec<_>>().join(", "))
            };
            
            let type_marker = if combatant.is_player { "🧙" } else { "👹" };
            
            println!("{}{}Init {}: {} {} (AC: {}, HP: {}){}", 
                marker, type_marker, combatant.initiative, combatant.name,
                if combatant.initiative == 0 { "(SKIPPED)" } else { "" },
                combatant.ac, hp_display, status_info);
        }
        println!("═══════════════════════════════════════════════════════════");
    }

    pub fn save_npc(&self, npc_name: &str) -> io::Result<()> {
        // Create npcs directory if it doesn't exist
        fs::create_dir_all("npcs")?;
        
        if let Some(combatant) = self.get_combatant(npc_name) {
            let path = format!("npcs/{}.txt", npc_name);
            let mut file = fs::File::create(path)?;
            
            writeln!(file, "Name: {}", combatant.name)?;
            writeln!(file, "HP: {}/{}", combatant.current_hp, combatant.max_hp)?;
            writeln!(file, "AC: {}", combatant.ac)?;
            writeln!(file, "Initiative: {}", combatant.initiative)?;
            writeln!(file, "Type: {}", if combatant.is_player { "Player" } else { "NPC" })?;
            
            if !combatant.status_effects.is_empty() {
                writeln!(file, "Status Effects:")?;
                for status in &combatant.status_effects {
                    let duration = match status.duration {
                        Some(d) => format!(" ({} rounds)", d),
                        None => String::new(),
                    };
                    writeln!(file, "  - {}{}", status.name, duration)?;
                }
            }
            
            println!("💾 Saved NPC '{}' to npcs/{}.txt", npc_name, npc_name);
        }
        
        Ok(())
    }
    
    pub fn previous_turn(&mut self) -> bool {
        if self.combatants.is_empty() {
            return false;
        }
        
        if self.current_turn == 0 {
            self.current_turn = self.combatants.len() - 1;
            if self.round_number > 1 {
                self.round_number -= 1;
                println!("🔄 Going back to Round {}", self.round_number);
            }
        } else {
            self.current_turn -= 1;
        }
        
        true
    }
    
    pub fn get_current_combatant(&mut self) -> Option<&mut Combatant> {
        if self.current_turn < self.combatants.len() {
            Some(&mut self.combatants[self.current_turn])
        } else {
            None
        }
    }
}

pub fn enhanced_initiative_setup() -> CombatTracker {
    let mut tracker = CombatTracker::new();
    let existing_characters = load_character_files();
    
    println!("\n⚔️  Setting up Initiative Tracker ⚔️");
    println!("═══════════════════════════════════════");
    
    // Ask existing players for their initiative first
    if !existing_characters.is_empty() {
        println!("\n📝 Found existing player characters:");
        for (i, character) in existing_characters.iter().enumerate() {
            println!("{}. {}", i + 1, character.name);
        }
        
        println!("\n🎲 Please enter initiative for each player (or press Enter to auto-roll d20+DEX):");
        for mut character in existing_characters {
            // Ensure character has complete stats before using in combat
            character.ensure_complete_stats();
            
            loop {
                let dex_mod = character.get_dexterity_modifier();
                let dex_mod_str = if dex_mod >= 0 { format!("+{}", dex_mod) } else { dex_mod.to_string() };
                
                println!("Initiative for {} (DEX modifier: {}): ", 
                         character.name, dex_mod_str);
                
                let mut buffer = String::new();
                if io::stdin().read_line(&mut buffer).is_ok() {
                    let input = buffer.trim();
                    
                    if input.is_empty() {
                        // Auto-roll initiative: d20 + DEX modifier
                        match crate::dice::roll_dice_with_crits("1d20") {
                            Ok((rolls, base_roll, crit_message)) => {
                                let initiative = base_roll as i32 + dex_mod as i32;
                                let mut message = format!("🎲 Rolled {} (d20: {}, DEX: {}) = {}", 
                                        initiative, rolls[0], dex_mod_str, initiative);
                                
                                if let Some(crit) = crit_message {
                                    message.push_str(&format!("\n{}", crit));
                                }
                                println!("{}", message);
                                
                                let combatant = Combatant::from_character(character.clone(), initiative);
                                tracker.add_combatant(combatant);
                                println!("✅ Added {} with initiative {}", character.name, initiative);
                                break;
                            }
                            Err(e) => {
                                println!("❌ Error rolling initiative: {}", e);
                                continue;
                            }
                        }
                    } else if let Ok(initiative) = input.parse::<i32>() {
                        if initiative > 0 {
                            let combatant = Combatant::from_character(character.clone(), initiative);
                            tracker.add_combatant(combatant);
                            println!("✅ Added {} with initiative {}", character.name, initiative);
                        } else {
                            println!("⏭️  Skipping {} (initiative 0)", character.name);
                        }
                        break;
                    } else {
                        println!("❌ Invalid input. Please enter a number or press Enter to auto-roll.");
                    }
                } else {
                    println!("❌ Failed to read input. Please try again.");
                }
            }
        }
    }
    
    // Add additional combatants (NPCs, etc.)
    loop {
        println!("\n➕ Add more combatants? (y/n)");
        let mut buffer = String::new();
        if io::stdin().read_line(&mut buffer).is_ok() {
            match buffer.trim().to_lowercase().as_str() {
                "y" | "yes" => {
                    add_manual_combatant(&mut tracker);
                }
                "n" | "no" => break,
                _ => println!("Please enter 'y' or 'n'"),
            }
        }
    }
    
    tracker
}

fn add_manual_combatant(tracker: &mut CombatTracker) {
    println!("\n📝 Adding new combatant:");
    
    print!("Name: ");
    io::stdout().flush().unwrap();
    let mut name = String::new();
    io::stdin().read_line(&mut name).expect("Failed to read name");
    let name = name.trim().to_string();
    
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
    
    let combatant = Combatant::new_npc(name.clone(), hp, ac, initiative);
    tracker.add_combatant(combatant);
    
    println!("✅ Added {} to combat tracker!", name);
}