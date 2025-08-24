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
            AppMode::ToolsMenu => vec!["Initiative tracker", "NPC randomizer", "Dice", "Combat tracker", "Search D&D 5e API", "Back to main menu"],
            _ => vec![],
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match self.mode {
            AppMode::CombatTrackerTUI | AppMode::SearchTUI | AppMode::CharacterCreationTUI 
            | AppMode::CharacterDisplayTUI | AppMode::CharacterDeletionTUI | AppMode::InitiativeTrackerTUI 
            | AppMode::NpcGeneratorTUI | AppMode::DiceTUI => {
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
                    2 => self.mode = AppMode::DiceTUI,
                    3 => self.mode = AppMode::CombatTrackerTUI,
                    4 => self.mode = AppMode::SearchTUI,
                    5 => {
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
            AppMode::InitiativeTracker | AppMode::NpcGenerator | AppMode::Dice | AppMode::CombatTracker | AppMode::Search 
            | AppMode::InitiativeTrackerTUI | AppMode::NpcGeneratorTUI | AppMode::DiceTUI => {
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
                self.add_output("⚔️ Enhanced Combat Mode Commands:".to_string());
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
                if let Some(ref tracker) = self.combat_tracker {
                    if tracker.combatants.is_empty() {
                        self.add_output("❌ No combatants in combat.".to_string());
                    } else {
                        let _old_turn = tracker.current_turn;
                        let new_turn = (tracker.current_turn + 1) % tracker.combatants.len();
                        
                        let mut messages = Vec::new();
                        if new_turn == 0 {
                            let new_round = tracker.round_number + 1;
                            messages.push(format!("🔄 Starting Round {}", new_round));
                        }
                        
                        let current_combatant = tracker.combatants[new_turn].clone();
                        messages.push(format!("🎯 It's {}'s turn! (Initiative: {}, HP: {}/{})", 
                            current_combatant.name, current_combatant.initiative, 
                            current_combatant.current_hp, current_combatant.max_hp));
                        
                        // Now update the tracker
                        if let Some(ref mut tracker) = self.combat_tracker {
                            tracker.current_turn = new_turn;
                            if new_turn == 0 {
                                tracker.round_number += 1;
                            }
                        }
                        
                        for message in messages {
                            self.add_output(message);
                        }
                        
                        // Display combat contact card for current character
                        self.display_combat_contact_card(&current_combatant);
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
                self.add_output("🎲 Enhanced Dice Roller Commands:".to_string());
                self.add_output("".to_string());
                self.add_output("📊 BASIC ROLLS:".to_string());
                self.add_output("  roll <dice> - Roll dice with ASCII art and colors".to_string());
                self.add_output("    Examples: roll 1d20, roll 2d6+3, roll 4d8-1".to_string());
                self.add_output("  advantage - Roll with advantage (2d20, keep higher)".to_string());
                self.add_output("  disadvantage - Roll with disadvantage (2d20, keep lower)".to_string());
                self.add_output("".to_string());
                self.add_output("🎨 FEATURES:".to_string());
                self.add_output("  • ASCII art for dice (d4-triangle, d6-square, d8-hexagon, etc.)".to_string());
                self.add_output("  • Color coding: Red(low) → Yellow(mid) → Green(high)".to_string());
                self.add_output("  • Special colors: Black(1), Gold(natural 20)".to_string());
                self.add_output("  • Proper modifier handling: dice first, then add/subtract".to_string());
                self.add_output("".to_string());
                self.add_output("📋 OTHER COMMANDS:".to_string());
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
        self.add_output("".to_string());
        self.add_output("╔═══════════════════════════════════════════════════════════════════════════════╗".to_string());
        self.add_output(format!("║ 📋 {} - COMPLETE CHARACTER SHEET{} ║", 
            character.name, 
            " ".repeat(47_i32.saturating_sub(character.name.len() as i32) as usize)
        ));
        self.add_output("╠═══════════════════════════════════════════════════════════════════════════════╣".to_string());
        
        // Basic Info Section
        self.add_output("║ 🎭 BASIC INFORMATION                                                         ║".to_string());
        self.add_output("╠───────────────────────────────────────────────────────────────────────────────╣".to_string());
        self.add_output(format!("║ Level: {:<15} Class: {:<15} Race: {:<15} ║", 
            character.level.map(|l| l.to_string()).unwrap_or("N/A".to_string()),
            character.class.as_ref().unwrap_or(&"Unknown".to_string()),
            character.race.as_ref().unwrap_or(&"Unknown".to_string())
        ));
        
        // Ability Scores Section
        self.add_output("╠───────────────────────────────────────────────────────────────────────────────╣".to_string());
        self.add_output("║ 💪 ABILITY SCORES                                                            ║".to_string());
        self.add_output("╠───────────────────────────────────────────────────────────────────────────────╣".to_string());
        
        let str_display = character.stre.map(|v| format!("{} ({})", v, character.get_strength_modifier())).unwrap_or("N/A".to_string());
        let dex_display = character.dext.map(|v| format!("{} ({})", v, character.get_dexterity_modifier())).unwrap_or("N/A".to_string());
        let con_display = character.cons.map(|v| format!("{} ({})", v, character.get_constitution_modifier())).unwrap_or("N/A".to_string());
        
        self.add_output(format!("║ STR: {:<12} DEX: {:<12} CON: {:<12} ║", str_display, dex_display, con_display));
        
        let int_display = character.intl.map(|v| format!("{} ({})", v, character.get_intelligence_modifier())).unwrap_or("N/A".to_string());
        let wis_display = character.wisd.map(|v| format!("{} ({})", v, character.get_wisdom_modifier())).unwrap_or("N/A".to_string());
        let cha_display = character.chas.map(|v| format!("{} ({})", v, character.get_charisma_modifier())).unwrap_or("N/A".to_string());
        
        self.add_output(format!("║ INT: {:<12} WIS: {:<12} CHA: {:<12} ║", int_display, wis_display, cha_display));
        
        // Combat Stats Section
        self.add_output("╠───────────────────────────────────────────────────────────────────────────────╣".to_string());
        self.add_output("║ ⚔️ COMBAT STATISTICS                                                          ║".to_string());
        self.add_output("╠───────────────────────────────────────────────────────────────────────────────╣".to_string());
        
        let hp_display = character.hp.map(|h| h.to_string()).unwrap_or("N/A".to_string());
        let ac_display = character.ac.map(|a| a.to_string()).unwrap_or("N/A".to_string());
        let speed_display = character.speed.map(|s| format!("{} ft", s)).unwrap_or("N/A".to_string());
        let init_display = character.initiative.map(|i| i.to_string()).unwrap_or("N/A".to_string());
        
        self.add_output(format!("║ HP: {:<8} AC: {:<8} Speed: {:<8} Initiative: {:<8} ║", hp_display, ac_display, speed_display, init_display));
        
        // Skills and Proficiencies Section
        self.add_output("╠───────────────────────────────────────────────────────────────────────────────╣".to_string());
        self.add_output("║ 🎯 SKILLS & PROFICIENCIES                                                    ║".to_string());
        self.add_output("╠───────────────────────────────────────────────────────────────────────────────╣".to_string());
        
        let prof_bonus = character.prof_bonus.map(|p| format!("+{}", p)).unwrap_or("N/A".to_string());
        let pass_perc = character.passive_perception.map(|p| p.to_string()).unwrap_or("Auto-calc".to_string());
        
        self.add_output(format!("║ Proficiency Bonus: {:<10} Passive Perception: {:<10}           ║", prof_bonus, pass_perc));
        
        // Equipment Section  
        self.add_output("╠───────────────────────────────────────────────────────────────────────────────╣".to_string());
        self.add_output("║ 🎒 INVENTORY & SPELLS                                                        ║".to_string());
        self.add_output("╠───────────────────────────────────────────────────────────────────────────────╣".to_string());
        
        if !character.inventory.is_empty() {
            let inventory_display = character.inventory.join(", ");
            let inventory_short = if inventory_display.len() > 60 { 
                format!("{}...", &inventory_display[..57])
            } else { 
                inventory_display
            };
            self.add_output(format!("║ Inventory: {:<64} ║", inventory_short));
        } else {
            self.add_output("║ Inventory: Empty                                                            ║".to_string());
        }
        
        // Spells Section
        if !character.spells.is_empty() {
            let spells_display = character.spells.join(", ");
            let spells_short = if spells_display.len() > 60 { 
                format!("{}...", &spells_display[..57])
            } else { 
                spells_display 
            };
            self.add_output(format!("║ Spells: {:<67} ║", spells_short));
        } else {
            self.add_output("║ Spells: None                                                                ║".to_string());
        }
        
        // Description/Notes Section
        if let Some(ref desc) = character.desc {
            self.add_output("╠───────────────────────────────────────────────────────────────────────────────╣".to_string());
            self.add_output("║ 📝 DESCRIPTION                                                               ║".to_string());
            self.add_output("╠───────────────────────────────────────────────────────────────────────────────╣".to_string());
            let desc_short = if desc.len() > 60 { 
                format!("{}...", &desc[..57])
            } else { 
                desc.clone() 
            };
            self.add_output(format!("║ {:<77} ║", desc_short));
        }
        
        self.add_output("╚═══════════════════════════════════════════════════════════════════════════════╝".to_string());
        self.add_output("".to_string());
    }

    fn display_combat_contact_card(&mut self, combatant: &crate::combat::Combatant) {
        self.add_output("".to_string());
        self.add_output("┌─────────────────────── COMBAT CONTACT CARD ──────────────────────────┐".to_string());
        self.add_output(format!("│ 🎭 {} {}", combatant.name, " ".repeat(67_i32.saturating_sub(combatant.name.len() as i32) as usize)));
        self.add_output("├─────────────────────────────────────────────────────────────────────┤".to_string());
        
        // Core Combat Stats
        let hp_status = if combatant.current_hp <= combatant.max_hp / 4 {
            "🩸 BLOODIED"
        } else if combatant.current_hp == 0 {
            "💀 DOWN"
        } else {
            "❤️ HEALTHY"
        };
        
        self.add_output(format!("│ HP: {}/{} {} │", 
            combatant.current_hp, combatant.max_hp, 
            hp_status
        ));
        self.add_output(format!("│ AC: {} │", combatant.ac));
        self.add_output(format!("│ Initiative: {} │", combatant.initiative));
        
        // Temporary HP if any
        if combatant.temp_hp > 0 {
            self.add_output(format!("│ Temp HP: {} │", combatant.temp_hp));
        }
        
        // Status Effects
        if !combatant.status_effects.is_empty() {
            self.add_output("├─────────────────────────────────────────────────────────────────────┤".to_string());
            self.add_output("│ 🎯 ACTIVE STATUS EFFECTS: │".to_string());
            for effect in &combatant.status_effects {
                let duration_text = if let Some(duration) = effect.duration {
                    format!(" ({} rounds)", duration)
                } else {
                    " (permanent)".to_string()
                };
                self.add_output(format!("│   • {}{} │", effect.name, duration_text));
            }
        }
        
        // Try to get full character stats if it's a player character
        if let Some(character) = self.characters.iter().find(|c| c.name == combatant.name) {
            // Extract all data first to avoid borrowing issues
            let str_mod = character.get_strength_modifier();
            let dex_mod = character.get_dexterity_modifier();
            let con_mod = character.get_constitution_modifier();
            let int_mod = character.get_intelligence_modifier();
            let wis_mod = character.get_wisdom_modifier();
            let cha_mod = character.get_charisma_modifier();
            let prof_bonus = character.prof_bonus;
            let speed = character.speed;
            
            // Now use the extracted data
            self.add_output("├─────────────────────────────────────────────────────────────────────┤".to_string());
            self.add_output("│ 📊 ABILITY MODIFIERS: │".to_string());
            
            self.add_output(format!("│   STR: {} │ DEX: {} │ CON: {} │ INT: {} │ WIS: {} │ CHA: {} │", 
                str_mod, dex_mod, con_mod, int_mod, wis_mod, cha_mod));
            
            if let Some(prof_bonus) = prof_bonus {
                self.add_output(format!("│ Proficiency Bonus: +{} │", prof_bonus));
            }
            
            if let Some(speed) = speed {
                self.add_output(format!("│ Speed: {} ft │", speed));
            }
        }
        
        self.add_output("└─────────────────────────────────────────────────────────────────────┘".to_string());
        self.add_output("💡 Quick Combat Reference - Type 'help' for available actions".to_string());
        self.add_output("".to_string());
    }

    fn generate_random_npc(&mut self) {
        use crate::races_classes::{get_random_race, get_random_class};
        
        self.add_output("🎲 Generating comprehensive random NPC...".to_string());
        
        let race = get_random_race();
        let class = get_random_class();
        
        // Generate all stats
        let ac = (rand::random::<u8>() % 11) + 10; // 10-20
        let hp = (rand::random::<u8>() % 41) + 10; // 10-50
        let speed = ((rand::random::<u8>() % 7) + 2) * 10; // 20-80
        let level = (rand::random::<u8>() % 10) + 1; // 1-10
        
        // Generate ability scores (rolling 4d6 drop lowest)
        let mut abilities = Vec::new();
        for _ in 0..6 {
            let mut rolls = vec![];
            for _ in 0..4 {
                rolls.push((rand::random::<u8>() % 6) + 1);
            }
            rolls.sort_by(|a, b| b.cmp(a)); // Sort descending
            let total: u8 = rolls[0] + rolls[1] + rolls[2]; // Take top 3
            abilities.push(total);
        }
        
        let (str_score, dex_score, con_score, int_score, wis_score, cha_score) = 
            (abilities[0], abilities[1], abilities[2], abilities[3], abilities[4], abilities[5]);
        
        // Calculate modifiers
        let str_mod = ((str_score as i32) - 10) / 2;
        let dex_mod = ((dex_score as i32) - 10) / 2;
        let con_mod = ((con_score as i32) - 10) / 2;
        let int_mod = ((int_score as i32) - 10) / 2;
        let wis_mod = ((wis_score as i32) - 10) / 2;
        let cha_mod = ((cha_score as i32) - 10) / 2;
        
        let prof_bonus = ((level - 1) / 4) + 2; // Standard proficiency progression
        let passive_perception = 10 + wis_mod + prof_bonus as i32;
        
        self.add_output("".to_string());
        self.add_output("╔═══════════════════════════════════════════════════════════════════════════════╗".to_string());
        self.add_output("║ 🎭 COMPREHENSIVE GENERATED NPC                                               ║".to_string());
        self.add_output("╠═══════════════════════════════════════════════════════════════════════════════╣".to_string());
        self.add_output(format!("║ Race: {:<15} Class: {:<15} Level: {:<15} ║", race, class, level));
        self.add_output("╠───────────────────────────────────────────────────────────────────────────────╣".to_string());
        self.add_output("║ 💪 ABILITY SCORES                                                            ║".to_string());
        self.add_output("╠───────────────────────────────────────────────────────────────────────────────╣".to_string());
        self.add_output(format!("║ STR: {} ({:+})      DEX: {} ({:+})      CON: {} ({:+})               ║", 
            str_score, str_mod, dex_score, dex_mod, con_score, con_mod));
        self.add_output(format!("║ INT: {} ({:+})      WIS: {} ({:+})      CHA: {} ({:+})               ║", 
            int_score, int_mod, wis_score, wis_mod, cha_score, cha_mod));
        self.add_output("╠───────────────────────────────────────────────────────────────────────────────╣".to_string());
        self.add_output("║ ⚔️ COMBAT STATISTICS                                                          ║".to_string());
        self.add_output("╠───────────────────────────────────────────────────────────────────────────────╣".to_string());
        self.add_output(format!("║ HP: {:<8} AC: {:<8} Speed: {} ft{:<8} Initiative: {:+}{}  ║", 
            hp, ac, speed, "", dex_mod, " ".repeat(6)));
        self.add_output(format!("║ Proficiency Bonus: +{:<5} Passive Perception: {:<10}          ║", 
            prof_bonus, passive_perception));
        self.add_output("╚═══════════════════════════════════════════════════════════════════════════════╝".to_string());
        self.add_output("💡 This NPC has complete D&D 5e stats ready for use!".to_string());
    }

    fn generate_custom_npc(&mut self, race: &str, class: &str) {
        self.add_output(format!("🎲 Generating comprehensive {} {}...", race, class));
        
        // Generate all stats
        let ac = (rand::random::<u8>() % 11) + 10; // 10-20
        let hp = (rand::random::<u8>() % 41) + 10; // 10-50
        let speed = ((rand::random::<u8>() % 7) + 2) * 10; // 20-80
        let level = (rand::random::<u8>() % 10) + 1; // 1-10
        
        // Generate ability scores (rolling 4d6 drop lowest)
        let mut abilities = Vec::new();
        for _ in 0..6 {
            let mut rolls = vec![];
            for _ in 0..4 {
                rolls.push((rand::random::<u8>() % 6) + 1);
            }
            rolls.sort_by(|a, b| b.cmp(a)); // Sort descending
            let total: u8 = rolls[0] + rolls[1] + rolls[2]; // Take top 3
            abilities.push(total);
        }
        
        let (str_score, dex_score, con_score, int_score, wis_score, cha_score) = 
            (abilities[0], abilities[1], abilities[2], abilities[3], abilities[4], abilities[5]);
        
        // Calculate modifiers
        let str_mod = ((str_score as i32) - 10) / 2;
        let dex_mod = ((dex_score as i32) - 10) / 2;
        let con_mod = ((con_score as i32) - 10) / 2;
        let int_mod = ((int_score as i32) - 10) / 2;
        let wis_mod = ((wis_score as i32) - 10) / 2;
        let cha_mod = ((cha_score as i32) - 10) / 2;
        
        let prof_bonus = ((level - 1) / 4) + 2; // Standard proficiency progression
        let passive_perception = 10 + wis_mod + prof_bonus as i32;
        
        self.add_output("".to_string());
        self.add_output("╔═══════════════════════════════════════════════════════════════════════════════╗".to_string());
        self.add_output("║ 🎭 COMPREHENSIVE CUSTOM NPC                                                  ║".to_string());
        self.add_output("╠═══════════════════════════════════════════════════════════════════════════════╣".to_string());
        self.add_output(format!("║ Race: {:<15} Class: {:<15} Level: {:<15} ║", race, class, level));
        self.add_output("╠───────────────────────────────────────────────────────────────────────────────╣".to_string());
        self.add_output("║ 💪 ABILITY SCORES                                                            ║".to_string());
        self.add_output("╠───────────────────────────────────────────────────────────────────────────────╣".to_string());
        self.add_output(format!("║ STR: {} ({:+})      DEX: {} ({:+})      CON: {} ({:+})               ║", 
            str_score, str_mod, dex_score, dex_mod, con_score, con_mod));
        self.add_output(format!("║ INT: {} ({:+})      WIS: {} ({:+})      CHA: {} ({:+})               ║", 
            int_score, int_mod, wis_score, wis_mod, cha_score, cha_mod));
        self.add_output("╠───────────────────────────────────────────────────────────────────────────────╣".to_string());
        self.add_output("║ ⚔️ COMBAT STATISTICS                                                          ║".to_string());
        self.add_output("╠───────────────────────────────────────────────────────────────────────────────╣".to_string());
        self.add_output(format!("║ HP: {:<8} AC: {:<8} Speed: {} ft{:<8} Initiative: {:+}{}  ║", 
            hp, ac, speed, "", dex_mod, " ".repeat(6)));
        self.add_output(format!("║ Proficiency Bonus: +{:<5} Passive Perception: {:<10}          ║", 
            prof_bonus, passive_perception));
        self.add_output("╚═══════════════════════════════════════════════════════════════════════════════╝".to_string());
        self.add_output("💡 This custom NPC has complete D&D 5e stats ready for use!".to_string());
    }

    fn roll_dice_with_display(&mut self, dice_expr: &str) {
        match crate::dice::roll_dice_with_crits(dice_expr) {
            Ok((rolls, total, crit_message)) => {
                self.add_output("".to_string());
                self.add_output("┌─────────────────────────────────┐".to_string());
                self.add_output("│         🎲 DICE ROLL! 🎲         │".to_string());
                self.add_output("├─────────────────────────────────┤".to_string());
                self.add_output(format!("│ Expression: {:<19} │", dice_expr));
                
                // Extract dice type for ASCII art
                let dice_type = if let Some(d_pos) = dice_expr.find('d') {
                    let after_d = &dice_expr[d_pos + 1..];
                    let sides_str = after_d.chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect::<String>();
                    sides_str.parse::<u8>().unwrap_or(6)
                } else {
                    6
                };
                
                // Display ASCII art for each die (limit to 3 dice for space)
                if rolls.len() <= 3 {
                    self.add_output("├─────────────────────────────────┤".to_string());
                    
                    for (i, &roll) in rolls.iter().enumerate() {
                        let ascii_art = crate::dice::get_dice_ascii_art(dice_type, roll);
                        let color = crate::dice::get_dice_color_code(roll, dice_type);
                        let reset = crate::dice::reset_color();
                        
                        self.add_output(format!("│ Die #{} (d{}):{}{}{}│", 
                            i + 1, dice_type, 
                            " ".repeat(19 - format!("Die #{} (d{}):", i + 1, dice_type).len()),
                            color, reset
                        ));
                        
                        for line in ascii_art {
                            let padded_line = format!("{}{}{}", color, line, reset);
                            let clean_line_len = line.len();
                            let padding = if clean_line_len < 31 { 31 - clean_line_len } else { 0 };
                            self.add_output(format!("│{}{}{} │", 
                                padded_line, 
                                " ".repeat(padding),
                                color
                            ));
                        }
                    }
                } else {
                    // For many dice, just show the values with colors
                    let mut colored_rolls = Vec::new();
                    for &roll in &rolls {
                        let color = crate::dice::get_dice_color_code(roll, dice_type);
                        let reset = crate::dice::reset_color();
                        colored_rolls.push(format!("{}{}{}", color, roll, reset));
                    }
                    self.add_output(format!("│ Rolls: {:<22} │", colored_rolls.join(", ")));
                }
                
                self.add_output("├─────────────────────────────────┤".to_string());
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
                                    self.add_output("┌─ Quick Reference ─────────────────┐".to_string());
                                    self.add_output(format!("│ 📝 {} - {}", result.name(), result.page.content_type.to_uppercase()));
                                    self.add_output("├───────────────────────────────────┤".to_string());
                                    
                                    // Display key info only (first 8 lines)
                                    let content_lines: Vec<&str> = result.page.content.lines().collect();
                                    for line in content_lines.iter().take(8) {
                                        let trimmed = line.trim();
                                        if !trimmed.is_empty() {
                                            if trimmed.contains(':') && trimmed.len() < 60 {
                                                self.add_output(format!("│ 📊 {}", trimmed));
                                            } else {
                                                self.add_output(format!("│   {}", trimmed));
                                            }
                                        }
                                    }
                                    
                                    if content_lines.len() > 8 {
                                        self.add_output("│ ... (use search mode for full details)".to_string());
                                    }
                                    
                                    self.add_output("└───────────────────────────────────┘".to_string());
                                    
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
                                        self.add_output(format!("┌─ Result {} ─────────────────────────────┐", i + 1));
                                    } else {
                                        self.add_output("┌─ Search Result ─────────────────────────┐".to_string());
                                    }
                                    
                                    // Header with name and type in a nice format
                                    let name = result.name();
                                    let content_type = result.page.content_type.to_uppercase();
                                    self.add_output(format!("│ 📝 {} - {} ", name, content_type));
                                    self.add_output("├─────────────────────────────────────────┤".to_string());
                                    
                                    // URL source  
                                    self.add_output(format!("│ 🔗 Source: {}", result.page.url));
                                    self.add_output("├─────────────────────────────────────────┤".to_string());
                                    
                                    // Format content in readable columns
                                    self.format_search_content_for_tui(&result.page.content);
                                    
                                    self.add_output("└─────────────────────────────────────────┘".to_string());
                                    
                                    // Attribution footer
                                    self.add_output("📄 Source: dnd5e.wikidot.com | CC BY-SA 3.0".to_string());
                                    self.add_output("ℹ️  Educational use - see license at link above".to_string());
                                    
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

    fn format_search_content_for_tui(&mut self, content: &str) {
        let lines: Vec<&str> = content.lines().collect();
        let max_lines = 25; // Limit content to keep it readable
        
        for (line_num, line) in lines.iter().enumerate() {
            if line_num >= max_lines {
                self.add_output(format!("│ ... ({} more lines) [scroll or CLI for full]", lines.len() - max_lines));
                break;
            }
            
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            
            // Format different types of content
            if self.is_stat_line(trimmed) {
                // Format as stat line with icon
                self.add_output(format!("│ 📊 {}", trimmed));
            } else if self.is_heading_line(trimmed) {
                // Format as heading with separator
                self.add_output(format!("│ 🔸 {}", trimmed.to_uppercase()));
                self.add_output(format!("│ {}", "─".repeat(trimmed.len().min(35))));
            } else if trimmed.len() > 80 {
                // Wrap long lines
                let wrapped = self.wrap_content_line(trimmed, 75);
                for wrapped_line in wrapped {
                    self.add_output(format!("│   {}", wrapped_line));
                }
            } else {
                // Regular content line
                self.add_output(format!("│   {}", trimmed));
            }
        }
    }
    
    fn is_stat_line(&self, line: &str) -> bool {
        // Lines that look like "Casting Time: 1 action" or "Range: 150 feet"
        line.contains(':') && line.len() < 60 && line.split(':').count() == 2
    }
    
    fn is_heading_line(&self, line: &str) -> bool {
        // Simple heuristics for headings - short lines that are likely titles
        line.len() < 50 && 
        (line.ends_with(':') || 
         line.chars().all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '\'' || c == '-') &&
         line.split_whitespace().count() <= 5)
    }
    
    fn wrap_content_line(&self, text: &str, max_width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        
        for word in text.split_whitespace() {
            if current_line.len() + word.len() + 1 > max_width {
                if !current_line.is_empty() {
                    lines.push(current_line);
                    current_line = String::new();
                }
            }
            
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
        
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        
        lines
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
        | AppMode::NpcGeneratorTUI | AppMode::DiceTUI => {
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
        | AppMode::NpcGeneratorTUI | AppMode::DiceTUI => 
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