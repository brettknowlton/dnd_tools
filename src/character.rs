use serde::{Deserialize, Serialize};
use std::{fs, io::{self, Write}};

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

    pub fn get_value(&self, key: String) -> String {
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

    pub fn get_ordered_stats(&self) -> Vec<String> {
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