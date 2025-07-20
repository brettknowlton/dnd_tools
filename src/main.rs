use std::{
    collections::HashMap, fs, io::{self, Write}, path::Path
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Cards {
    suit: Suit,
    rank: u8,
    desc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Character {
    name: String,
    level: Option<u8>,

    desc: Option<String>,

    ac: Option<u8>,
    hp: Option<u8>,
    max_hp: Option<u8>,
    temp_hp: Option<u8>,
    speed: Option<u8>,

    intl: Option<u8>,
    wisd: Option<u8>,
    chas: Option<u8>,
    stre: Option<u8>,
    dext: Option<u8>,
    cons: Option<u8>,

    passive_perception: Option<u8>,
    initiative: Option<u8>,
    prof_bonus: Option<u8>,

    inventory: Vec<String>,

    cards: Vec<Cards>,
    spells: Vec<String>,
}
impl Character {
    fn new(name: &str) -> Character {
        Character {
            name: name.to_string(),
            level: None,
            desc: None,
            ac: None,
            hp: None,
            max_hp: None,
            temp_hp: None,
            speed: None,
            intl: None,
            wisd: None,
            chas: None,
            stre: None,
            dext: None,
            cons: None,
            passive_perception: None,
            initiative: None,
            prof_bonus: None,
            inventory: Vec::new(),
            cards: Vec::new(),
            spells: Vec::new(),
        }
    }

    fn get_value(&self, key: String) -> String {
        match key.as_str() {
            "name" => self.name.clone(),
            "level" => self.level.unwrap_or(0).to_string(),
            "desc" => self.desc.clone().unwrap_or("".to_string()),
            "ac" => self.ac.unwrap_or(0).to_string(),
            "hp" => self.hp.unwrap_or(0).to_string(),
            "max_hp" => self.max_hp.unwrap_or(0).to_string(),
            "temp_hp" => self.temp_hp.unwrap_or(0).to_string(),
            "speed" => self.speed.unwrap_or(0).to_string(),
            "intl" => self.intl.unwrap_or(0).to_string(),
            "wisd" => self.wisd.unwrap_or(0).to_string(),
            "chas" => self.chas.unwrap_or(0).to_string(),
            "stre" => self.stre.unwrap_or(0).to_string(),
            "dext" => self.dext.unwrap_or(0).to_string(),
            "cons" => self.cons.unwrap_or(0).to_string(),
            "passive_perception" => self.passive_perception.unwrap_or(0).to_string(),
            "initiative" => self.initiative.unwrap_or(0).to_string(),
            "prof_bonus" => self.prof_bonus.unwrap_or(0).to_string(),
            _ => "".to_string(),
        }
    }

    fn get_ordered_stats(&self) -> Vec<String> {
        let mut stats = Vec::new();
        stats.push(format!("Name: {}", self.name));
        stats.push(format!("Level: {}", self.level.unwrap_or(0)));
        stats.push(format!(
            "Description: {}",
            self.desc.clone().unwrap_or("".to_string())
        ));
        stats.push(format!("AC: {}", self.ac.unwrap_or(0)));
        stats.push(format!("HP: {}", self.hp.unwrap_or(0)));
        stats.push(format!("Max HP: {}", self.max_hp.unwrap_or(0)));
        stats.push(format!("Temp HP: {}", self.temp_hp.unwrap_or(0)));
        stats.push(format!("Speed: {}", self.speed.unwrap_or(0)));
        stats.push(format!("Intelligence: {}", self.intl.unwrap_or(0)));
        stats.push(format!("Wisdom: {}", self.wisd.unwrap_or(0)));
        stats.push(format!("Charisma: {}", self.chas.unwrap_or(0)));
        stats.push(format!("Strength: {}", self.stre.unwrap_or(0)));
        stats.push(format!("Dexterity: {}", self.dext.unwrap_or(0)));
        stats.push(format!("Constitution: {}", self.cons.unwrap_or(0)));
        stats.push(format!(
            "Passive Perception: {}",
            self.passive_perception.unwrap_or(0)
        ));
        stats.push(format!("Initiative: {}", self.initiative.unwrap_or(0)));
        stats.push(format!(
            "Proficiency Bonus: {}",
            self.prof_bonus.unwrap_or(0)
        ));
        stats
    }

    fn write_to_file(&self) -> io::Result<()> {
        let path = format!("characters/{}.txt", self.name);
        let mut file = fs::File::create(path)?;
        for stat in self.get_ordered_stats() {
            file.write_all(stat.as_bytes())?;
            file.write_all(b"\n")?;
        }
        Ok(())
    }
    fn as_vec(&self) -> Vec<String> {
        let mut vec = Vec::new();
        vec.push(self.name.clone());
        vec.push(self.level.unwrap_or(0).to_string());
        vec.push(self.desc.clone().unwrap_or("".to_string()));
        vec.push(self.ac.unwrap_or(0).to_string());
        vec.push(self.hp.unwrap_or(0).to_string());
        vec.push(self.max_hp.unwrap_or(0).to_string());
        vec.push(self.temp_hp.unwrap_or(0).to_string());
        vec.push(self.speed.unwrap_or(0).to_string());
        vec.push(self.intl.unwrap_or(0).to_string());
        vec.push(self.wisd.unwrap_or(0).to_string());
        vec.push(self.chas.unwrap_or(0).to_string());
        vec.push(self.stre.unwrap_or(0).to_string());
        vec.push(self.dext.unwrap_or(0).to_string());
        vec.push(self.cons.unwrap_or(0).to_string());
        vec.push(self.passive_perception.unwrap_or(0).to_string());
        vec.push(self.initiative.unwrap_or(0).to_string());
        vec.push(self.prof_bonus.unwrap_or(0).to_string());
        vec
    }

    fn as_hashmap(&self) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        map.insert("name".to_string(), self.name.clone());
        map.insert("level".to_string(), self.level.unwrap_or(0).to_string());
        map.insert(
            "desc".to_string(),
            self.desc.clone().unwrap_or("".to_string()),
        );
        map.insert("ac".to_string(), self.ac.unwrap_or(0).to_string());
        map.insert("hp".to_string(), self.hp.unwrap_or(0).to_string());
        map.insert("max_hp".to_string(), self.max_hp.unwrap_or(0).to_string());
        map.insert("temp_hp".to_string(), self.temp_hp.unwrap_or(0).to_string());
        map.insert("speed".to_string(), self.speed.unwrap_or(0).to_string());
        map.insert("intl".to_string(), self.intl.unwrap_or(0).to_string());
        map.insert("wisd".to_string(), self.wisd.unwrap_or(0).to_string());
        map.insert("chas".to_string(), self.chas.unwrap_or(0).to_string());
        map.insert("stre".to_string(), self.stre.unwrap_or(0).to_string());
        map.insert("dext".to_string(), self.dext.unwrap_or(0).to_string());
        map.insert("cons".to_string(), self.cons.unwrap_or(0).to_string());
        map.insert(
            "passive_perception".to_string(),
            self.passive_perception.unwrap_or(0).to_string(),
        );
        map.insert(
            "initiative".to_string(),
            self.initiative.unwrap_or(0).to_string(),
        );
        map.insert(
            "prof_bonus".to_string(),
            self.prof_bonus.unwrap_or(0).to_string(),
        );
        map
    }

    fn apply_hash_changes(
        &mut self,
        changes: std::collections::HashMap<String, String>,
    ) -> Character {
        let mut new_character = self.clone();
        for (key, value) in changes {
            match key.as_str() {
                "name" => new_character.name = value,
                "level" => new_character.level = Some(value.parse().unwrap()),
                "desc" => new_character.desc = Some(value),
                "ac" => new_character.ac = Some(value.parse().unwrap()),
                "hp" => new_character.hp = Some(value.parse().unwrap()),
                "max_hp" => new_character.max_hp = Some(value.parse().unwrap()),
                "temp_hp" => new_character.temp_hp = Some(value.parse().unwrap()),
                "speed" => new_character.speed = Some(value.parse().unwrap()),
                "intl" => new_character.intl = Some(value.parse().unwrap()),
                "wisd" => new_character.wisd = Some(value.parse().unwrap()),
                "chas" => new_character.chas = Some(value.parse().unwrap()),
                "stre" => new_character.stre = Some(value.parse().unwrap()),
                "dext" => new_character.dext = Some(value.parse().unwrap()),
                "cons" => new_character.cons = Some(value.parse().unwrap()),
                "passive_perception" => {
                    new_character.passive_perception = Some(value.parse().unwrap())
                }
                "initiative" => new_character.initiative = Some(value.parse().unwrap()),
                "prof_bonus" => new_character.prof_bonus = Some(value.parse().unwrap()),
                _ => (),
            }
        }
        new_character
    }

    fn apply_vec_changes(self, changes: Vec<String>) -> Character {
        let mut new_character = self.clone();
        new_character.name = changes[0].clone();
        new_character.level = Some(changes[1].parse().unwrap());
        new_character.desc = Some(changes[2].clone());
        new_character.ac = Some(changes[3].parse().unwrap());
        new_character.hp = Some(changes[4].parse().unwrap());
        new_character.max_hp = Some(changes[5].parse().unwrap());
        new_character.temp_hp = Some(changes[6].parse().unwrap());
        new_character.speed = Some(changes[7].parse().unwrap());
        new_character.intl = Some(changes[8].parse().unwrap());
        new_character.wisd = Some(changes[9].parse().unwrap());
        new_character.chas = Some(changes[10].parse().unwrap());
        new_character.stre = Some(changes[11].parse().unwrap());
        new_character.dext = Some(changes[12].parse().unwrap());
        new_character.cons = Some(changes[13].parse().unwrap());
        new_character.passive_perception = Some(changes[14].parse().unwrap());
        new_character.initiative = Some(changes[15].parse().unwrap());
        new_character.prof_bonus = Some(changes[16].parse().unwrap());
        new_character
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Data {
    data: HashMap<char, Vec<String>>
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct InitiativeEntry {
    name: String,
    initiative: i32,
    is_player: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InitiativeTracker {
    entries: Vec<InitiativeEntry>,
    current_turn: usize,
}

impl InitiativeTracker {
    fn new() -> Self {
        InitiativeTracker {
            entries: Vec::new(),
            current_turn: 0,
        }
    }

    fn add_entry(&mut self, name: String, initiative: i32, is_player: bool) {
        let entry = InitiativeEntry {
            name,
            initiative,
            is_player,
        };
        self.entries.push(entry);
        self.sort_by_initiative();
    }

    fn sort_by_initiative(&mut self) {
        self.entries.sort_by(|a, b| b.initiative.cmp(&a.initiative));
        self.current_turn = 0;
    }

    fn next_turn(&mut self) -> Option<&InitiativeEntry> {
        if self.entries.is_empty() {
            return None;
        }
        let current = &self.entries[self.current_turn];
        self.current_turn = (self.current_turn + 1) % self.entries.len();
        Some(current)
    }

    fn display(&self) {
        println!("Initiative Order:");
        for (i, entry) in self.entries.iter().enumerate() {
            let marker = if i == self.current_turn { ">>> " } else { "    " };
            let player_type = if entry.is_player { "(Player)" } else { "(NPC)" };
            println!("{}Initiative {}: {} {}", marker, entry.initiative, entry.name, player_type);
        }
    }

    fn remove_entry(&mut self, name: &str) -> bool {
        if let Some(pos) = self.entries.iter().position(|entry| entry.name == name) {
            self.entries.remove(pos);
            if self.current_turn >= self.entries.len() && !self.entries.is_empty() {
                self.current_turn = 0;
            }
            true
        } else {
            false
        }
    }
}

fn main() -> io::Result<()> {
    println!("Welcome to DnD tools!");
    let mut characters = load_character_files();
    println!("Loaded {} character sheets:", characters.len());
    for character_sheet in characters.clone() {
        println!("{:?}\n", character_sheet);
    }

    let events = Data {
        data: HashMap::new()
    };


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
            "3" => roll_dice_mode(events.clone()),
            "4" => initiative_tracker_mode(),
            "0" => ending = true,

            _ => println!("Invalid input"),
        };
    }
    Ok(())
}

fn roll_dice_mode(_events: Data) {
    let mut ending = false;
    while !ending {
        println!("Options:");
        println!("r(i)d(n) will roll a n sided die i times");
        println!("e - event management");
        println!("q - quit");
        let mut buffer = String::new();
        io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to read line");
        //match on first letter of input
        match buffer.trim().chars().next() {
            Some('r') => {
                roll_dice(&mut buffer);
            }
            Some('e') => {
                add_event(&mut buffer);
            }
            Some('q') => ending = true,
            _ => println!("Invalid input"),
        }
    }
}

fn add_event(buffer: &mut str) {
    let mut buffer = buffer.trim().to_string();
    buffer.remove(0);

    match buffer.trim().chars().next() {
        Some('a') => {
            println!("Event management - add functionality coming soon");
        }
        Some('r') => {
            remove_event_from_file(&mut buffer);
        }
        Some('l') => {
            load_events();
        }
        _ => println!("Invalid input"),
    }

    if buffer.trim().is_empty() {
        return;
    }

    let mut split = buffer.split(" ");
    if let (Some(event), Some(time), Some(desc)) = (split.next(), split.next(), split.next()) {
        println!("Event: {}\nTime: {}\nDescription: {}", event, time, desc);
    }
}

fn remove_event_from_file(_buffer: &mut String) {
    println!("Remove event functionality coming soon");
}

fn load_events() {
    println!("Load events functionality coming soon");
}

fn initiative_tracker_mode() {
    let mut tracker = InitiativeTracker::new();
    let mut ending = false;
    
    println!("Welcome to the Initiative Tracker!");
    println!("Commands: add, remove, next, display, clear, quit");
    
    while !ending {
        println!("\nInitiative Tracker > Enter command:");
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).expect("Failed to read line");
        
        let input = buffer.trim().to_lowercase();
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        match parts.get(0) {
            Some(&"add") => {
                if parts.len() >= 3 {
                    let name = parts[1].to_string();
                    if let Ok(initiative) = parts[2].parse::<i32>() {
                        let is_player = parts.get(3).map_or(true, |&s| s == "player");
                        tracker.add_entry(name, initiative, is_player);
                        println!("Added to initiative tracker!");
                        tracker.display();
                    } else {
                        println!("Invalid initiative value. Please enter a number.");
                    }
                } else {
                    println!("Usage: add <name> <initiative> [player|npc]");
                    println!("Example: add Gandalf 18 player");
                }
            }
            Some(&"remove") => {
                if parts.len() >= 2 {
                    let name = parts[1];
                    if tracker.remove_entry(name) {
                        println!("Removed {} from initiative tracker", name);
                        tracker.display();
                    } else {
                        println!("Could not find {} in initiative tracker", name);
                    }
                } else {
                    println!("Usage: remove <name>");
                }
            }
            Some(&"next") => {
                if let Some(current) = tracker.next_turn() {
                    println!("Current turn: {} (Initiative: {})", current.name, current.initiative);
                    tracker.display();
                } else {
                    println!("No entries in initiative tracker. Use 'add' to add some!");
                }
            }
            Some(&"display") => {
                tracker.display();
            }
            Some(&"clear") => {
                tracker = InitiativeTracker::new();
                println!("Initiative tracker cleared!");
            }
            Some(&"quit") | Some(&"q") => {
                ending = true;
            }
            Some(&"help") | Some(&"h") => {
                println!("Commands:");
                println!("  add <name> <initiative> [player|npc] - Add entry to tracker");
                println!("  remove <name> - Remove entry from tracker");
                println!("  next - Advance to next turn");
                println!("  display - Show current initiative order");
                println!("  clear - Clear all entries");
                println!("  quit - Exit initiative tracker");
            }
            _ => {
                println!("Unknown command. Type 'help' for available commands.");
            }
        }
    }
}


fn roll_dice(buffer: &mut str) {
    let mut buffer = buffer.trim().to_string();
    buffer.remove(0);
    let mut split = buffer.split("d");
    let num = split.next().unwrap().parse::<u8>().unwrap();
    let sides = split.next().unwrap().parse::<u8>().unwrap();
    let mut total = 0;
    for i in 0..num {
        let roll = rand::random::<u8>() % sides + 1;
        total += roll;
        println!("Roll {}: {}", i + 1, roll);
    }
    println!("Rolled d{} x{} times: {}", sides, num, total);
}

fn load_character_files() -> Vec<Character> {
    let mut characters = Vec::new();
    if let Ok(paths) = fs::read_dir("characters") {
        for path in paths {
            if let Ok(path) = path {
                let character_sheet = fs::read_to_string(path.path()).expect("Failed to read file");
                let character = ron::de::from_str::<Character>(&character_sheet).unwrap();
                characters.push(character);
            }
        }
    }
    characters
}

fn create_character() -> Character {
    println!("Creating a new character\nEnter the character's name:");
    let mut buffer = String::new();

    io::stdin()
        .read_line(&mut buffer)
        .expect("Failed to read line");
    let name = buffer.trim();

    let mut character = Character::new(name);
    println!("Character {} created!", name);

    println!("Would you like to add more information to the character sheet?\n1. Yes\n2. No\n");

    let mut input_taken = false;
    while !input_taken {
        let mut buffer = String::new();
        io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to read line");
        match buffer.trim() {
            "1" => {
                println!("Adding more information to the character sheet");
                character = data_entry(character);
                return character;
            }
            "2" => input_taken = true,
            _ => println!("Invalid input"),
        }
    }

    character
}

fn save_characters(characters: Vec<Character>) {
    fn save_character(name: String, data: Character, _characters: &mut Vec<Character>) {
        println!("Saving character sheet for {}", name);

        let path = format!("characters/{}.txt", name);
        let mut file = fs::File::create(path).expect("Failed to create file");
        let serialized =
            ron::ser::to_string_pretty(&data, ron::ser::PrettyConfig::default()).unwrap();

        file.write(serialized.as_bytes())
            .expect("Failed to write to file");

        println!("Character sheet saved!");
    }

    for character in characters {
        save_character(character.name.clone(), character, &mut Vec::new());
    }
}

fn display_character_info() {
    println!("Enter the name of the character you would like to load:");

    let mut buffer = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .expect("Failed to read line");

    let name = buffer.trim();
    println!("Loading character sheet for {}", name);

    let path = format!("characters/{}.txt", name);
    let character_sheet = fs::read_to_string(Path::new(&path)).expect("Failed to read file");
    println!("Read: {}", character_sheet);
    println!("Finished loading character sheet");
}

fn data_entry(mut character: Character) -> Character {
    let data = character.as_vec();
    let stats = character.get_ordered_stats();
    let mut changes = std::collections::HashMap::new();
    //loop over each item in data, show what the current value is, and ask for an overwrite value - blank entry means no change
    let mut index = 0;
    for item in data {
        let stat = stats[index].clone();
        println!("{}: {}", stat, item);
        println!(
            "Enter a new value for {} or press enter to keep the current value:",
            stat
        );
        let mut buffer = String::new();
        io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to read line");
        let new_value = buffer.trim();
        if new_value != "" {
            print!(
                "Updated {} from {} to {}\n",
                stat,
                character.get_value(format!("{}", stat)),
                new_value
            );
            changes.insert(stat, new_value.to_string());
        }

        index += 1;
    }
    character = character.apply_hash_changes(changes);
    character
}
