use serde::{Deserialize, Serialize};
use std::{fs, io::{self, Write}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbilityScore {
    Strength = 0,
    Dexterity = 1,
    Constitution = 2,
    Wisdom = 3,
    Intelligence = 4,
    Charisma = 5,
}

impl AbilityScore {
    pub fn all() -> [AbilityScore; 6] {
        [
            AbilityScore::Strength,
            AbilityScore::Dexterity,
            AbilityScore::Constitution,
            AbilityScore::Wisdom,
            AbilityScore::Intelligence,
            AbilityScore::Charisma,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            AbilityScore::Strength => "Strength",
            AbilityScore::Dexterity => "Dexterity", 
            AbilityScore::Constitution => "Constitution",
            AbilityScore::Wisdom => "Wisdom",
            AbilityScore::Intelligence => "Intelligence",
            AbilityScore::Charisma => "Charisma",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            AbilityScore::Strength => "STR",
            AbilityScore::Dexterity => "DEX",
            AbilityScore::Constitution => "CON", 
            AbilityScore::Wisdom => "WIS",
            AbilityScore::Intelligence => "INT",
            AbilityScore::Charisma => "CHA",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cards {
    pub suit: Suit,
    pub rank: u8,
    pub desc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub race: Option<String>,
    pub class: Option<String>, 
    pub level: Option<u8>,
    pub desc: Option<String>,
    pub ac: Option<u8>,
    pub hp: Option<u8>,
    pub max_hp: Option<u8>,
    pub temp_hp: Option<u8>,
    pub speed: Option<u8>,
    pub intl: Option<u8>,
    pub wisd: Option<u8>,
    pub chas: Option<u8>,
    pub stre: Option<u8>,
    pub dext: Option<u8>,
    pub cons: Option<u8>,
    pub passive_perception: Option<u8>,
    pub initiative: Option<u8>,
    pub prof_bonus: Option<u8>,
    pub inventory: Vec<String>,
    pub cards: Vec<Cards>,
    pub spells: Vec<String>,
}

impl Character {
    pub fn new(name: &str) -> Character {
        Character {
            name: name.to_string(),
            race: None,
            class: None,
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

    /// Calculate ability modifier from ability score using D&D rules:
    /// Take the ability score and subtract 10. Divide the result by 2 and round down.
    pub fn calculate_modifier(ability_score: u8) -> i8 {
        ((ability_score as i16 - 10) / 2) as i8
    }

    pub fn get_ability_score(&self, ability: AbilityScore) -> Option<u8> {
        match ability {
            AbilityScore::Strength => self.stre,
            AbilityScore::Dexterity => self.dext,
            AbilityScore::Constitution => self.cons,
            AbilityScore::Wisdom => self.wisd,
            AbilityScore::Intelligence => self.intl,
            AbilityScore::Charisma => self.chas,
        }
    }

    pub fn get_ability_modifier(&self, ability: AbilityScore) -> i8 {
        if let Some(score) = self.get_ability_score(ability) {
            Self::calculate_modifier(score)
        } else {
            0 // Default modifier for missing scores
        }
    }

    pub fn get_strength_modifier(&self) -> i8 {
        self.get_ability_modifier(AbilityScore::Strength)
    }

    pub fn get_dexterity_modifier(&self) -> i8 {
        self.get_ability_modifier(AbilityScore::Dexterity)
    }

    pub fn get_constitution_modifier(&self) -> i8 {
        self.get_ability_modifier(AbilityScore::Constitution)
    }

    pub fn get_wisdom_modifier(&self) -> i8 {
        self.get_ability_modifier(AbilityScore::Wisdom)
    }

    pub fn get_intelligence_modifier(&self) -> i8 {
        self.get_ability_modifier(AbilityScore::Intelligence)
    }

    pub fn get_charisma_modifier(&self) -> i8 {
        self.get_ability_modifier(AbilityScore::Charisma)
    }

    /// Calculate passive perception: 10 + Wisdom Modifier + Proficiency Bonus
    pub fn calculate_passive_perception(&self) -> u8 {
        let wisdom_mod = self.get_wisdom_modifier();
        let prof_bonus = self.prof_bonus.unwrap_or(2) as i8;
        (10 + wisdom_mod + prof_bonus).max(1) as u8
    }

    /// Ensure passive perception is calculated and up-to-date
    pub fn update_passive_perception(&mut self) {
        self.passive_perception = Some(self.calculate_passive_perception());
    }

    /// Check for missing stats and prompt user input
    pub fn ensure_complete_stats(&mut self) {
        // Check if we should offer autofill-all for missing data
        let missing_data = self.count_missing_essential_data();
        if missing_data > 3 {
            println!("\n⚠️  Missing {} essential stats for {}!", missing_data, self.name);
            println!("Would you like to autofill all missing stats with defaults? (y/n): ");
            
            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_ok() && input.trim().to_lowercase() == "y" {
                self.autofill_missing_stats();
                return;
            }
        }

        if self.race.is_none() {
            self.race = Some(self.prompt_for_stat("Race", "Human"));
        }
        if self.class.is_none() {
            self.class = Some(self.prompt_for_stat("Class", "Fighter"));
        }
        if self.level.is_none() {
            self.level = Some(self.prompt_for_stat("Level", "1").parse().unwrap_or(1));
        }
        if self.prof_bonus.is_none() {
            let default_prof = match self.level.unwrap_or(1) {
                1..=4 => 2,
                5..=8 => 3,
                9..=12 => 4,
                13..=16 => 5,
                _ => 6,
            };
            self.prof_bonus = Some(self.prompt_for_stat("Proficiency Bonus", &default_prof.to_string()).parse().unwrap_or(default_prof));
        }

        // Ensure all ability scores are present
        if self.stre.is_none() {
            self.stre = Some(self.prompt_for_stat("Strength", "10").parse().unwrap_or(10));
        }
        if self.dext.is_none() {
            self.dext = Some(self.prompt_for_stat("Dexterity", "10").parse().unwrap_or(10));
        }
        if self.cons.is_none() {
            self.cons = Some(self.prompt_for_stat("Constitution", "10").parse().unwrap_or(10));
        }
        if self.wisd.is_none() {
            self.wisd = Some(self.prompt_for_stat("Wisdom", "10").parse().unwrap_or(10));
        }
        if self.intl.is_none() {
            self.intl = Some(self.prompt_for_stat("Intelligence", "10").parse().unwrap_or(10));
        }
        if self.chas.is_none() {
            self.chas = Some(self.prompt_for_stat("Charisma", "10").parse().unwrap_or(10));
        }

        // Ensure other core stats
        if self.ac.is_none() {
            self.ac = Some(self.prompt_for_stat("Armor Class", "10").parse().unwrap_or(10));
        }
        if self.max_hp.is_none() {
            self.max_hp = Some(self.prompt_for_stat("Max HP", "10").parse().unwrap_or(10));
        }
        if self.hp.is_none() {
            self.hp = self.max_hp;
        }
        if self.speed.is_none() {
            self.speed = Some(self.prompt_for_stat("Speed", "30").parse().unwrap_or(30));
        }

        // Update calculated stats
        self.update_passive_perception();
    }

    /// Count missing essential data fields
    fn count_missing_essential_data(&self) -> i32 {
        let mut missing = 0;
        if self.race.is_none() { missing += 1; }
        if self.class.is_none() { missing += 1; }
        if self.level.is_none() { missing += 1; }
        if self.stre.is_none() { missing += 1; }
        if self.dext.is_none() { missing += 1; }
        if self.cons.is_none() { missing += 1; }
        if self.wisd.is_none() { missing += 1; }
        if self.intl.is_none() { missing += 1; }
        if self.chas.is_none() { missing += 1; }
        if self.ac.is_none() { missing += 1; }
        if self.max_hp.is_none() { missing += 1; }
        if self.speed.is_none() { missing += 1; }
        missing
    }

    /// Autofill all missing stats with defaults
    fn autofill_missing_stats(&mut self) {
        if self.race.is_none() { self.race = Some("Human".to_string()); }
        if self.class.is_none() { self.class = Some("Fighter".to_string()); }
        if self.level.is_none() { self.level = Some(1); }
        if self.prof_bonus.is_none() { self.prof_bonus = Some(2); }
        if self.stre.is_none() { self.stre = Some(13); }
        if self.dext.is_none() { self.dext = Some(12); }
        if self.cons.is_none() { self.cons = Some(14); }
        if self.wisd.is_none() { self.wisd = Some(12); }
        if self.intl.is_none() { self.intl = Some(10); }
        if self.chas.is_none() { self.chas = Some(11); }
        if self.ac.is_none() { self.ac = Some(16); }
        if self.max_hp.is_none() { self.max_hp = Some(12); }
        if self.hp.is_none() { self.hp = self.max_hp; }
        if self.speed.is_none() { self.speed = Some(30); }
        self.update_passive_perception();
        
        println!("✅ Autofilled missing stats for {}", self.name);
    }

    fn prompt_for_stat(&self, stat_name: &str, default_value: &str) -> String {
        println!("{} is missing for {}. Enter {} (default: {}): ", 
                 stat_name, self.name, stat_name, default_value);
        
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok() {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                default_value.to_string()
            } else {
                trimmed.to_string()
            }
        } else {
            default_value.to_string()
        }
    }

    pub fn get_value(&self, key: String) -> String {
        match key.as_str() {
            "name" => self.name.clone(),
            "race" => self.race.clone().unwrap_or("Unknown".to_string()),
            "class" => self.class.clone().unwrap_or("Unknown".to_string()),
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

    pub fn get_ordered_stats(&self) -> Vec<String> {
        let mut stats = Vec::new();
        stats.push(format!("Name: {}", self.name));
        stats.push(format!("Race: {}", self.race.as_ref().unwrap_or(&"Unknown".to_string())));
        stats.push(format!("Class: {}", self.class.as_ref().unwrap_or(&"Unknown".to_string())));
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

        // Display ability scores in D&D standard order with modifiers
        for ability in AbilityScore::all() {
            let score = self.get_ability_score(ability).unwrap_or(10);
            let modifier = Self::calculate_modifier(score);
            let modifier_str = if modifier >= 0 {
                format!("+{}", modifier)
            } else {
                modifier.to_string()
            };
            stats.push(format!("{}: {} ({})", ability.name(), score, modifier_str));
        }

        stats.push(format!(
            "Passive Perception: {}",
            self.passive_perception.unwrap_or_else(|| self.calculate_passive_perception())
        ));
        stats.push(format!("Initiative: {}", self.initiative.unwrap_or(0)));
        stats.push(format!(
            "Proficiency Bonus: {}",
            self.prof_bonus.unwrap_or(0)
        ));
        stats
    }

    pub fn write_to_file(&self) -> io::Result<()> {
        let path = format!("characters/{}.txt", self.name);
        let mut file = fs::File::create(path)?;
        for stat in self.get_ordered_stats() {
            file.write_all(stat.as_bytes())?;
            file.write_all(b"\n")?;
        }
        Ok(())
    }

    pub fn as_vec(&self) -> Vec<String> {
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

    pub fn as_hashmap(&self) -> std::collections::HashMap<String, String> {
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

    pub fn apply_hash_changes(
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

    pub fn apply_vec_changes(self, changes: Vec<String>) -> Character {
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