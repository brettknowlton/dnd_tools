use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use crate::character::Character;
use rand;

#[derive(Debug, Clone)]
pub enum AppMode {
    MainMenu,
    CharactersMenu,
    ToolsMenu,
    CharacterCreation,
    CharacterCreationTUI,
    CharacterDisplay,
    CharacterDisplayTUI,
    CharacterDeletion,
    CharacterDeletionTUI,
    InitiativeTracker,
    InitiativeTrackerTUI,
    NpcGenerator,
    NpcGeneratorTUI,
    EncounterRandomizer,
    EncounterRandomizerTUI,
    Dice,
    DiceTUI,
    CombatTracker,
    CombatTrackerTUI,
    Search,
    SearchTUI,
    Exit,
}

#[derive(Debug)]
pub struct App {
    pub mode: AppMode,
    pub selected_index: usize,
    pub characters: Vec<Character>,
    pub should_quit: bool,
    pub message: Option<String>,
    // TUI terminal fields
    pub input_buffer: String,
    pub output_history: Vec<String>,
    pub command_history: Vec<String>,
    pub history_index: Option<usize>,
    pub scroll_offset: usize,
    // Combat tracker state
    pub combat_tracker: Option<crate::combat::CombatTracker>,
    // State tracking
    pub current_state: String,
    pub waiting_for: Option<String>,
    // Dice rolling state
    pub dice_results: Vec<String>,
}

impl App {
    pub fn new(characters: Vec<Character>) -> Self {
        Self {
            mode: AppMode::MainMenu,
            selected_index: 0,
            characters,
            should_quit: false,
            message: None,
            input_buffer: String::new(),
            output_history: Vec::new(),
            command_history: Vec::new(),
            history_index: None,
            scroll_offset: 0,
            combat_tracker: None,
            current_state: "Ready".to_string(),
            waiting_for: None,
            dice_results: Vec::new(),
        }
    }

    pub fn get_menu_items(&self) -> Vec<&str> {
        match self.mode {
            AppMode::MainMenu => vec!["Characters", "Tools", "Exit"],
            AppMode::CharactersMenu => vec!["Creation", "Display single character", "Display all characters", "Character deletion", "Back to main menu"],
            AppMode::ToolsMenu => vec!["Initiative tracker", "NPC randomizer", "Encounter randomizer", "Dice", "Combat tracker", "Search D&D 5e API", "Back to main menu"],
            _ => vec![],
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match self.mode {
            AppMode::CombatTrackerTUI | AppMode::SearchTUI | AppMode::CharacterCreationTUI 
            | AppMode::CharacterDisplayTUI | AppMode::CharacterDeletionTUI | AppMode::InitiativeTrackerTUI 
            | AppMode::NpcGeneratorTUI | AppMode::EncounterRandomizerTUI | AppMode::DiceTUI => {
                self.handle_terminal_key(key);
            }
            _ => {
                match key {
                    KeyCode::Up => self.previous_item(),
                    KeyCode::Down => self.next_item(),
                    KeyCode::Enter => self.select_current(),
                    KeyCode::Esc => self.go_back(),
                    // Removed auto-quit on 'q' - now requires Ctrl+Q
                    _ => {}
                }
            }
        }
    }

    fn previous_item(&mut self) {
        let items = self.get_menu_items();
        if !items.is_empty() {
            self.selected_index = (self.selected_index + items.len() - 1) % items.len();
        }
    }

    fn next_item(&mut self) {
        let items = self.get_menu_items();
        if !items.is_empty() {
            self.selected_index = (self.selected_index + 1) % items.len();
        }
    }

    fn select_current(&mut self) {
        match self.mode {
            AppMode::MainMenu => {
                match self.selected_index {
                    0 => {
                        self.mode = AppMode::CharactersMenu;
                        self.selected_index = 0;
                    }
                    1 => {
                        self.mode = AppMode::ToolsMenu;
                        self.selected_index = 0;
                    }
                    2 => self.mode = AppMode::Exit,
                    _ => {}
                }
            }
            AppMode::CharactersMenu => {
                match self.selected_index {
                    0 => self.mode = AppMode::CharacterCreationTUI,
                    1 => self.mode = AppMode::CharacterDisplayTUI,
                    2 => self.mode = AppMode::CharacterDisplayTUI,
                    3 => self.mode = AppMode::CharacterDeletionTUI,
                    4 => {
                        self.mode = AppMode::MainMenu;
                        self.selected_index = 0;
                    }
                    _ => {}
                }
            }
            AppMode::ToolsMenu => {
                match self.selected_index {
                    0 => self.mode = AppMode::InitiativeTrackerTUI,
                    1 => self.mode = AppMode::NpcGeneratorTUI,
                    2 => self.mode = AppMode::EncounterRandomizerTUI,
                    3 => self.mode = AppMode::DiceTUI,
                    4 => self.mode = AppMode::CombatTrackerTUI,
                    5 => self.mode = AppMode::SearchTUI,
                    6 => {
                        self.mode = AppMode::MainMenu;
                        self.selected_index = 0;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn go_back(&mut self) {
        match self.mode {
            AppMode::CharactersMenu | AppMode::ToolsMenu => {
                self.mode = AppMode::MainMenu;
                self.selected_index = 0;
            }
            AppMode::CharacterCreation | AppMode::CharacterDisplay | AppMode::CharacterDeletion 
            | AppMode::CharacterCreationTUI | AppMode::CharacterDisplayTUI | AppMode::CharacterDeletionTUI => {
                self.mode = AppMode::CharactersMenu;
                self.selected_index = 0;
                self.clear_terminal_state();
            }
            AppMode::InitiativeTracker | AppMode::NpcGenerator | AppMode::EncounterRandomizer | AppMode::Dice | AppMode::CombatTracker | AppMode::Search 
            | AppMode::InitiativeTrackerTUI | AppMode::NpcGeneratorTUI | AppMode::EncounterRandomizerTUI | AppMode::DiceTUI => {
                self.mode = AppMode::ToolsMenu;
                self.selected_index = 0;
                self.clear_terminal_state();
            }
            AppMode::CombatTrackerTUI | AppMode::SearchTUI => {
                self.mode = AppMode::ToolsMenu;
                self.selected_index = 0;
                self.clear_terminal_state();
            }
            _ => {}
        }
    }

    fn clear_terminal_state(&mut self) {
        self.input_buffer.clear();
        self.output_history.clear();
        self.scroll_offset = 0;
        self.combat_tracker = None;
        self.current_state = "Ready".to_string();
        self.waiting_for = None;
        self.dice_results.clear();
    }

    fn handle_terminal_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter => {
                if !self.input_buffer.trim().is_empty() {
                    let command = self.input_buffer.trim().to_string();
                    self.command_history.push(command.clone());
                    self.history_index = None;
                    self.process_terminal_command(command);
                    self.input_buffer.clear();
                }
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Up => {
                if !self.command_history.is_empty() {
                    if let Some(index) = self.history_index {
                        if index > 0 {
                            self.history_index = Some(index - 1);
                        }
                    } else {
                        self.history_index = Some(self.command_history.len() - 1);
                    }
                    if let Some(index) = self.history_index {
                        self.input_buffer = self.command_history[index].clone();
                    }
                }
            }
            KeyCode::Down => {
                if let Some(index) = self.history_index {
                    if index < self.command_history.len() - 1 {
                        self.history_index = Some(index + 1);
                        self.input_buffer = self.command_history[index + 1].clone();
                    } else {
                        self.history_index = None;
                        self.input_buffer.clear();
                    }
                }
            }
            KeyCode::PageUp => {
                if self.scroll_offset > 0 {
                    self.scroll_offset = self.scroll_offset.saturating_sub(5);
                }
            }
            KeyCode::PageDown => {
                if self.scroll_offset + 10 < self.output_history.len() {
                    self.scroll_offset += 5;
                }
            }
            KeyCode::Esc => {
                self.go_back();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
    }

    fn process_terminal_command(&mut self, command: String) {
        match self.mode {
            AppMode::CombatTrackerTUI => self.process_combat_command(command),
            AppMode::SearchTUI => self.process_search_command(command),
            AppMode::CharacterCreationTUI => self.process_character_creation_command(command),
            AppMode::CharacterDisplayTUI => self.process_character_display_command(command),
            AppMode::CharacterDeletionTUI => self.process_character_deletion_command(command),
            AppMode::InitiativeTrackerTUI => self.process_initiative_command(command),
            AppMode::NpcGeneratorTUI => self.process_npc_generator_command(command),
            AppMode::EncounterRandomizerTUI => self.process_encounter_randomizer_command(command),
            AppMode::DiceTUI => self.process_dice_command(command),
            _ => {}
        }
    }

    fn process_combat_command(&mut self, command: String) {
        // Check if we're waiting for damage input after an attack
        if let Some(ref waiting) = self.waiting_for.clone() {
            if waiting.starts_with("damage_for_") {
                let target_name = waiting.strip_prefix("damage_for_").unwrap();
                
                // Try to parse as damage (either dice roll or number)
                if let Ok(damage) = command.trim().parse::<i32>() {
                    // Direct damage number
                    self.process_hit_command(target_name, damage);
                    self.waiting_for = None;
                    self.current_state = "Combat Ready".to_string();
                    return;
                } else {
                    // Try as dice roll
                    match crate::dice::roll_dice_with_crits(&command.trim()) {
                        Ok((rolls, total, crit_message)) => {
                            self.add_output(format!("🎲 Damage roll: {} (dice: {:?})", total, rolls));
                            if let Some(message) = crit_message {
                                self.add_output(message);
                            }
                            self.process_hit_command(target_name, total as i32);
                            self.waiting_for = None;
                            self.current_state = "Combat Ready".to_string();
                            return;
                        }
                        Err(_) => {
                            self.add_output("❌ Invalid damage input. Enter a number or dice expression (e.g., 2d6+3)".to_string());
                            return;
                        }
                    }
                }
            }
        }

        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        let cmd = parts[0].to_lowercase();
        
        match cmd.as_str() {
            "help" | "h" => {
                self.add_output("Combat Mode Commands:".to_string());
                self.add_output("  init - Initialize combat tracker".to_string());
                self.add_output("  stats [name] - Show character stats".to_string());
                self.add_output("  attack <target> - Roll attack against target's AC".to_string());
                self.add_output("  save <stat> [target] - Make saving throw (str/dex/con/int/wis/cha)".to_string());
                self.add_output("  hit <target> <amount> - Deal direct damage".to_string());
                self.add_output("  damage <name> <amount> - Apply damage".to_string());
                self.add_output("  heal <name> <amount> - Heal character".to_string());
                self.add_output("  status <target> add <status> [rounds] - Add status effect".to_string());
                self.add_output("  status <target> remove <status> - Remove status effect".to_string());
                self.add_output("  next|continue - Advance to next combatant".to_string());
                self.add_output("  search <query> - Search D&D 5e API".to_string());
                self.add_output("  show|list - Display current initiative order".to_string());
                self.add_output("  quit|exit - Exit combat mode".to_string());
                self.add_output("".to_string());
                self.add_output("Examples:".to_string());
                self.add_output("  attack goblin".to_string());
                self.add_output("  save wis fighter".to_string());
                self.add_output("  hit goblin 8".to_string());
                self.add_output("  status goblin add poisoned 3".to_string());
            }
            "init" | "initialize" => {
                self.initialize_combat();
            }
            "search" => {
                if let Some(_query) = parts.get(1) {
                    let full_query = parts[1..].join(" ");
                    self.handle_combat_search(&full_query);
                } else {
                    self.add_output("Usage: search <query>".to_string());
                    self.add_output("Example: search fireball".to_string());
                }
            }
            "quit" | "exit" | "q" => {
                self.add_output("Exiting combat mode...".to_string());
                self.mode = AppMode::ToolsMenu;
                self.selected_index = 0;
                self.input_buffer.clear();
                self.output_history.clear();
                self.scroll_offset = 0;
                self.combat_tracker = None;
            }
            "show" | "list" => {
                if let Some(ref tracker) = self.combat_tracker {
                    let mut lines = vec!["Initiative Order:".to_string()];
                    for (i, combatant) in tracker.combatants.iter().enumerate() {
                        let marker = if i == tracker.current_turn { "►" } else { " " };
                        let status_text = if combatant.status_effects.is_empty() {
                            "".to_string()
                        } else {
                            format!(" [{}]", combatant.status_effects.iter()
                                .map(|s| s.name.as_str()).collect::<Vec<_>>().join(", "))
                        };
                        lines.push(format!("{} {}. {} (Init: {}, HP: {}/{}, AC: {}){}",
                            marker, i + 1, combatant.name, combatant.initiative,
                            combatant.current_hp, combatant.max_hp, combatant.ac, status_text));
                    }
                    for line in lines {
                        self.add_output(line);
                    }
                } else {
                    self.add_output("No combat initialized. Use 'init' to start combat.".to_string());
                }
            }
            "next" | "continue" => {
                if let Some(ref mut tracker) = self.combat_tracker {
                    if tracker.combatants.is_empty() {
                        self.add_output("❌ No combatants in combat.".to_string());
                    } else {
                        let _old_turn = tracker.current_turn;
                        tracker.current_turn = (tracker.current_turn + 1) % tracker.combatants.len();
                        
                        let mut messages = Vec::new();
                        if tracker.current_turn == 0 {
                            tracker.round_number += 1;
                            messages.push(format!("🔄 Starting Round {}", tracker.round_number));
                        }
                        
                        let current = &tracker.combatants[tracker.current_turn];
                        messages.push(format!("🎯 It's {}'s turn! (Initiative: {}, HP: {}/{})", 
                            current.name, current.initiative, current.current_hp, current.max_hp));
                        
                        for message in messages {
                            self.add_output(message);
                        }
                    }
                } else {
                    self.add_output("No combat initialized. Use 'init' to start combat.".to_string());
                }
            }
            "stats" => {
                if let Some(ref tracker) = self.combat_tracker {
                    if parts.len() >= 2 {
                        let name = parts[1];
                        if let Some(combatant) = tracker.combatants.iter().find(|c| c.name.eq_ignore_ascii_case(name)) {
                            let mut messages = vec![
                                format!("📊 Stats for {}", combatant.name),
                                format!("  HP: {}/{} ({})", combatant.current_hp, combatant.max_hp, 
                                    if combatant.current_hp > 0 { "Alive" } else { "Unconscious/Dead" }),
                                format!("  AC: {}", combatant.ac),
                                format!("  Initiative: {}", combatant.initiative),
                                format!("  Type: {}", if combatant.is_player { "Player" } else { "NPC" }),
                            ];
                            
                            if !combatant.status_effects.is_empty() {
                                messages.push("  Status Effects:".to_string());
                                for effect in &combatant.status_effects {
                                    let duration_text = match effect.duration {
                                        Some(d) => format!(" ({} rounds)", d),
                                        None => " (permanent)".to_string(),
                                    };
                                    messages.push(format!("    - {}{}", effect.name, duration_text));
                                }
                            }
                            
                            for message in messages {
                                self.add_output(message);
                            }
                        } else {
                            self.add_output(format!("❌ Combatant '{}' not found", name));
                        }
                    } else {
                        // Show current combatant stats
                        if let Some(current) = tracker.combatants.get(tracker.current_turn) {
                            let messages = vec![
                                format!("📊 Current Turn: {}", current.name),
                                format!("  HP: {}/{}", current.current_hp, current.max_hp),
                                format!("  AC: {}", current.ac),
                            ];
                            
                            for message in messages {
                                self.add_output(message);
                            }
                        } else {
                            self.add_output("❌ No current combatant".to_string());
                        }
                    }
                } else {
                    self.add_output("No combat initialized. Use 'init' to start combat.".to_string());
                }
            }
            "attack" => {
                if parts.len() >= 2 {
                    let target_name = parts[1];
                    self.process_attack_command(target_name);
                } else {
                    self.add_output("Usage: attack <target>".to_string());
                    self.add_output("Example: attack goblin".to_string());
                }
            }
            "save" => {
                if parts.len() >= 2 {
                    let ability = parts[1].to_lowercase();
                    let target = if parts.len() >= 3 { parts[2] } else { "self" };
                    self.process_save_command(&ability, target);
                } else {
                    self.add_output("Usage: save <ability> [target]".to_string());
                    self.add_output("Abilities: str, dex, con, int, wis, cha".to_string());
                    self.add_output("Example: save wis goblin".to_string());
                }
            }
            "hit" => {
                if parts.len() >= 3 {
                    let target_name = parts[1];
                    if let Ok(damage_amount) = parts[2].parse::<i32>() {
                        self.process_hit_command(target_name, damage_amount);
                    } else {
                        self.add_output("❌ Invalid damage amount".to_string());
                    }
                } else {
                    self.add_output("Usage: hit <target> <amount>".to_string());
                    self.add_output("Example: hit goblin 8".to_string());
                }
            }
            "status" => {
                if parts.len() >= 4 {
                    let target = parts[1];
                    let action = parts[2].to_lowercase();
                    let status_name = parts[3];
                    let rounds = if parts.len() >= 5 { 
                        parts[4].parse::<i32>().ok() 
                    } else { 
                        None 
                    };
                    self.process_status_command(target, &action, status_name, rounds);
                } else {
                    self.add_output("Usage: status <target> <add|remove> <status> [rounds]".to_string());
                    self.add_output("Example: status goblin add poisoned 3".to_string());
                    self.add_output("Example: status fighter remove stunned".to_string());
                }
            }
            "damage" => {
                if parts.len() >= 3 {
                    let target_name = parts[1];
                    if let Ok(damage_amount) = parts[2].parse::<i32>() {
                        if let Some(ref mut tracker) = self.combat_tracker {
                            if let Some(combatant) = tracker.combatants.iter_mut().find(|c| c.name.eq_ignore_ascii_case(target_name)) {
                                let old_hp = combatant.current_hp;
                                combatant.current_hp = (combatant.current_hp - damage_amount).max(0);
                                
                                let mut messages = vec![
                                    format!("⚔️ {} takes {} damage! HP: {} → {}", 
                                        combatant.name, damage_amount, old_hp, combatant.current_hp)
                                ];
                                    
                                if combatant.current_hp <= 0 {
                                    messages.push(format!("💀 {} is unconscious/dead!", combatant.name));
                                }
                                
                                for message in messages {
                                    self.add_output(message);
                                }
                            } else {
                                self.add_output(format!("❌ Combatant '{}' not found", target_name));
                            }
                        } else {
                            self.add_output("No combat initialized.".to_string());
                        }
                    } else {
                        self.add_output("❌ Invalid damage amount".to_string());
                    }
                } else {
                    self.add_output("Usage: damage <target> <amount>".to_string());
                }
            }
            "heal" => {
                if parts.len() >= 3 {
                    let target_name = parts[1];
                    if let Ok(heal_amount) = parts[2].parse::<i32>() {
                        if let Some(ref mut tracker) = self.combat_tracker {
                            if let Some(combatant) = tracker.combatants.iter_mut().find(|c| c.name.eq_ignore_ascii_case(target_name)) {
                                let old_hp = combatant.current_hp;
                                combatant.current_hp = (combatant.current_hp + heal_amount).min(combatant.max_hp);
                                
                                let message = format!("💚 {} heals {} HP! HP: {} → {}", 
                                    combatant.name, heal_amount, old_hp, combatant.current_hp);
                                self.add_output(message);
                            } else {
                                self.add_output(format!("❌ Combatant '{}' not found", target_name));
                            }
                        } else {
                            self.add_output("No combat initialized.".to_string());
                        }
                    } else {
                        self.add_output("❌ Invalid heal amount".to_string());
                    }
                } else {
                    self.add_output("Usage: heal <target> <amount>".to_string());
                }
            }
            _ => {
                if self.combat_tracker.is_some() {
                    // Handle other combat commands
                    self.add_output(format!("Unknown command '{}'. Type 'help' for available commands.", cmd));
                } else {
                    self.add_output("No combat initialized. Use 'init' to start combat.".to_string());
                }
            }
        }
    }

    fn process_attack_command(&mut self, target_name: &str) {
        if let Some(ref tracker) = self.combat_tracker {
            if let Some(target) = tracker.combatants.iter().find(|c| c.name.eq_ignore_ascii_case(target_name)) {
                let target_ac = target.ac;
                
                // Roll d20 for attack
                match crate::dice::roll_dice_with_crits("1d20") {
                    Ok((rolls, total, crit_message)) => {
                        let attack_roll = rolls[0] as i32;
                        let hit = attack_roll >= target_ac;
                        
                        self.add_output(format!("⚔️  Attack Roll: {} (d20: {})", total, attack_roll));
                        
                        if let Some(message) = crit_message {
                            self.add_output(message);
                        }
                        
                        self.add_output(format!("🎯 Target AC: {}", target_ac));
                        
                        if hit {
                            self.add_output("💥 HIT! The attack connects!".to_string());
                            self.add_output("🎲 Enter damage (e.g., '2d6+3' or just '8'):".to_string());
                            self.current_state = format!("Waiting for damage against {}", target_name);
                            self.waiting_for = Some(format!("damage_for_{}", target_name));
                        } else {
                            self.add_output("🛡️  MISS! The attack fails to connect.".to_string());
                        }
                    }
                    Err(e) => {
                        self.add_output(format!("❌ Error rolling attack: {}", e));
                    }
                }
            } else {
                self.add_output(format!("❌ Target '{}' not found in combat", target_name));
            }
        } else {
            self.add_output("No combat initialized. Use 'init' to start combat.".to_string());
        }
    }

    fn process_save_command(&mut self, ability: &str, target: &str) {
        let ability_full = match ability {
            "str" => "Strength",
            "dex" => "Dexterity", 
            "con" => "Constitution",
            "int" => "Intelligence",
            "wis" => "Wisdom",
            "cha" => "Charisma",
            _ => {
                self.add_output("❌ Invalid ability. Use: str, dex, con, int, wis, cha".to_string());
                return;
            }
        };

        let target_name = if target == "self" {
            if let Some(ref tracker) = self.combat_tracker {
                if let Some(current) = tracker.combatants.get(tracker.current_turn) {
                    current.name.clone()
                } else {
                    self.add_output("❌ No current combatant".to_string());
                    return;
                }
            } else {
                self.add_output("No combat initialized.".to_string());
                return;
            }
        } else {
            target.to_string()
        };

        if let Some(ref tracker) = self.combat_tracker {
            if let Some(_combatant) = tracker.combatants.iter().find(|c| c.name.eq_ignore_ascii_case(&target_name)) {
                // Roll d20 for saving throw
                match crate::dice::roll_dice_with_crits("1d20") {
                    Ok((rolls, total, crit_message)) => {
                        self.add_output(format!("🎲 {} saving throw for {}: {} (d20: {})", 
                            ability_full, target_name, total, rolls[0]));
                        
                        if let Some(message) = crit_message {
                            self.add_output(message);
                        }
                    }
                    Err(e) => {
                        self.add_output(format!("❌ Error rolling saving throw: {}", e));
                    }
                }
            } else {
                self.add_output(format!("❌ Combatant '{}' not found", target_name));
            }
        } else {
            self.add_output("No combat initialized.".to_string());
        }
    }

    fn process_hit_command(&mut self, target_name: &str, damage: i32) {
        if let Some(ref mut tracker) = self.combat_tracker {
            if let Some(combatant) = tracker.combatants.iter_mut().find(|c| c.name.eq_ignore_ascii_case(target_name)) {
                let old_hp = combatant.current_hp;
                combatant.current_hp = (combatant.current_hp - damage).max(0);
                
                let mut messages = vec![
                    format!("⚔️ {} takes {} damage directly! HP: {} → {}", 
                        combatant.name, damage, old_hp, combatant.current_hp)
                ];
                    
                if combatant.current_hp <= 0 {
                    messages.push(format!("💀 {} is unconscious/dead!", combatant.name));
                }
                
                for message in messages {
                    self.add_output(message);
                }
            } else {
                self.add_output(format!("❌ Combatant '{}' not found", target_name));
            }
        } else {
            self.add_output("No combat initialized.".to_string());
        }
    }

    fn process_status_command(&mut self, target: &str, action: &str, status_name: &str, rounds: Option<i32>) {
        let target_name = if target == "self" {
            if let Some(ref tracker) = self.combat_tracker {
                if let Some(current) = tracker.combatants.get(tracker.current_turn) {
                    current.name.clone()
                } else {
                    self.add_output("❌ No current combatant".to_string());
                    return;
                }
            } else {
                self.add_output("No combat initialized.".to_string());
                return;
            }
        } else {
            target.to_string()
        };

        if let Some(ref mut tracker) = self.combat_tracker {
            if let Some(combatant) = tracker.combatants.iter_mut().find(|c| c.name.eq_ignore_ascii_case(&target_name)) {
                match action {
                    "add" => {
                        let status = crate::combat::StatusEffect {
                            name: status_name.to_string(),
                            description: None,
                            duration: rounds,
                        };
                        combatant.add_status(status);
                        
                        let duration_text = match rounds {
                            Some(r) => format!(" for {} rounds", r),
                            None => " (permanent)".to_string(),
                        };
                        self.add_output(format!("✅ Added status '{}' to {}{}", 
                            status_name, target_name, duration_text));
                    }
                    "remove" => {
                        if combatant.remove_status(status_name) {
                            self.add_output(format!("✅ Removed status '{}' from {}", 
                                status_name, target_name));
                        } else {
                            self.add_output(format!("❌ Status '{}' not found on {}", 
                                status_name, target_name));
                        }
                    }
                    _ => {
                        self.add_output("❌ Invalid action. Use 'add' or 'remove'".to_string());
                    }
                }
            } else {
                self.add_output(format!("❌ Combatant '{}' not found", target_name));
            }
        } else {
            self.add_output("No combat initialized.".to_string());
        }
    }

    fn process_search_command(&mut self, command: String) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        let cmd = parts[0].to_lowercase();
        
        match cmd.as_str() {
            "help" | "h" => {
                self.add_output("Search Commands:".to_string());
                self.add_output("  search <query> - Search all categories".to_string());
                self.add_output("  search <category> <query> - Search specific category".to_string());
                self.add_output("  categories - List available categories".to_string());
                self.add_output("  back - Return to tools menu".to_string());
                self.add_output("".to_string());
                self.add_output("Categories: spells, classes, equipment, monsters, races".to_string());
                self.add_output("Example: search fireball".to_string());
                self.add_output("Example: search spell magic missile".to_string());
            }
            "search" => {
                if let Some(_query) = parts.get(1) {
                    let full_query = parts[1..].join(" ");
                    self.handle_search_query(&full_query);
                } else {
                    self.add_output("Usage: search <query> or search <category> <query>".to_string());
                }
            }
            "categories" => {
                self.add_output("Available Categories:".to_string());
                self.add_output("  • spells - Magic spells".to_string());
                self.add_output("  • classes - Character classes".to_string());
                self.add_output("  • equipment (or items/gear) - Weapons, armor, and gear".to_string());
                self.add_output("  • monsters (or creatures) - Monsters and NPCs".to_string());
                self.add_output("  • races - Character races".to_string());
            }
            "back" | "exit" | "quit" => {
                self.add_output("Returning to tools menu...".to_string());
                self.mode = AppMode::ToolsMenu;
                self.selected_index = 0;
                self.input_buffer.clear();
                self.output_history.clear();
                self.scroll_offset = 0;
            }
            _ => {
                // Try to interpret the entire command as a search query
                self.handle_search_query(&command);
            }
        }
    }

    fn process_character_creation_command(&mut self, command: String) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd_string = if parts.is_empty() { 
            String::new() 
        } else { 
            parts[0].to_lowercase() 
        };
        let cmd = cmd_string.as_str();

        match cmd {
            "help" | "h" => {
                self.add_output("Character Creation Commands:".to_string());
                self.add_output("  create - Start character creation wizard".to_string());
                self.add_output("  back - Return to characters menu".to_string());
            }
            "create" => {
                self.add_output("🎭 Starting character creation wizard...".to_string());
                self.add_output("This will guide you through creating a new character.".to_string());
                self.add_output("Feature coming soon - interactive character creation!".to_string());
            }
            "back" | "exit" => {
                self.mode = AppMode::CharactersMenu;
                self.selected_index = 0;
                self.clear_terminal_state();
            }
            _ => {
                self.add_output(format!("Unknown command '{}'. Type 'help' for commands.", cmd));
            }
        }
    }

    fn process_character_display_command(&mut self, command: String) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd_string = if parts.is_empty() { 
            String::new() 
        } else { 
            parts[0].to_lowercase() 
        };
        let cmd = cmd_string.as_str();

        match cmd {
            "help" | "h" => {
                self.add_output("Character Display Commands:".to_string());
                self.add_output("  list - List all characters".to_string());
                self.add_output("  show <name> - Show specific character details".to_string());
                self.add_output("  back - Return to characters menu".to_string());
            }
            "list" => {
                self.add_output("📋 Available Characters:".to_string());
                if self.characters.is_empty() {
                    self.add_output("  No characters found.".to_string());
                } else {
                    let character_list: Vec<String> = self.characters.iter().enumerate()
                        .map(|(i, character)| {
                            format!("  {}. {} (Level {}, {})", 
                                i + 1, character.name, 
                                character.level.unwrap_or(1), 
                                character.class.as_ref().unwrap_or(&"Unknown".to_string()))
                        })
                        .collect();
                    for line in character_list {
                        self.add_output(line);
                    }
                }
            }
            "show" => {
                if parts.len() >= 2 {
                    let char_name = parts[1..].join(" ");
                    let character_data = self.characters.iter()
                        .find(|c| c.name.eq_ignore_ascii_case(&char_name))
                        .cloned();
                    
                    if let Some(character) = character_data {
                        self.display_character_details(&character);
                    } else {
                        self.add_output(format!("❌ Character '{}' not found", char_name));
                    }
                } else {
                    self.add_output("Usage: show <character_name>".to_string());
                }
            }
            "back" | "exit" => {
                self.mode = AppMode::CharactersMenu;
                self.selected_index = 0;
                self.clear_terminal_state();
            }
            _ => {
                self.add_output(format!("Unknown command '{}'. Type 'help' for commands.", cmd));
            }
        }
    }

    fn process_character_deletion_command(&mut self, command: String) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd_string = if parts.is_empty() { 
            String::new() 
        } else { 
            parts[0].to_lowercase() 
        };
        let cmd = cmd_string.as_str();

        match cmd {
            "help" | "h" => {
                self.add_output("Character Deletion Commands:".to_string());
                self.add_output("  list - List all characters".to_string());
                self.add_output("  delete <name> - Delete specific character".to_string());
                self.add_output("  back - Return to characters menu".to_string());
            }
            "list" => {
                self.add_output("📋 Characters available for deletion:".to_string());
                if self.characters.is_empty() {
                    self.add_output("  No characters found.".to_string());
                } else {
                    let character_list: Vec<String> = self.characters.iter().enumerate()
                        .map(|(i, character)| format!("  {}. {}", i + 1, character.name))
                        .collect();
                    for line in character_list {
                        self.add_output(line);
                    }
                }
            }
            "delete" => {
                if parts.len() >= 2 {
                    let char_name = parts[1..].join(" ");
                    if let Some(index) = self.characters.iter().position(|c| c.name.eq_ignore_ascii_case(&char_name)) {
                        let removed = self.characters.remove(index);
                        self.add_output(format!("🗑️  Deleted character '{}'", removed.name));
                        crate::file_manager::save_characters(self.characters.clone());
                    } else {
                        self.add_output(format!("❌ Character '{}' not found", char_name));
                    }
                } else {
                    self.add_output("Usage: delete <character_name>".to_string());
                }
            }
            "back" | "exit" => {
                self.mode = AppMode::CharactersMenu;
                self.selected_index = 0;
                self.clear_terminal_state();
            }
            _ => {
                self.add_output(format!("Unknown command '{}'. Type 'help' for commands.", cmd));
            }
        }
    }

    fn process_initiative_command(&mut self, command: String) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd_string = if parts.is_empty() { 
            String::new() 
        } else { 
            parts[0].to_lowercase() 
        };
        let cmd = cmd_string.as_str();

        match cmd {
            "help" | "h" => {
                self.add_output("Initiative Tracker Commands:".to_string());
                self.add_output("  roll <name> - Roll initiative for character/monster".to_string());
                self.add_output("  list - Show current initiative order".to_string());
                self.add_output("  clear - Clear all initiative rolls".to_string());
                self.add_output("  back - Return to tools menu".to_string());
            }
            "roll" => {
                if parts.len() >= 2 {
                    let name = parts[1..].join(" ");
                    match crate::dice::roll_dice_with_crits("1d20") {
                        Ok((rolls, total, crit_message)) => {
                            self.add_output(format!("🎲 {} rolled initiative: {} (d20: {})", 
                                name, total, rolls[0]));
                            if let Some(message) = crit_message {
                                self.add_output(message);
                            }
                        }
                        Err(e) => {
                            self.add_output(format!("❌ Error rolling initiative: {}", e));
                        }
                    }
                } else {
                    self.add_output("Usage: roll <name>".to_string());
                }
            }
            "list" => {
                self.add_output("📋 Initiative Order: (Feature coming soon)".to_string());
            }
            "clear" => {
                self.add_output("🧹 Cleared all initiative rolls".to_string());
            }
            "back" | "exit" => {
                self.mode = AppMode::ToolsMenu;
                self.selected_index = 0;
                self.clear_terminal_state();
            }
            _ => {
                self.add_output(format!("Unknown command '{}'. Type 'help' for commands.", cmd));
            }
        }
    }

    fn process_npc_generator_command(&mut self, command: String) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd_string = if parts.is_empty() { 
            String::new() 
        } else { 
            parts[0].to_lowercase() 
        };
        let cmd = cmd_string.as_str();

        match cmd {
            "help" | "h" => {
                self.add_output("NPC Generator Commands:".to_string());
                self.add_output("  random - Generate completely random NPC".to_string());
                self.add_output("  custom <race> <class> - Generate NPC with specific race/class".to_string());
                self.add_output("  races - List available races".to_string());
                self.add_output("  classes - List available classes".to_string());
                self.add_output("  back - Return to tools menu".to_string());
            }
            "random" => {
                self.generate_random_npc();
            }
            "custom" => {
                if parts.len() >= 3 {
                    let race = parts[1];
                    let class = parts[2];
                    self.generate_custom_npc(race, class);
                } else {
                    self.add_output("Usage: custom <race> <class>".to_string());
                    self.add_output("Example: custom elf wizard".to_string());
                }
            }
            "races" => {
                self.add_output("Available Races:".to_string());
                self.add_output("human, elf, dwarf, halfling, dragonborn, gnome, half-elf, half-orc, tiefling".to_string());
            }
            "classes" => {
                self.add_output("Available Classes:".to_string());
                self.add_output("fighter, wizard, cleric, rogue, ranger, paladin, barbarian, bard, druid, monk, sorcerer, warlock".to_string());
            }
            "back" | "exit" => {
                self.mode = AppMode::ToolsMenu;
                self.selected_index = 0;
                self.clear_terminal_state();
            }
            _ => {
                self.add_output(format!("Unknown command '{}'. Type 'help' for commands.", cmd));
            }
        }
    }

    fn process_encounter_randomizer_command(&mut self, command: String) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd_string = if parts.is_empty() { 
            String::new() 
        } else { 
            parts[0].to_lowercase() 
        };
        let cmd = cmd_string.as_str();

        match cmd {
            "help" | "h" => {
                self.add_output("⚔️  Encounter Randomizer Commands:".to_string());
                self.add_output("  generate <CR> - Generate encounter for Challenge Rating (e.g., generate 5)".to_string());
                self.add_output("  generate <CR> <count> - Generate encounter with specific creature count".to_string());
                self.add_output("  cr <CR> - Set Challenge Rating for next generation".to_string());
                self.add_output("  examples - Show CR examples and guidelines".to_string());
                self.add_output("  back - Return to tools menu".to_string());
            }
            "generate" | "gen" => {
                if parts.len() >= 2 {
                    if let Ok(cr) = parts[1].parse::<f64>() {
                        if cr >= 0.0 && cr <= 30.0 {
                            let creature_count = if parts.len() >= 3 {
                                parts[2].parse::<usize>().unwrap_or(1).clamp(1, 8)
                            } else {
                                1
                            };
                            self.generate_encounter_tui(cr, creature_count);
                        } else {
                            self.add_output("❌ CR must be between 0 and 30".to_string());
                        }
                    } else {
                        self.add_output("❌ Invalid CR format. Use numbers like 0.25, 1, 5, etc.".to_string());
                    }
                } else {
                    self.add_output("Usage: generate <CR> [creature_count]".to_string());
                    self.add_output("Examples: generate 1, generate 5 3, generate 0.25 2".to_string());
                }
            }
            "cr" => {
                if parts.len() >= 2 {
                    if let Ok(cr) = parts[1].parse::<f64>() {
                        if cr >= 0.0 && cr <= 30.0 {
                            self.add_output(format!("✅ Challenge Rating set to {}", self.format_cr_tui(cr)));
                            self.add_output("Use 'generate' to create an encounter with this CR".to_string());
                            // Store CR in app state if needed
                        } else {
                            self.add_output("❌ CR must be between 0 and 30".to_string());
                        }
                    } else {
                        self.add_output("❌ Invalid CR format. Use numbers like 0.25, 1, 5, etc.".to_string());
                    }
                } else {
                    self.add_output("Usage: cr <CR>".to_string());
                    self.add_output("Examples: cr 1, cr 5, cr 0.25".to_string());
                }
            }
            "examples" | "ex" => {
                self.add_output("⚔️  Challenge Rating Examples:".to_string());
                self.add_output("".to_string());
                self.add_output("  CR 0 (1/8, 1/4) - Weak creatures (rats, commoners)".to_string());
                self.add_output("  CR 1-2 - Low-level threats (goblins, wolves)".to_string());
                self.add_output("  CR 3-5 - Mid-level encounters (orcs, bugbears)".to_string());
                self.add_output("  CR 6-10 - Challenging foes (trolls, giants)".to_string());
                self.add_output("  CR 11-15 - Dangerous enemies (dragons, demons)".to_string());
                self.add_output("  CR 16+ - Epic threats (ancient dragons, gods)".to_string());
                self.add_output("".to_string());
                self.add_output("Use fractional CR: 0.125 (1/8), 0.25 (1/4), 0.5 (1/2)".to_string());
            }
            "back" | "exit" => {
                self.mode = AppMode::ToolsMenu;
                self.selected_index = 0;
                self.clear_terminal_state();
            }
            _ => {
                self.add_output(format!("Unknown command '{}'. Type 'help' for commands.", cmd));
            }
        }
    }

    fn generate_encounter_tui(&mut self, cr: f64, creature_count: usize) {
        use crate::races_classes::{get_random_race, get_random_class};
        
        self.add_output(format!("🎲 Generating encounter for CR {} with {} creature(s)...", 
                               self.format_cr_tui(cr), creature_count));
        self.add_output("".to_string());
        
        // Calculate individual creature CR for balanced encounter
        let individual_cr = if creature_count == 1 {
            cr
        } else {
            let multiplier = match creature_count {
                2 => 0.7,
                3..=4 => 0.5,
                5..=6 => 0.4,
                _ => 0.3,
            };
            (cr * multiplier).max(0.125)
        };
        
        if creature_count > 1 {
            self.add_output(format!("🎯 Individual creature CR adjusted to {} for balance", 
                                   self.format_cr_tui(individual_cr)));
            self.add_output("".to_string());
        }
        
        for i in 1..=creature_count {
            let race = get_random_race();
            let class = get_random_class();
            let creature_name = format!("{} {} #{}", race, class, i);
            
            // Generate stats based on CR (using the same logic from main.rs)
            let (base_hp, base_ac, prof_bonus) = match individual_cr {
                x if x < 0.25 => (7, 11, 2),
                x if x < 0.5 => (22, 12, 2),
                x if x < 1.0 => (32, 13, 2),
                x if x < 2.0 => (58, 13, 2),
                x if x < 3.0 => (71, 13, 2),
                x if x < 4.0 => (84, 14, 2),
                x if x < 5.0 => (97, 14, 2),
                x if x < 6.0 => (110, 15, 3),
                x if x < 7.0 => (123, 15, 3),
                x if x < 8.0 => (136, 15, 3),
                x if x < 9.0 => (149, 16, 3),
                x if x < 10.0 => (162, 16, 4),
                x if x < 11.0 => (175, 17, 4),
                x if x < 13.0 => (195, 17, 4),
                x if x < 15.0 => (225, 18, 5),
                x if x < 17.0 => (255, 18, 5),
                x if x < 21.0 => (285, 19, 6),
                _ => (400, 20, 7),
            };
            
            // Add randomization
            let hp_variance = (base_hp as f64 * 0.2) as i32;
            let hp = (base_hp + (rand::random::<i32>() % (hp_variance * 2 + 1)) - hp_variance).max(1);
            
            let ac_variance = if base_ac < 15 { 1 } else { 2 };
            let ac = base_ac + (rand::random::<i32>() % (ac_variance * 2 + 1)) - ac_variance;
            
            let speed = ((rand::random::<u8>() % 4) + 3) * 10;
            
            // Generate ability scores
            let ability_bonus = match individual_cr {
                x if x < 1.0 => 0,
                x if x < 5.0 => 1,
                x if x < 10.0 => 2,
                x if x < 15.0 => 3,
                x if x < 20.0 => 4,
                _ => 5,
            };
            
            let strength = (self.roll_3d6_tui() + ability_bonus as u8).min(20);
            let dexterity = (self.roll_3d6_tui() + ability_bonus as u8).min(20);
            let constitution = (self.roll_3d6_tui() + ability_bonus as u8).min(20);
            let intelligence = (self.roll_3d6_tui() + ability_bonus as u8).min(20);
            let wisdom = (self.roll_3d6_tui() + ability_bonus as u8).min(20);
            let charisma = (self.roll_3d6_tui() + ability_bonus as u8).min(20);
            
            // Display creature
            self.add_output(format!("═══ {} ═══", creature_name));
            self.add_output(format!("CR: {} | Race: {} | Class: {}", 
                                   self.format_cr_tui(individual_cr), race, class));
            self.add_output(format!("HP: {} | AC: {} | Speed: {} ft | Prof: +{}", 
                                   hp, ac, speed, prof_bonus));
            self.add_output(format!("STR: {} | DEX: {} | CON: {} | INT: {} | WIS: {} | CHA: {}", 
                                   strength, dexterity, constitution, intelligence, wisdom, charisma));
            self.add_output("".to_string());
        }
        
        self.add_output("📋 Encounter Summary:".to_string());
        self.add_output(format!("Total CR: {} | Creatures: {}", self.format_cr_tui(cr), creature_count));
        self.add_output("".to_string());
        self.add_output("💡 Tip: Use 'generate <CR> <count>' to create more encounters!".to_string());
    }
    
    fn format_cr_tui(&self, cr: f64) -> String {
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
    
    fn roll_3d6_tui(&self) -> u8 {
        let roll1 = (rand::random::<u8>() % 6) + 1;
        let roll2 = (rand::random::<u8>() % 6) + 1;
        let roll3 = (rand::random::<u8>() % 6) + 1;
        (roll1 + roll2 + roll3).clamp(1, 20)
    }

    fn process_dice_command(&mut self, command: String) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd_string = if parts.is_empty() { 
            String::new() 
        } else { 
            parts[0].to_lowercase() 
        };
        let cmd = cmd_string.as_str();

        match cmd {
            "help" | "h" => {
                self.add_output("🎲 Dice Roller Commands:".to_string());
                self.add_output("  roll <dice> - Roll dice (e.g., 1d20, 2d6+3, 4d8)".to_string());
                self.add_output("  advantage - Roll with advantage (2d20, keep higher)".to_string());
                self.add_output("  disadvantage - Roll with disadvantage (2d20, keep lower)".to_string());
                self.add_output("  stats - Roll 4d6 drop lowest for ability scores".to_string());
                self.add_output("  back - Return to tools menu".to_string());
            }
            "roll" => {
                if parts.len() >= 2 {
                    let dice_expr = parts[1..].join("");
                    self.roll_dice_with_display(&dice_expr);
                } else {
                    self.add_output("Usage: roll <dice_expression>".to_string());
                    self.add_output("Examples: roll 1d20, roll 2d6+3, roll 4d8".to_string());
                }
            }
            "advantage" => {
                self.add_output("🎲 Rolling with advantage (2d20, keep higher):".to_string());
                self.roll_dice_with_display("2d20");
                self.add_output("📈 Use the HIGHER roll for advantage!".to_string());
            }
            "disadvantage" => {
                self.add_output("🎲 Rolling with disadvantage (2d20, keep lower):".to_string());
                self.roll_dice_with_display("2d20");
                self.add_output("📉 Use the LOWER roll for disadvantage!".to_string());
            }
            "stats" => {
                self.add_output("🎲 Rolling ability scores (4d6, drop lowest):".to_string());
                self.add_output("".to_string());
                for ability in &["Strength", "Dexterity", "Constitution", "Intelligence", "Wisdom", "Charisma"] {
                    self.roll_ability_score(ability);
                }
            }
            "back" | "exit" => {
                self.mode = AppMode::ToolsMenu;
                self.selected_index = 0;
                self.clear_terminal_state();
            }
            _ => {
                // Try to interpret as dice roll
                self.roll_dice_with_display(&command);
            }
        }
    }

    // Helper functions for the new TUI modes
    fn display_character_details(&mut self, character: &Character) {
        self.add_output(format!("📋 Character Details: {}", character.name));
        self.add_output("".to_string());
        
        if let Some(level) = character.level {
            self.add_output(format!("Level: {}", level));
        }
        
        if let Some(class) = &character.class {
            self.add_output(format!("Class: {}", class));
        }
        
        if let Some(race) = &character.race {
            self.add_output(format!("Race: {}", race));
        }
        
        self.add_output("".to_string());
        
        // Ability Scores
        self.add_output("Ability Scores:".to_string());
        if let Some(str_val) = character.stre {
            self.add_output(format!("  Strength: {} ({})", str_val, character.get_strength_modifier()));
        }
        if let Some(dex_val) = character.dext {
            self.add_output(format!("  Dexterity: {} ({})", dex_val, character.get_dexterity_modifier()));
        }
        if let Some(con_val) = character.cons {
            self.add_output(format!("  Constitution: {} ({})", con_val, character.get_constitution_modifier()));
        }
        if let Some(int_val) = character.intl {
            self.add_output(format!("  Intelligence: {} ({})", int_val, character.get_intelligence_modifier()));
        }
        if let Some(wis_val) = character.wisd {
            self.add_output(format!("  Wisdom: {} ({})", wis_val, character.get_wisdom_modifier()));
        }
        if let Some(cha_val) = character.chas {
            self.add_output(format!("  Charisma: {} ({})", cha_val, character.get_charisma_modifier()));
        }
        
        self.add_output("".to_string());
        
        // Combat Stats
        if let (Some(hp), Some(ac)) = (character.hp, character.ac) {
            self.add_output(format!("HP: {}, AC: {}", hp, ac));
        }
        
        if let Some(speed) = character.speed {
            self.add_output(format!("Speed: {} ft", speed));
        }
    }

    fn generate_random_npc(&mut self) {
        use crate::races_classes::{get_random_race, get_random_class};
        
        self.add_output("🎲 Generating random NPC...".to_string());
        
        let race = get_random_race();
        let class = get_random_class();
        let ac = (rand::random::<u8>() % 11) + 10; // 10-20
        let hp = (rand::random::<u8>() % 41) + 10; // 10-50
        let speed = ((rand::random::<u8>() % 7) + 2) * 10; // 20-80
        
        self.add_output("".to_string());
        self.add_output("╔═══════════════════════════════════════╗".to_string());
        self.add_output("║            Generated NPC              ║".to_string());
        self.add_output("╠═══════════════════════════════════════╣".to_string());
        self.add_output(format!("║ Race: {:<31} ║", race));
        self.add_output(format!("║ Class: {:<30} ║", class));
        self.add_output(format!("║ AC: {:<33} ║", ac));
        self.add_output(format!("║ HP: {:<33} ║", hp));
        self.add_output(format!("║ Speed: {} feet{:<21} ║", speed, ""));
        self.add_output("╚═══════════════════════════════════════╝".to_string());
    }

    fn generate_custom_npc(&mut self, race: &str, class: &str) {
        self.add_output(format!("🎲 Generating {} {}...", race, class));
        
        let ac = (rand::random::<u8>() % 11) + 10; // 10-20
        let hp = (rand::random::<u8>() % 41) + 10; // 10-50
        let speed = ((rand::random::<u8>() % 7) + 2) * 10; // 20-80
        
        self.add_output("".to_string());
        self.add_output("╔═══════════════════════════════════════╗".to_string());
        self.add_output("║          Generated Custom NPC         ║".to_string());
        self.add_output("╠═══════════════════════════════════════╣".to_string());
        self.add_output(format!("║ Race: {:<31} ║", race));
        self.add_output(format!("║ Class: {:<30} ║", class));
        self.add_output(format!("║ AC: {:<33} ║", ac));
        self.add_output(format!("║ HP: {:<33} ║", hp));
        self.add_output(format!("║ Speed: {} feet{:<21} ║", speed, ""));
        self.add_output("╚═══════════════════════════════════════╝".to_string());
    }

    fn roll_dice_with_display(&mut self, dice_expr: &str) {
        match crate::dice::roll_dice_with_crits(dice_expr) {
            Ok((rolls, total, crit_message)) => {
                self.add_output("".to_string());
                self.add_output("┌─────────────────────────────────┐".to_string());
                self.add_output("│         🎲 DICE ROLL! 🎲         │".to_string());
                self.add_output("├─────────────────────────────────┤".to_string());
                self.add_output(format!("│ Expression: {:<19} │", dice_expr));
                self.add_output(format!("│ Individual Rolls: {:<13} │", format!("{:?}", rolls)));
                self.add_output(format!("│ TOTAL: {:<23} │", total));
                
                if let Some(message) = crit_message {
                    self.add_output("├─────────────────────────────────┤".to_string());
                    self.add_output(format!("│ {:<31} │", message));
                }
                
                self.add_output("└─────────────────────────────────┘".to_string());
                self.add_output("".to_string());
            }
            Err(e) => {
                self.add_output(format!("❌ Error rolling dice: {}", e));
                self.add_output("💡 Try format like: 1d20, 2d6+3, 4d8".to_string());
            }
        }
    }

    fn roll_ability_score(&mut self, ability_name: &str) {
        // Roll 4d6, drop lowest
        let mut rolls = vec![];
        for _ in 0..4 {
            rolls.push((rand::random::<u8>() % 6) + 1);
        }
        rolls.sort_by(|a, b| b.cmp(a)); // Sort descending
        let total: u8 = rolls[0] + rolls[1] + rolls[2]; // Take top 3
        
        self.add_output(format!("  {}: {} (rolled: [{}, {}, {}, {}], dropped: {})", 
            ability_name, total, rolls[0], rolls[1], rolls[2], rolls[3], rolls[3]));
    }

    fn add_output(&mut self, text: String) {
        self.output_history.push(text);
        // Auto-scroll to bottom
        if self.output_history.len() > 10 {
            self.scroll_offset = self.output_history.len().saturating_sub(10);
        }
    }

    fn initialize_combat(&mut self) {
        self.add_output("⚔️ Enhanced Combat Tracker ⚔️".to_string());
        self.add_output("Initializing combat setup...".to_string());
        
        // Create a combat tracker with some example combatants for testing
        let mut tracker = crate::combat::CombatTracker::new();
        
        // Add a sample fighter
        let fighter = crate::combat::Combatant::new_npc(
            "Fighter".to_string(),
            30, // HP
            16, // AC
            15, // Initiative
        );
        tracker.combatants.push(fighter);
        
        // Add a sample goblin
        let goblin = crate::combat::Combatant::new_npc(
            "Goblin".to_string(),
            7,  // HP
            13, // AC
            12, // Initiative
        );
        tracker.combatants.push(goblin);
        
        // Sort by initiative (highest first)
        tracker.combatants.sort_by(|a, b| b.initiative.cmp(&a.initiative));
        
        self.combat_tracker = Some(tracker);
        
        self.add_output("Combat initialized with sample characters!".to_string());
        self.add_output("".to_string());
        self.add_output("Combatants added:".to_string());
        self.add_output("  • Fighter (HP: 30, AC: 16, Init: 15)".to_string());
        self.add_output("  • Goblin (HP: 7, AC: 13, Init: 12)".to_string());
        self.add_output("".to_string());
        self.add_output("Type 'show' to see initiative order, or 'next' to start combat!".to_string());
    }

    fn handle_combat_search(&mut self, query: &str) {
        self.add_output(format!("🔍 Searching for '{}'...", query));
        
        // Create a blocking task to handle the async search
        let query_clone = query.to_string();
        
        // Create runtime for async operations
        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                let client = crate::search::DndSearchClient::new();
                
                rt.block_on(async {
                    match client.search(&query_clone, None).await {
                        Ok(results) => {
                            if results.is_empty() {
                                self.add_output(format!("❌ No exact match found for '{}'", query_clone));
                                
                                let suggestions = client.get_suggestions(&query_clone, None).await;
                                if !suggestions.is_empty() {
                                    self.add_output("🔍 Similar items found:".to_string());
                                    for (i, suggestion) in suggestions.iter().take(3).enumerate() {
                                        self.add_output(format!("  {}. {}", i + 1, suggestion));
                                    }
                                }
                            } else {
                                self.add_output(format!("✅ Found {} result(s):", results.len()));
                                
                                for (i, result) in results.iter().take(2).enumerate() { // Show max 2 results in combat
                                    if results.len() > 1 {
                                        self.add_output(format!("--- Result {} ---", i + 1));
                                    }
                                    
                                    self.add_output(format!("📝 {}: {}", result.index(), result.name()));
                                    
                                    // Display key info only (first 10 lines)
                                    let content_lines: Vec<&str> = result.page.content.lines().collect();
                                    for line in content_lines.iter().take(10) {
                                        self.add_output(line.to_string());
                                    }
                                    
                                    if content_lines.len() > 10 {
                                        self.add_output(format!("... (use search mode for full details)"));
                                    }
                                    
                                    if i == 0 && results.len() > 1 {
                                        self.add_output("".to_string());
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            self.add_output(format!("❌ Search failed: {}", e));
                        }
                    }
                });
            }
            Err(e) => {
                self.add_output(format!("❌ Failed to create async runtime: {}", e));
                self.add_output("Search functionality unavailable.".to_string());
            }
        }
        
        self.add_output("".to_string());
        self.add_output("📋 Returning to combat...".to_string());
    }

    fn handle_search_query(&mut self, query: &str) {
        self.add_output(format!("🔍 Searching for '{}'...", query));
        
        // Create a blocking task to handle the async search
        let query_clone = query.to_string();
        
        // Create runtime for async operations
        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                let client = crate::search::DndSearchClient::new();
                
                rt.block_on(async {
                    match client.search(&query_clone, None).await {
                        Ok(results) => {
                            if results.is_empty() {
                                self.add_output(format!("❌ No exact match found for '{}'", query_clone));
                                
                                let suggestions = client.get_suggestions(&query_clone, None).await;
                                if !suggestions.is_empty() {
                                    self.add_output("🔍 Similar items found:".to_string());
                                    for (i, suggestion) in suggestions.iter().take(5).enumerate() {
                                        self.add_output(format!("  {}. {}", i + 1, suggestion));
                                    }
                                    self.add_output("".to_string());
                                    self.add_output("💡 Try searching for one of these suggestions".to_string());
                                }
                            } else {
                                self.add_output(format!("✅ Found {} result(s):", results.len()));
                                self.add_output("".to_string());
                                
                                for (i, result) in results.iter().enumerate() {
                                    if results.len() > 1 {
                                        self.add_output(format!("--- Result {} ---", i + 1));
                                    }
                                    
                                    // Display result information
                                    self.add_output(format!("📝 Name: {}", result.name()));
                                    self.add_output(format!("🏷️ Type: {}", result.index()));
                                    self.add_output(format!("🔗 Source: {}", result.page.url));
                                    self.add_output("".to_string());
                                    
                                    // Display content with line breaks
                                    let content_lines: Vec<&str> = result.page.content.lines().collect();
                                    for line in content_lines.iter().take(20) { // Show first 20 lines
                                        self.add_output(line.to_string());
                                    }
                                    
                                    if content_lines.len() > 20 {
                                        self.add_output(format!("... ({} more lines)", content_lines.len() - 20));
                                    }
                                    
                                    if i < results.len() - 1 {
                                        self.add_output("".to_string());
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            self.add_output(format!("❌ Search failed: {}", e));
                            self.add_output("💡 This might be due to network issues".to_string());
                        }
                    }
                });
            }
            Err(e) => {
                self.add_output(format!("❌ Failed to create async runtime: {}", e));
                self.add_output("Search functionality unavailable.".to_string());
            }
        }
    }
}

// Theme colors - Dark blue with black and white highlights
pub const BACKGROUND_COLOR: Color = Color::Rgb(16, 24, 48);       // Dark blue
pub const MENU_COLOR: Color = Color::Rgb(32, 48, 96);             // Medium blue
pub const SELECTED_COLOR: Color = Color::Rgb(64, 96, 192);        // Lighter blue
pub const TEXT_COLOR: Color = Color::White;
pub const BORDER_COLOR: Color = Color::Rgb(128, 144, 192);        // Light blue-gray

pub fn run_tui(mut app: App) -> Result<App, Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run main loop
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    // Ctrl+Q to quit
                    KeyCode::Char('q') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => break,
                    _ => app.handle_key(key.code),
                }
            }
        }

        // Initialize TUI modes when switching to them
        match app.mode {
            AppMode::Exit => break,
            AppMode::CharacterCreationTUI => {
                // Initialize character creation TUI
                if app.output_history.is_empty() {
                    app.add_output("🎭 Character Creation - Interactive Mode 🎭".to_string());
                    app.add_output("Type 'help' for available commands or 'create' to start".to_string());
                    app.current_state = "Character Creation Ready".to_string();
                }
            }
            AppMode::CharacterDisplayTUI => {
                // Initialize character display TUI
                if app.output_history.is_empty() {
                    app.add_output("📋 Character Display - Interactive Mode 📋".to_string());
                    app.add_output("Type 'help' for commands or 'list' to see all characters".to_string());
                    app.current_state = "Character Display Ready".to_string();
                }
            }
            AppMode::CharacterDeletionTUI => {
                // Initialize character deletion TUI
                if app.output_history.is_empty() {
                    app.add_output("🗑️  Character Deletion - Interactive Mode 🗑️".to_string());
                    app.add_output("Type 'help' for commands or 'list' to see characters".to_string());
                    app.current_state = "Character Deletion Ready".to_string();
                }
            }
            AppMode::InitiativeTrackerTUI => {
                // Initialize initiative tracker TUI
                if app.output_history.is_empty() {
                    app.add_output("⚡ Initiative Tracker - Interactive Mode ⚡".to_string());
                    app.add_output("Type 'help' for commands or 'roll <name>' to roll initiative".to_string());
                    app.current_state = "Initiative Tracker Ready".to_string();
                }
            }
            AppMode::NpcGeneratorTUI => {
                // Initialize NPC generator TUI
                if app.output_history.is_empty() {
                    app.add_output("🎭 NPC Generator - Interactive Mode 🎭".to_string());
                    app.add_output("Type 'help' for commands or 'random' to generate an NPC".to_string());
                    app.current_state = "NPC Generator Ready".to_string();
                }
            }
            AppMode::EncounterRandomizerTUI => {
                // Initialize encounter randomizer TUI
                if app.output_history.is_empty() {
                    app.add_output("⚔️  Encounter Randomizer - Interactive Mode ⚔️".to_string());
                    app.add_output("Type 'help' for commands or 'generate <CR>' to create an encounter".to_string());
                    app.add_output("Example: 'generate 5' for CR 5 encounter".to_string());
                    app.current_state = "Encounter Randomizer Ready".to_string();
                }
            }
            AppMode::DiceTUI => {
                // Initialize dice roller TUI
                if app.output_history.is_empty() {
                    app.add_output("🎲 Dice Roller - Interactive Mode 🎲".to_string());
                    app.add_output("Type 'help' for commands or 'roll 1d20' to start rolling".to_string());
                    app.current_state = "Dice Roller Ready".to_string();
                }
            }
            AppMode::CombatTrackerTUI => {
                // Initialize combat tracker if not already done
                if app.combat_tracker.is_none() {
                    app.add_output("⚔️ Combat Tracker - Interactive Mode ⚔️".to_string());
                    app.add_output("Type 'init' to initialize combat or 'help' for commands".to_string());
                    app.current_state = "Combat Tracker Ready".to_string();
                }
            }
            AppMode::SearchTUI => {
                // Initialize search mode
                if app.output_history.is_empty() {
                    app.add_output("🔍 D&D 5e Search - Interactive Mode 🔍".to_string());
                    app.add_output("Type 'search <query>' to search or 'help' for commands".to_string());
                    app.current_state = "Search Ready".to_string();
                }
            }
            _ => {}
        }

        if app.should_quit {
            break;
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(app)
}

pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();
    
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),      // Title
            Constraint::Min(10),        // Main content
            Constraint::Length(3),      // Help/Status
        ])
        .split(size);

    // Title
    let title = get_title_for_mode(&app.mode);
    let title_paragraph = Paragraph::new(title)
        .style(Style::default().fg(TEXT_COLOR).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR))
                .style(Style::default().bg(BACKGROUND_COLOR))
        );
    f.render_widget(title_paragraph, chunks[0]);

    // Main content
    match app.mode {
        AppMode::CombatTrackerTUI | AppMode::SearchTUI | AppMode::CharacterCreationTUI 
        | AppMode::CharacterDisplayTUI | AppMode::CharacterDeletionTUI | AppMode::InitiativeTrackerTUI 
        | AppMode::NpcGeneratorTUI | AppMode::EncounterRandomizerTUI | AppMode::DiceTUI => {
            render_terminal_content(f, chunks[1], app);
        }
        _ => {
            render_main_content(f, chunks[1], app);
        }
    }

    // Help text
    let help_text = get_help_text(&app.mode);
    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(TEXT_COLOR))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR))
                .style(Style::default().bg(BACKGROUND_COLOR))
        );
    f.render_widget(help_paragraph, chunks[2]);

    // Show message if present
    if let Some(ref message) = app.message {
        let popup_area = centered_rect(60, 20, size);
        f.render_widget(Clear, popup_area);
        let message_popup = Paragraph::new(message.as_str())
            .style(Style::default().fg(TEXT_COLOR))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(BORDER_COLOR))
                    .style(Style::default().bg(MENU_COLOR))
                    .title("Message")
            );
        f.render_widget(message_popup, popup_area);
    }
}

fn render_main_content(f: &mut Frame, area: Rect, app: &mut App) {
    let items = app.get_menu_items();
    
    if items.is_empty() {
        let content = Paragraph::new("Loading...")
            .style(Style::default().fg(TEXT_COLOR))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(BORDER_COLOR))
                    .style(Style::default().bg(BACKGROUND_COLOR))
            );
        f.render_widget(content, area);
        return;
    }

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, &item)| {
            let style = if i == app.selected_index {
                Style::default()
                    .bg(SELECTED_COLOR)
                    .fg(TEXT_COLOR)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(TEXT_COLOR)
            };
            
            let content = if i == app.selected_index {
                format!("► {}", item)
            } else {
                format!("  {}", item)
            };
            
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR))
                .style(Style::default().bg(MENU_COLOR))
        )
        .style(Style::default().fg(TEXT_COLOR));

    f.render_widget(list, area);
}

fn render_terminal_content(f: &mut Frame, area: Rect, app: &mut App) {
    // Create layout for terminal: output area and input area
    let terminal_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),     // Output area (scrollable)
            Constraint::Length(3),   // Input area
        ])
        .split(area);

    // Render output area
    render_output_area(f, terminal_chunks[0], app);
    
    // Render input area
    render_input_area(f, terminal_chunks[1], app);
}

fn render_output_area(f: &mut Frame, area: Rect, app: &mut App) {
    let output_lines = if app.output_history.is_empty() {
        match app.mode {
            AppMode::CombatTrackerTUI => {
                vec![
                    "⚔️ Combat Tracker - Interactive Mode ⚔️".to_string(),
                    "".to_string(),
                    format!("State: {}", app.current_state),
                    "".to_string(),
                    "Type 'help' for available commands".to_string(),
                    "Type 'init' to initialize combat".to_string(),
                    "".to_string(),
                ]
            },
            AppMode::SearchTUI => {
                vec![
                    "🔍 D&D 5e Search - Interactive Mode 🔍".to_string(),
                    "".to_string(),
                    format!("State: {}", app.current_state),
                    "".to_string(),
                    "Type 'help' for available commands".to_string(),
                    "Type 'search <query>' to search".to_string(),
                    "Example: search fireball".to_string(),
                    "".to_string(),
                ]
            },
            AppMode::CharacterCreationTUI => {
                vec![
                    "🎭 Character Creation - Interactive Mode 🎭".to_string(),
                    "".to_string(),
                    format!("State: {}", app.current_state),
                    "".to_string(),
                    "Type 'help' for available commands".to_string(),
                    "Type 'create' to start character creation".to_string(),
                    "".to_string(),
                ]
            },
            AppMode::CharacterDisplayTUI => {
                vec![
                    "📋 Character Display - Interactive Mode 📋".to_string(),
                    "".to_string(),
                    format!("State: {}", app.current_state),
                    "".to_string(),
                    "Type 'help' for available commands".to_string(),
                    "Type 'list' to see all characters".to_string(),
                    "".to_string(),
                ]
            },
            AppMode::CharacterDeletionTUI => {
                vec![
                    "🗑️  Character Deletion - Interactive Mode 🗑️".to_string(),
                    "".to_string(),
                    format!("State: {}", app.current_state),
                    "".to_string(),
                    "Type 'help' for available commands".to_string(),
                    "Type 'list' to see characters to delete".to_string(),
                    "⚠️  Warning: Deletions are permanent!".to_string(),
                    "".to_string(),
                ]
            },
            AppMode::InitiativeTrackerTUI => {
                vec![
                    "⚡ Initiative Tracker - Interactive Mode ⚡".to_string(),
                    "".to_string(),
                    format!("State: {}", app.current_state),
                    "".to_string(),
                    "Type 'help' for available commands".to_string(),
                    "Type 'roll <name>' to roll initiative".to_string(),
                    "".to_string(),
                ]
            },
            AppMode::NpcGeneratorTUI => {
                vec![
                    "🎭 NPC Generator - Interactive Mode 🎭".to_string(),
                    "".to_string(),
                    format!("State: {}", app.current_state),
                    "".to_string(),
                    "Type 'help' for available commands".to_string(),
                    "Type 'random' to generate a random NPC".to_string(),
                    "Type 'custom <race> <class>' for custom NPC".to_string(),
                    "".to_string(),
                ]
            },
            AppMode::EncounterRandomizerTUI => {
                vec![
                    "⚔️  Encounter Randomizer - Interactive Mode ⚔️".to_string(),
                    "".to_string(),
                    format!("State: {}", app.current_state),
                    "".to_string(),
                    "Type 'help' for available commands".to_string(),
                    "Type 'generate <CR>' to create encounter (e.g., generate 5)".to_string(),
                    "Type 'generate <CR> <count>' for multiple creatures".to_string(),
                    "Type 'examples' for CR guidelines".to_string(),
                    "".to_string(),
                ]
            },
            AppMode::DiceTUI => {
                vec![
                    "🎲 Dice Roller - Interactive Mode 🎲".to_string(),
                    "".to_string(),
                    format!("State: {}", app.current_state),
                    "".to_string(),
                    "Type 'help' for available commands".to_string(),
                    "Type 'roll 1d20' to roll dice".to_string(),
                    "Examples: roll 2d6+3, roll 4d8, advantage, disadvantage".to_string(),
                    "".to_string(),
                ]
            },
            _ => vec![format!("State: {}", app.current_state)],
        }
    } else {
        // Show recent output with scrolling, but add state header
        let mut lines = vec![format!("State: {} {}", app.current_state,
            if let Some(ref waiting) = app.waiting_for {
                format!("(Waiting: {})", waiting)
            } else {
                "".to_string()
            }
        ), "".to_string()];
        
        let start_index = app.scroll_offset;
        let end_index = std::cmp::min(
            app.output_history.len(),
            start_index + (area.height as usize).saturating_sub(4) // Leave room for state header
        );
        
        if start_index < app.output_history.len() {
            lines.extend_from_slice(&app.output_history[start_index..end_index]);
        } else {
            lines.extend_from_slice(&app.output_history);
        }
        
        lines
    };

    let output_text = output_lines.join("\n");
    let output_paragraph = Paragraph::new(output_text)
        .style(Style::default().fg(TEXT_COLOR))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR))
                .style(Style::default().bg(BACKGROUND_COLOR))
                .title("Output")
        );
    
    f.render_widget(output_paragraph, area);
}

fn render_input_area(f: &mut Frame, area: Rect, app: &mut App) {
    let input_text = format!("> {}", app.input_buffer);
    
    let input_paragraph = Paragraph::new(input_text)
        .style(Style::default().fg(TEXT_COLOR))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(SELECTED_COLOR))  // Highlight input area
                .style(Style::default().bg(MENU_COLOR))
                .title("Command Input")
        );
    
    f.render_widget(input_paragraph, area);
}

fn get_title_for_mode(mode: &AppMode) -> Text {
    let title = match mode {
        AppMode::MainMenu => "🎲 D&D Tools - Main Menu 🎲",
        AppMode::CharactersMenu => "👥 Characters Menu 👥",
        AppMode::ToolsMenu => "🛠️  Tools Menu 🛠️",
        AppMode::CharacterCreation => "✨ Character Creation ✨",
        AppMode::CharacterCreationTUI => "✨ Character Creation (Interactive) ✨",
        AppMode::CharacterDisplay => "📋 Character Display 📋",
        AppMode::CharacterDisplayTUI => "📋 Character Display (Interactive) 📋",
        AppMode::CharacterDeletion => "🗑️  Character Deletion 🗑️",
        AppMode::CharacterDeletionTUI => "🗑️  Character Deletion (Interactive) 🗑️",
        AppMode::InitiativeTracker => "⚡ Initiative Tracker ⚡",
        AppMode::InitiativeTrackerTUI => "⚡ Initiative Tracker (Interactive) ⚡",
        AppMode::NpcGenerator => "🎭 NPC Generator 🎭",
        AppMode::NpcGeneratorTUI => "🎭 NPC Generator (Interactive) 🎭",
        AppMode::EncounterRandomizer => "⚔️  Encounter Randomizer ⚔️",
        AppMode::EncounterRandomizerTUI => "⚔️  Encounter Randomizer (Interactive) ⚔️",
        AppMode::Dice => "🎲 Dice Roller 🎲",
        AppMode::DiceTUI => "🎲 Dice Roller (Interactive) 🎲",
        AppMode::CombatTracker => "⚔️  Combat Tracker ⚔️",
        AppMode::CombatTrackerTUI => "⚔️  Combat Tracker (Interactive) ⚔️",
        AppMode::Search => "🔍 D&D 5e Search 🔍",
        AppMode::SearchTUI => "🔍 D&D 5e Search (Interactive) 🔍",
        AppMode::Exit => "👋 Goodbye! 👋",
    };
    Text::from(title)
}

fn get_help_text(mode: &AppMode) -> Text {
    let help = match mode {
        AppMode::MainMenu | AppMode::CharactersMenu | AppMode::ToolsMenu => 
            "↑↓ Navigate • Enter Select • Esc Back • Ctrl+Q Quit",
        AppMode::CombatTrackerTUI | AppMode::SearchTUI | AppMode::CharacterCreationTUI 
        | AppMode::CharacterDisplayTUI | AppMode::CharacterDeletionTUI | AppMode::InitiativeTrackerTUI 
        | AppMode::NpcGeneratorTUI | AppMode::EncounterRandomizerTUI | AppMode::DiceTUI => 
            "Type commands • Enter Execute • ↑↓ History • PgUp/PgDn Scroll • Esc Back • Ctrl+Q Quit",
        _ => "Press any key to continue...",
    };
    Text::from(help)
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

// TUI-compatible wrapper functions that call the main CLI functions
fn npc_randomizer_tui_mode() {
    super::npc_randomizer_mode();
}

fn combat_tracker_tui_mode() {
    super::combat_tracker_mode();
}

fn search_tui_mode() {
    super::search_mode();
}