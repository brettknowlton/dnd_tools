use rand::Rng;

// Common D&D 5e races
pub const RACES: &[&str] = &[
    "Human", "Elf", "Dwarf", "Halfling", "Dragonborn", "Gnome", 
    "Half-Elf", "Half-Orc", "Tiefling", "Aasimar", "Firbolg", 
    "Goliath", "Kenku", "Lizardfolk", "Tabaxi", "Triton",
    "Bugbear", "Goblin", "Hobgoblin", "Kobold", "Orc", "Yuan-Ti",
    "Aarakocra", "Genasi", "Githyanki", "Githzerai", "Minotaur",
    "Centaur", "Loxodon", "Simic Hybrid", "Vedalken", "Verdan",
    "Warforged", "Changeling", "Kalashtar", "Shifter", "Eladrin",
    "Fairy", "Harengon", "Owlin", "Satyr", "Sea Elf", "Shadar-Kai",
    "Duergar", "Deep Gnome", "Drow"
];

// Common D&D 5e classes  
pub const CLASSES: &[&str] = &[
    "Fighter", "Wizard", "Cleric", "Rogue", "Ranger", "Paladin",
    "Barbarian", "Bard", "Druid", "Monk", "Sorcerer", "Warlock",
    "Artificer", "Blood Hunter"
];

pub fn get_random_race() -> String {
    let mut rng = rand::rng();
    RACES[rng.random_range(0..RACES.len())].to_string()
}

pub fn get_random_class() -> String {
    let mut rng = rand::rng();
    CLASSES[rng.random_range(0..CLASSES.len())].to_string()
}

pub fn list_races() -> Vec<String> {
    RACES.iter().map(|&s| s.to_string()).collect()
}

pub fn list_classes() -> Vec<String> {
    CLASSES.iter().map(|&s| s.to_string()).collect()
}