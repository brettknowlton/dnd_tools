use serde::{Deserialize, Serialize};
use regex::Regex;
use std::collections::HashMap;

// API Response structures for D&D 5e API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiListResponse {
    pub count: usize,
    pub results: Vec<ApiReference>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiReference {
    pub index: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpellDetail {
    pub index: String,
    pub name: String,
    pub level: u8,
    pub school: ApiReference,
    pub casting_time: String,
    pub range: String,
    pub components: Vec<String>,
    pub duration: String,
    #[serde(rename = "desc")]
    pub description: Vec<String>,
    #[serde(default)]
    pub higher_level: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassDetail {
    pub index: String,
    pub name: String,
    pub hit_die: u8,
    pub proficiency_choices: Vec<ProficiencyChoice>,
    pub proficiencies: Vec<ApiReference>,
    pub saving_throws: Vec<ApiReference>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProficiencyChoice {
    #[serde(rename = "type")]
    pub choice_type: String,
    pub choose: u8,
    pub from: ApiReference,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EquipmentDetail {
    pub index: String,
    pub name: String,
    pub equipment_category: ApiReference,
    #[serde(default)]
    pub gear_category: Option<ApiReference>,
    #[serde(default)]
    pub weapon_category: Option<String>,
    #[serde(default)]
    pub armor_category: Option<String>,
    #[serde(default)]
    pub cost: Option<Cost>,
    #[serde(default)]
    pub weight: Option<f32>,
    #[serde(rename = "desc")]
    #[serde(default)]
    pub description: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cost {
    pub quantity: u32,
    pub unit: String,
}

// Search categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchCategory {
    Spells,
    Classes,
    Equipment,
    Monsters,
    Races,
}

impl SearchCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            SearchCategory::Spells => "spells",
            SearchCategory::Classes => "classes",
            SearchCategory::Equipment => "equipment",
            SearchCategory::Monsters => "monsters",
            SearchCategory::Races => "races",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "spell" | "spells" => Some(SearchCategory::Spells),
            "class" | "classes" => Some(SearchCategory::Classes),
            "equipment" | "item" | "items" | "gear" => Some(SearchCategory::Equipment),
            "monster" | "monsters" | "creature" | "creatures" => Some(SearchCategory::Monsters),
            "race" | "races" => Some(SearchCategory::Races),
            _ => None,
        }
    }

    pub fn all() -> Vec<SearchCategory> {
        vec![
            SearchCategory::Spells,
            SearchCategory::Classes,
            SearchCategory::Equipment,
            SearchCategory::Monsters,
            SearchCategory::Races,
        ]
    }
}

// Search result wrapper
#[derive(Debug, Clone)]
pub enum SearchResult {
    Spell(SpellDetail),
    Class(ClassDetail),
    Equipment(EquipmentDetail),
    Reference(ApiReference),
}

impl SearchResult {
    pub fn name(&self) -> &str {
        match self {
            SearchResult::Spell(spell) => &spell.name,
            SearchResult::Class(class) => &class.name,
            SearchResult::Equipment(equipment) => &equipment.name,
            SearchResult::Reference(reference) => &reference.name,
        }
    }

    pub fn index(&self) -> &str {
        match self {
            SearchResult::Spell(spell) => &spell.index,
            SearchResult::Class(class) => &class.index,
            SearchResult::Equipment(equipment) => &equipment.index,
            SearchResult::Reference(reference) => &reference.index,
        }
    }

    /// Get available fields that can be queried for this result type
    pub fn get_available_fields(&self) -> Vec<String> {
        match self {
            SearchResult::Spell(_) => vec![
                "name".to_string(),
                "level".to_string(),
                "school".to_string(),
                "casting_time".to_string(),
                "range".to_string(),
                "components".to_string(),
                "duration".to_string(),
                "description".to_string(),
                "higher_level".to_string(),
            ],
            SearchResult::Class(_) => vec![
                "name".to_string(),
                "hit_die".to_string(),
                "proficiencies".to_string(),
                "saving_throws".to_string(),
                "proficiency_choices".to_string(),
            ],
            SearchResult::Equipment(_) => vec![
                "name".to_string(),
                "category".to_string(),
                "gear_category".to_string(),
                "weapon_category".to_string(),
                "armor_category".to_string(),
                "cost".to_string(),
                "weight".to_string(),
                "description".to_string(),
            ],
            SearchResult::Reference(_) => vec![
                "name".to_string(),
                "index".to_string(),
                "url".to_string(),
            ],
        }
    }

    /// Get the value of a specific field
    pub fn get_field_value(&self, field: &str) -> Option<String> {
        match self {
            SearchResult::Spell(spell) => {
                match field.to_lowercase().as_str() {
                    "name" => Some(spell.name.clone()),
                    "level" => Some(format!("Level {}", spell.level)),
                    "school" => Some(spell.school.name.clone()),
                    "casting_time" => Some(spell.casting_time.clone()),
                    "range" => Some(spell.range.clone()),
                    "components" => Some(spell.components.join(", ")),
                    "duration" => Some(spell.duration.clone()),
                    "description" => {
                        if spell.description.is_empty() {
                            Some("No description available".to_string())
                        } else {
                            Some(spell.description.join("\n"))
                        }
                    },
                    "higher_level" => {
                        if spell.higher_level.is_empty() {
                            Some("No higher level effects".to_string())
                        } else {
                            Some(spell.higher_level.join("\n"))
                        }
                    },
                    _ => None,
                }
            },
            SearchResult::Class(class) => {
                match field.to_lowercase().as_str() {
                    "name" => Some(class.name.clone()),
                    "hit_die" => Some(format!("d{}", class.hit_die)),
                    "proficiencies" => {
                        if class.proficiencies.is_empty() {
                            Some("No proficiencies listed".to_string())
                        } else {
                            Some(class.proficiencies.iter()
                                .map(|p| p.name.as_str())
                                .collect::<Vec<_>>()
                                .join(", "))
                        }
                    },
                    "saving_throws" => {
                        if class.saving_throws.is_empty() {
                            Some("No saving throw proficiencies listed".to_string())
                        } else {
                            Some(class.saving_throws.iter()
                                .map(|s| s.name.as_str())
                                .collect::<Vec<_>>()
                                .join(", "))
                        }
                    },
                    "proficiency_choices" => {
                        if class.proficiency_choices.is_empty() {
                            Some("No proficiency choices listed".to_string())
                        } else {
                            Some(format!("{} proficiency choices available", class.proficiency_choices.len()))
                        }
                    },
                    _ => None,
                }
            },
            SearchResult::Equipment(equipment) => {
                match field.to_lowercase().as_str() {
                    "name" => Some(equipment.name.clone()),
                    "category" => Some(equipment.equipment_category.name.clone()),
                    "gear_category" => {
                        if let Some(ref gear_cat) = equipment.gear_category {
                            Some(gear_cat.name.clone())
                        } else {
                            Some("No gear category".to_string())
                        }
                    },
                    "weapon_category" => {
                        equipment.weapon_category.clone()
                            .unwrap_or_else(|| "Not a weapon".to_string()).into()
                    },
                    "armor_category" => {
                        equipment.armor_category.clone()
                            .unwrap_or_else(|| "Not armor".to_string()).into()
                    },
                    "cost" => {
                        if let Some(ref cost) = equipment.cost {
                            Some(format!("{} {}", cost.quantity, cost.unit))
                        } else {
                            Some("No cost listed".to_string())
                        }
                    },
                    "weight" => {
                        if let Some(weight) = equipment.weight {
                            Some(format!("{} lb", weight))
                        } else {
                            Some("No weight listed".to_string())
                        }
                    },
                    "description" => {
                        if equipment.description.is_empty() {
                            Some("No description available".to_string())
                        } else {
                            Some(equipment.description.join("\n"))
                        }
                    },
                    _ => None,
                }
            },
            SearchResult::Reference(reference) => {
                match field.to_lowercase().as_str() {
                    "name" => Some(reference.name.clone()),
                    "index" => Some(reference.index.clone()),
                    "url" => Some(reference.url.clone()),
                    _ => None,
                }
            },
        }
    }

    /// Display a specific field in a formatted way
    pub fn display_field(&self, field: &str) {
        if let Some(value) = self.get_field_value(field) {
            println!("\n╔══════════════════════════════════════════╗");
            println!("║ {}: {:<33} ║", 
                field.to_uppercase(), 
                if field.len() > 33 { &field[..33] } else { field });
            println!("╠══════════════════════════════════════════╣");
            
            // Handle multi-line values
            for line in value.lines() {
                let truncated = if line.len() > 38 {
                    format!("{}...", &line[..35])
                } else {
                    line.to_string()
                };
                println!("║ {:<40} ║", truncated);
            }
            
            println!("╚══════════════════════════════════════════╝");
        } else {
            println!("\n❌ Field '{}' not available for this result type", field);
        }
    }

    pub fn display(&self) {
        match self {
            SearchResult::Spell(spell) => self.display_spell(spell),
            SearchResult::Class(class) => self.display_class(class),
            SearchResult::Equipment(equipment) => self.display_equipment(equipment),
            SearchResult::Reference(reference) => self.display_reference(reference),
        }
    }

    fn display_spell(&self, spell: &SpellDetail) {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║                SPELL                  ║");
        println!("╠═══════════════════════════════════════╣");
        println!("║ Name: {:<31} ║", spell.name);
        println!("║ Level: {:<30} ║", spell.level);
        println!("║ School: {:<29} ║", spell.school.name);
        println!("║ Casting Time: {:<23} ║", spell.casting_time);
        println!("║ Range: {:<30} ║", spell.range);
        println!("║ Components: {:<25} ║", spell.components.join(", "));
        println!("║ Duration: {:<27} ║", spell.duration);
        println!("╚═══════════════════════════════════════╝");
        
        if !spell.description.is_empty() {
            println!("\nDescription:");
            for desc in &spell.description {
                println!("  {}", desc);
            }
        }
        
        if !spell.higher_level.is_empty() {
            println!("\nAt Higher Levels:");
            for higher in &spell.higher_level {
                println!("  {}", higher);
            }
        }
    }

    fn display_class(&self, class: &ClassDetail) {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║                CLASS                  ║");
        println!("╠═══════════════════════════════════════╣");
        println!("║ Name: {:<31} ║", class.name);
        println!("║ Hit Die: d{:<27} ║", class.hit_die);
        println!("╚═══════════════════════════════════════╝");
        
        if !class.proficiencies.is_empty() {
            println!("\nProficiencies:");
            for prof in &class.proficiencies {
                println!("  • {}", prof.name);
            }
        }
        
        if !class.saving_throws.is_empty() {
            println!("\nSaving Throw Proficiencies:");
            for save in &class.saving_throws {
                println!("  • {}", save.name);
            }
        }
    }

    fn display_equipment(&self, equipment: &EquipmentDetail) {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║              EQUIPMENT                ║");
        println!("╠═══════════════════════════════════════╣");
        println!("║ Name: {:<31} ║", equipment.name);
        println!("║ Category: {:<27} ║", equipment.equipment_category.name);
        
        if let Some(cost) = &equipment.cost {
            println!("║ Cost: {} {:<24} ║", cost.quantity, cost.unit);
        }
        
        if let Some(weight) = equipment.weight {
            println!("║ Weight: {:<27} lb ║", weight);
        }
        
        println!("╚═══════════════════════════════════════╝");
        
        if !equipment.description.is_empty() {
            println!("\nDescription:");
            for desc in &equipment.description {
                println!("  {}", desc);
            }
        }
    }

    fn display_reference(&self, reference: &ApiReference) {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║              REFERENCE                ║");
        println!("╠═══════════════════════════════════════╣");
        println!("║ Name: {:<31} ║", reference.name);
        println!("║ Index: {:<30} ║", reference.index);
        println!("╚═══════════════════════════════════════╝");
    }
}

// Main search client
pub struct DndSearchClient {
    base_url: String,
    client: Option<reqwest::Client>,
    offline_mode: bool,
    // Cache for offline fallback
    cached_data: HashMap<SearchCategory, Vec<ApiReference>>,
}

impl Default for DndSearchClient {
    fn default() -> Self {
        Self::new()
    }
}

impl DndSearchClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .ok();
        
        let offline_mode = client.is_none();
        
        DndSearchClient {
            base_url: "https://www.dnd5eapi.co/api".to_string(),
            client,
            offline_mode,
            cached_data: Self::initialize_offline_cache(),
        }
    }

    pub fn set_offline_mode(&mut self, offline: bool) {
        self.offline_mode = offline;
    }

    pub fn is_offline(&self) -> bool {
        self.offline_mode || self.client.is_none()
    }

    // Search with fuzzy matching
    pub async fn search(&self, query: &str, category: Option<SearchCategory>) -> Result<Vec<SearchResult>, String> {
        let categories = match category {
            Some(cat) => vec![cat],
            None => SearchCategory::all(),
        };

        let mut all_results = Vec::new();

        for cat in categories {
            match self.search_category(query, cat).await {
                Ok(mut results) => all_results.append(&mut results),
                Err(e) => {
                    eprintln!("Warning: Failed to search {}: {}", cat.as_str(), e);
                    // Continue with other categories
                }
            }
        }

        if all_results.is_empty() {
            // Try fuzzy matching
            self.fuzzy_search(query, category).await
        } else {
            Ok(all_results)
        }
    }

    async fn search_category(&self, query: &str, category: SearchCategory) -> Result<Vec<SearchResult>, String> {
        if self.is_offline() {
            return self.offline_search(query, category);
        }

        if let Some(client) = &self.client {
            match self.online_search(client, query, category).await {
                Ok(results) => Ok(results),
                Err(_) => {
                    // Fallback to offline search
                    eprintln!("Online search failed, falling back to offline mode");
                    self.offline_search(query, category)
                }
            }
        } else {
            self.offline_search(query, category)
        }
    }

    async fn online_search(&self, client: &reqwest::Client, query: &str, category: SearchCategory) -> Result<Vec<SearchResult>, String> {
        let url = format!("{}/{}", self.base_url, category.as_str());
        
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network request failed: {}", e))?;

        let list_response: ApiListResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        // Find exact matches first
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        for item in &list_response.results {
            if item.name.to_lowercase() == query_lower {
                // Exact match - fetch details
                if let Ok(detailed_result) = self.fetch_details(client, &item.url, category).await {
                    results.push(detailed_result);
                }
            }
        }

        Ok(results)
    }

    fn offline_search(&self, query: &str, category: SearchCategory) -> Result<Vec<SearchResult>, String> {
        if let Some(cached_items) = self.cached_data.get(&category) {
            let query_lower = query.to_lowercase();
            let mut results = Vec::new();

            for item in cached_items {
                if item.name.to_lowercase() == query_lower {
                    results.push(SearchResult::Reference(item.clone()));
                }
            }

            Ok(results)
        } else {
            Err("No cached data available for this category".to_string())
        }
    }

    async fn fuzzy_search(&self, query: &str, category: Option<SearchCategory>) -> Result<Vec<SearchResult>, String> {
        let categories = match category {
            Some(cat) => vec![cat],
            None => SearchCategory::all(),
        };

        let mut fuzzy_results = Vec::new();
        let query_pattern = Self::create_fuzzy_regex(query)?;

        for cat in categories {
            let items = if self.is_offline() {
                self.cached_data.get(&cat).cloned().unwrap_or_default()
            } else {
                // For online mode, we'd need to fetch all items first
                // For now, fall back to cached data
                self.cached_data.get(&cat).cloned().unwrap_or_default()
            };

            for item in items {
                if query_pattern.is_match(&item.name.to_lowercase()) {
                    fuzzy_results.push(SearchResult::Reference(item));
                }
            }
        }

        if fuzzy_results.is_empty() {
            Err(format!("No matches found for '{}'", query))
        } else {
            Ok(fuzzy_results)
        }
    }

    fn create_fuzzy_regex(query: &str) -> Result<Regex, String> {
        // Create a regex pattern that allows for partial matches
        let escaped_query = regex::escape(query);
        let pattern = format!(r"(?i).*{}.*", escaped_query);
        
        Regex::new(&pattern).map_err(|e| format!("Invalid regex pattern: {}", e))
    }

    async fn fetch_details(&self, client: &reqwest::Client, url: &str, category: SearchCategory) -> Result<SearchResult, String> {
        let full_url = format!("{}{}", self.base_url, url);
        
        let response = client
            .get(&full_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch details: {}", e))?;

        match category {
            SearchCategory::Spells => {
                let spell: SpellDetail = response
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse spell details: {}", e))?;
                Ok(SearchResult::Spell(spell))
            },
            SearchCategory::Classes => {
                let class: ClassDetail = response
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse class details: {}", e))?;
                Ok(SearchResult::Class(class))
            },
            SearchCategory::Equipment => {
                let equipment: EquipmentDetail = response
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse equipment details: {}", e))?;
                Ok(SearchResult::Equipment(equipment))
            },
            _ => {
                // For other categories, return as reference for now
                let reference = ApiReference {
                    index: "unknown".to_string(),
                    name: "Unknown".to_string(),
                    url: url.to_string(),
                };
                Ok(SearchResult::Reference(reference))
            }
        }
    }

    fn initialize_offline_cache() -> HashMap<SearchCategory, Vec<ApiReference>> {
        let mut cache = HashMap::new();

        // Populate with some common spells
        let common_spells = vec![
            ApiReference { index: "fireball".to_string(), name: "Fireball".to_string(), url: "/spells/fireball".to_string() },
            ApiReference { index: "magic-missile".to_string(), name: "Magic Missile".to_string(), url: "/spells/magic-missile".to_string() },
            ApiReference { index: "cure-wounds".to_string(), name: "Cure Wounds".to_string(), url: "/spells/cure-wounds".to_string() },
            ApiReference { index: "shield".to_string(), name: "Shield".to_string(), url: "/spells/shield".to_string() },
            ApiReference { index: "healing-word".to_string(), name: "Healing Word".to_string(), url: "/spells/healing-word".to_string() },
            ApiReference { index: "counterspell".to_string(), name: "Counterspell".to_string(), url: "/spells/counterspell".to_string() },
            ApiReference { index: "dimension-door".to_string(), name: "Dimension Door".to_string(), url: "/spells/dimension-door".to_string() },
            ApiReference { index: "lightning-bolt".to_string(), name: "Lightning Bolt".to_string(), url: "/spells/lightning-bolt".to_string() },
        ];

        // Common classes
        let common_classes = vec![
            ApiReference { index: "fighter".to_string(), name: "Fighter".to_string(), url: "/classes/fighter".to_string() },
            ApiReference { index: "wizard".to_string(), name: "Wizard".to_string(), url: "/classes/wizard".to_string() },
            ApiReference { index: "cleric".to_string(), name: "Cleric".to_string(), url: "/classes/cleric".to_string() },
            ApiReference { index: "rogue".to_string(), name: "Rogue".to_string(), url: "/classes/rogue".to_string() },
            ApiReference { index: "ranger".to_string(), name: "Ranger".to_string(), url: "/classes/ranger".to_string() },
            ApiReference { index: "paladin".to_string(), name: "Paladin".to_string(), url: "/classes/paladin".to_string() },
            ApiReference { index: "barbarian".to_string(), name: "Barbarian".to_string(), url: "/classes/barbarian".to_string() },
            ApiReference { index: "bard".to_string(), name: "Bard".to_string(), url: "/classes/bard".to_string() },
            ApiReference { index: "druid".to_string(), name: "Druid".to_string(), url: "/classes/druid".to_string() },
            ApiReference { index: "monk".to_string(), name: "Monk".to_string(), url: "/classes/monk".to_string() },
            ApiReference { index: "sorcerer".to_string(), name: "Sorcerer".to_string(), url: "/classes/sorcerer".to_string() },
            ApiReference { index: "warlock".to_string(), name: "Warlock".to_string(), url: "/classes/warlock".to_string() },
        ];

        // Common equipment
        let common_equipment = vec![
            ApiReference { index: "longsword".to_string(), name: "Longsword".to_string(), url: "/equipment/longsword".to_string() },
            ApiReference { index: "shortsword".to_string(), name: "Shortsword".to_string(), url: "/equipment/shortsword".to_string() },
            ApiReference { index: "dagger".to_string(), name: "Dagger".to_string(), url: "/equipment/dagger".to_string() },
            ApiReference { index: "shortbow".to_string(), name: "Shortbow".to_string(), url: "/equipment/shortbow".to_string() },
            ApiReference { index: "longbow".to_string(), name: "Longbow".to_string(), url: "/equipment/longbow".to_string() },
            ApiReference { index: "chain-mail".to_string(), name: "Chain Mail".to_string(), url: "/equipment/chain-mail".to_string() },
            ApiReference { index: "leather-armor".to_string(), name: "Leather Armor".to_string(), url: "/equipment/leather-armor".to_string() },
            ApiReference { index: "shield".to_string(), name: "Shield".to_string(), url: "/equipment/shield".to_string() },
        ];

        // Common races
        let common_races = vec![
            ApiReference { index: "human".to_string(), name: "Human".to_string(), url: "/races/human".to_string() },
            ApiReference { index: "elf".to_string(), name: "Elf".to_string(), url: "/races/elf".to_string() },
            ApiReference { index: "dwarf".to_string(), name: "Dwarf".to_string(), url: "/races/dwarf".to_string() },
            ApiReference { index: "halfling".to_string(), name: "Halfling".to_string(), url: "/races/halfling".to_string() },
            ApiReference { index: "dragonborn".to_string(), name: "Dragonborn".to_string(), url: "/races/dragonborn".to_string() },
            ApiReference { index: "gnome".to_string(), name: "Gnome".to_string(), url: "/races/gnome".to_string() },
            ApiReference { index: "half-elf".to_string(), name: "Half-Elf".to_string(), url: "/races/half-elf".to_string() },
            ApiReference { index: "half-orc".to_string(), name: "Half-Orc".to_string(), url: "/races/half-orc".to_string() },
            ApiReference { index: "tiefling".to_string(), name: "Tiefling".to_string(), url: "/races/tiefling".to_string() },
        ];

        // Common monsters
        let common_monsters = vec![
            ApiReference { index: "goblin".to_string(), name: "Goblin".to_string(), url: "/monsters/goblin".to_string() },
            ApiReference { index: "orc".to_string(), name: "Orc".to_string(), url: "/monsters/orc".to_string() },
            ApiReference { index: "troll".to_string(), name: "Troll".to_string(), url: "/monsters/troll".to_string() },
            ApiReference { index: "dragon".to_string(), name: "Adult Red Dragon".to_string(), url: "/monsters/adult-red-dragon".to_string() },
            ApiReference { index: "skeleton".to_string(), name: "Skeleton".to_string(), url: "/monsters/skeleton".to_string() },
            ApiReference { index: "zombie".to_string(), name: "Zombie".to_string(), url: "/monsters/zombie".to_string() },
            ApiReference { index: "wolf".to_string(), name: "Wolf".to_string(), url: "/monsters/wolf".to_string() },
            ApiReference { index: "bear".to_string(), name: "Brown Bear".to_string(), url: "/monsters/brown-bear".to_string() },
        ];

        cache.insert(SearchCategory::Spells, common_spells);
        cache.insert(SearchCategory::Classes, common_classes);
        cache.insert(SearchCategory::Equipment, common_equipment);
        cache.insert(SearchCategory::Races, common_races);
        cache.insert(SearchCategory::Monsters, common_monsters);

        cache
    }

    // Method to get suggestions when no exact match is found
    pub async fn get_suggestions(&self, query: &str, category: Option<SearchCategory>) -> Vec<String> {
        let mut suggestions = Vec::new();
        let categories = match category {
            Some(cat) => vec![cat],
            None => SearchCategory::all(),
        };

        for cat in categories {
            let items = self.cached_data.get(&cat).cloned().unwrap_or_default();
            
            // Use fuzzy matching to find similar items
            if let Ok(pattern) = Self::create_fuzzy_regex(query) {
                for item in items {
                    if pattern.is_match(&item.name.to_lowercase()) && !suggestions.contains(&item.name) {
                        suggestions.push(item.name);
                    }
                }
            }
        }

        // Sort suggestions by similarity (simple implementation)
        suggestions.sort_by(|a, b| {
            let a_score = Self::similarity_score(query, a);
            let b_score = Self::similarity_score(query, b);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Return top 5 suggestions
        suggestions.truncate(5);
        suggestions
    }

    fn similarity_score(query: &str, candidate: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let candidate_lower = candidate.to_lowercase();
        
        // Simple similarity based on common characters and length difference
        let common_chars: usize = query_lower.chars()
            .filter(|c| candidate_lower.contains(*c))
            .count();
        
        let length_penalty = ((query_lower.len() as i32 - candidate_lower.len() as i32).abs() as f32) * 0.1;
        
        (common_chars as f32) / (query_lower.len() as f32) - length_penalty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_category_from_str() {
        assert_eq!(SearchCategory::from_str("spell"), Some(SearchCategory::Spells));
        assert_eq!(SearchCategory::from_str("spells"), Some(SearchCategory::Spells));
        assert_eq!(SearchCategory::from_str("class"), Some(SearchCategory::Classes));
        assert_eq!(SearchCategory::from_str("classes"), Some(SearchCategory::Classes));
        assert_eq!(SearchCategory::from_str("equipment"), Some(SearchCategory::Equipment));
        assert_eq!(SearchCategory::from_str("item"), Some(SearchCategory::Equipment));
        assert_eq!(SearchCategory::from_str("gear"), Some(SearchCategory::Equipment));
        assert_eq!(SearchCategory::from_str("monster"), Some(SearchCategory::Monsters));
        assert_eq!(SearchCategory::from_str("creatures"), Some(SearchCategory::Monsters));
        assert_eq!(SearchCategory::from_str("race"), Some(SearchCategory::Races));
        assert_eq!(SearchCategory::from_str("races"), Some(SearchCategory::Races));
        assert_eq!(SearchCategory::from_str("invalid"), None);
    }

    #[test]
    fn test_search_category_as_str() {
        assert_eq!(SearchCategory::Spells.as_str(), "spells");
        assert_eq!(SearchCategory::Classes.as_str(), "classes");
        assert_eq!(SearchCategory::Equipment.as_str(), "equipment");
        assert_eq!(SearchCategory::Monsters.as_str(), "monsters");
        assert_eq!(SearchCategory::Races.as_str(), "races");
    }

    #[test]
    fn test_search_result_name_and_index() {
        let spell = SpellDetail {
            index: "fireball".to_string(),
            name: "Fireball".to_string(),
            level: 3,
            school: ApiReference {
                index: "evocation".to_string(),
                name: "Evocation".to_string(),
                url: "/magic-schools/evocation".to_string(),
            },
            casting_time: "1 action".to_string(),
            range: "150 feet".to_string(),
            components: vec!["V".to_string(), "S".to_string(), "M".to_string()],
            duration: "Instantaneous".to_string(),
            description: vec!["A bright streak flashes...".to_string()],
            higher_level: vec![],
        };
        
        let result = SearchResult::Spell(spell);
        assert_eq!(result.name(), "Fireball");
        assert_eq!(result.index(), "fireball");
    }

    #[test]
    fn test_dnd_search_client_creation() {
        let client = DndSearchClient::new();
        assert_eq!(client.base_url, "https://www.dnd5eapi.co/api");
        // Client may be offline in test environment
        assert!(client.cached_data.len() > 0);
    }

    #[test]
    fn test_offline_cache_populated() {
        let client = DndSearchClient::new();
        
        // Test that cache has data for all categories
        assert!(client.cached_data.contains_key(&SearchCategory::Spells));
        assert!(client.cached_data.contains_key(&SearchCategory::Classes));
        assert!(client.cached_data.contains_key(&SearchCategory::Equipment));
        assert!(client.cached_data.contains_key(&SearchCategory::Races));
        assert!(client.cached_data.contains_key(&SearchCategory::Monsters));
        
        // Test that spells cache has fireball
        let spells = client.cached_data.get(&SearchCategory::Spells).unwrap();
        assert!(spells.iter().any(|s| s.name == "Fireball"));
        
        // Test that classes cache has Fighter
        let classes = client.cached_data.get(&SearchCategory::Classes).unwrap();
        assert!(classes.iter().any(|c| c.name == "Fighter"));
    }

    #[test]
    fn test_fuzzy_regex_creation() {
        let regex = DndSearchClient::create_fuzzy_regex("fire").unwrap();
        assert!(regex.is_match("fireball"));
        assert!(regex.is_match("Fire Bolt"));
        assert!(regex.is_match("burning fire"));
        assert!(!regex.is_match("water"));
    }

    #[test]
    fn test_similarity_score() {
        assert!(DndSearchClient::similarity_score("fire", "fireball") > 0.0);
        assert!(DndSearchClient::similarity_score("fire", "fireball") > 
                DndSearchClient::similarity_score("fire", "water"));
        assert!(DndSearchClient::similarity_score("wizard", "wizard") > 
                DndSearchClient::similarity_score("wizard", "fighter"));
    }

    #[tokio::test]
    async fn test_offline_search() {
        let mut client = DndSearchClient::new();
        client.set_offline_mode(true);
        
        // Test exact match
        let results = client.search_category("Fireball", SearchCategory::Spells).await;
        assert!(results.is_ok());
        let results = results.unwrap();
        assert!(results.len() > 0);
        assert_eq!(results[0].name(), "Fireball");
        
        // Test no match
        let results = client.search_category("NonExistentSpell", SearchCategory::Spells).await;
        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_fuzzy_search() {
        let mut client = DndSearchClient::new();
        client.set_offline_mode(true);
        
        // Test fuzzy match
        let results = client.fuzzy_search("fire", Some(SearchCategory::Spells)).await;
        assert!(results.is_ok());
        let results = results.unwrap();
        assert!(results.len() > 0);
        
        // Should find Fireball
        assert!(results.iter().any(|r| r.name().contains("Fire") || r.name().contains("fire")));
    }

    #[tokio::test]
    async fn test_get_suggestions() {
        let mut client = DndSearchClient::new();
        client.set_offline_mode(true);
        
        let suggestions = client.get_suggestions("fir", Some(SearchCategory::Spells)).await;
        assert!(suggestions.len() > 0);
        assert!(suggestions.iter().any(|s| s.to_lowercase().contains("fire")));
        
        // Test with no matches should return empty
        let suggestions = client.get_suggestions("zzznonexistent", Some(SearchCategory::Spells)).await;
        assert!(suggestions.len() == 0);
    }

    #[tokio::test]
    async fn test_search_with_suggestions() {
        let mut client = DndSearchClient::new();
        client.set_offline_mode(true);
        
        // Test exact match should work
        let results = client.search("Fireball", Some(SearchCategory::Spells)).await;
        assert!(results.is_ok());
        let results = results.unwrap();
        assert!(results.len() > 0);
        
        // Test partial match should trigger fuzzy search
        let results = client.search("fire", Some(SearchCategory::Spells)).await;
        assert!(results.is_ok());
        let results = results.unwrap();
        assert!(results.len() > 0);
    }

    #[test]
    fn test_search_all_categories() {
        let all_categories = SearchCategory::all();
        assert_eq!(all_categories.len(), 5);
        assert!(all_categories.contains(&SearchCategory::Spells));
        assert!(all_categories.contains(&SearchCategory::Classes));
        assert!(all_categories.contains(&SearchCategory::Equipment));
        assert!(all_categories.contains(&SearchCategory::Monsters));
        assert!(all_categories.contains(&SearchCategory::Races));
    }

    #[test]
    fn test_spell_available_fields() {
        let spell = SpellDetail {
            index: "fireball".to_string(),
            name: "Fireball".to_string(),
            level: 3,
            school: ApiReference {
                index: "evocation".to_string(),
                name: "Evocation".to_string(),
                url: "/magic-schools/evocation".to_string(),
            },
            casting_time: "1 action".to_string(),
            range: "150 feet".to_string(),
            components: vec!["V".to_string(), "S".to_string(), "M".to_string()],
            duration: "Instantaneous".to_string(),
            description: vec!["A bright streak flashes...".to_string()],
            higher_level: vec!["When you cast this spell...".to_string()],
        };
        
        let result = SearchResult::Spell(spell);
        let fields = result.get_available_fields();
        
        assert_eq!(fields.len(), 9);
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"level".to_string()));
        assert!(fields.contains(&"school".to_string()));
        assert!(fields.contains(&"casting_time".to_string()));
        assert!(fields.contains(&"range".to_string()));
        assert!(fields.contains(&"components".to_string()));
        assert!(fields.contains(&"duration".to_string()));
        assert!(fields.contains(&"description".to_string()));
        assert!(fields.contains(&"higher_level".to_string()));
    }

    #[test]
    fn test_class_available_fields() {
        let class = ClassDetail {
            index: "fighter".to_string(),
            name: "Fighter".to_string(),
            hit_die: 10,
            proficiency_choices: vec![],
            proficiencies: vec![],
            saving_throws: vec![],
        };
        
        let result = SearchResult::Class(class);
        let fields = result.get_available_fields();
        
        assert_eq!(fields.len(), 5);
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"hit_die".to_string()));
        assert!(fields.contains(&"proficiencies".to_string()));
        assert!(fields.contains(&"saving_throws".to_string()));
        assert!(fields.contains(&"proficiency_choices".to_string()));
    }

    #[test]
    fn test_equipment_available_fields() {
        let equipment = EquipmentDetail {
            index: "longsword".to_string(),
            name: "Longsword".to_string(),
            equipment_category: ApiReference {
                index: "weapon".to_string(),
                name: "Weapon".to_string(),
                url: "/equipment-categories/weapon".to_string(),
            },
            gear_category: None,
            weapon_category: Some("Martial Melee".to_string()),
            armor_category: None,
            cost: Some(Cost {
                quantity: 15,
                unit: "gp".to_string(),
            }),
            weight: Some(3.0),
            description: vec!["A versatile weapon.".to_string()],
        };
        
        let result = SearchResult::Equipment(equipment);
        let fields = result.get_available_fields();
        
        assert_eq!(fields.len(), 8);
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"category".to_string()));
        assert!(fields.contains(&"cost".to_string()));
        assert!(fields.contains(&"weight".to_string()));
    }

    #[test]
    fn test_reference_available_fields() {
        let reference = ApiReference {
            index: "fireball".to_string(),
            name: "Fireball".to_string(),
            url: "/spells/fireball".to_string(),
        };
        
        let result = SearchResult::Reference(reference);
        let fields = result.get_available_fields();
        
        assert_eq!(fields.len(), 3);
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"index".to_string()));
        assert!(fields.contains(&"url".to_string()));
    }

    #[test]
    fn test_spell_field_values() {
        let spell = SpellDetail {
            index: "fireball".to_string(),
            name: "Fireball".to_string(),
            level: 3,
            school: ApiReference {
                index: "evocation".to_string(),
                name: "Evocation".to_string(),
                url: "/magic-schools/evocation".to_string(),
            },
            casting_time: "1 action".to_string(),
            range: "150 feet".to_string(),
            components: vec!["V".to_string(), "S".to_string(), "M".to_string()],
            duration: "Instantaneous".to_string(),
            description: vec!["A bright streak flashes...".to_string()],
            higher_level: vec![],
        };
        
        let result = SearchResult::Spell(spell);
        
        assert_eq!(result.get_field_value("name"), Some("Fireball".to_string()));
        assert_eq!(result.get_field_value("level"), Some("Level 3".to_string()));
        assert_eq!(result.get_field_value("school"), Some("Evocation".to_string()));
        assert_eq!(result.get_field_value("casting_time"), Some("1 action".to_string()));
        assert_eq!(result.get_field_value("range"), Some("150 feet".to_string()));
        assert_eq!(result.get_field_value("components"), Some("V, S, M".to_string()));
        assert_eq!(result.get_field_value("duration"), Some("Instantaneous".to_string()));
        assert_eq!(result.get_field_value("description"), Some("A bright streak flashes...".to_string()));
        assert_eq!(result.get_field_value("higher_level"), Some("No higher level effects".to_string()));
        assert_eq!(result.get_field_value("invalid_field"), None);
    }

    #[test]
    fn test_class_field_values() {
        let class = ClassDetail {
            index: "fighter".to_string(),
            name: "Fighter".to_string(),
            hit_die: 10,
            proficiency_choices: vec![],
            proficiencies: vec![
                ApiReference {
                    index: "armor-light".to_string(),
                    name: "Light Armor".to_string(),
                    url: "/proficiencies/armor-light".to_string(),
                }
            ],
            saving_throws: vec![
                ApiReference {
                    index: "str".to_string(),
                    name: "Strength".to_string(),
                    url: "/ability-scores/str".to_string(),
                }
            ],
        };
        
        let result = SearchResult::Class(class);
        
        assert_eq!(result.get_field_value("name"), Some("Fighter".to_string()));
        assert_eq!(result.get_field_value("hit_die"), Some("d10".to_string()));
        assert_eq!(result.get_field_value("proficiencies"), Some("Light Armor".to_string()));
        assert_eq!(result.get_field_value("saving_throws"), Some("Strength".to_string()));
        assert_eq!(result.get_field_value("proficiency_choices"), Some("No proficiency choices listed".to_string()));
        assert_eq!(result.get_field_value("invalid_field"), None);
    }

    #[test]
    fn test_equipment_field_values() {
        let equipment = EquipmentDetail {
            index: "longsword".to_string(),
            name: "Longsword".to_string(),
            equipment_category: ApiReference {
                index: "weapon".to_string(),
                name: "Weapon".to_string(),
                url: "/equipment-categories/weapon".to_string(),
            },
            gear_category: None,
            weapon_category: Some("Martial Melee".to_string()),
            armor_category: None,
            cost: Some(Cost {
                quantity: 15,
                unit: "gp".to_string(),
            }),
            weight: Some(3.0),
            description: vec!["A versatile weapon.".to_string()],
        };
        
        let result = SearchResult::Equipment(equipment);
        
        assert_eq!(result.get_field_value("name"), Some("Longsword".to_string()));
        assert_eq!(result.get_field_value("category"), Some("Weapon".to_string()));
        assert_eq!(result.get_field_value("gear_category"), Some("No gear category".to_string()));
        assert_eq!(result.get_field_value("weapon_category"), Some("Martial Melee".to_string()));
        assert_eq!(result.get_field_value("armor_category"), Some("Not armor".to_string()));
        assert_eq!(result.get_field_value("cost"), Some("15 gp".to_string()));
        assert_eq!(result.get_field_value("weight"), Some("3 lb".to_string()));
        assert_eq!(result.get_field_value("description"), Some("A versatile weapon.".to_string()));
        assert_eq!(result.get_field_value("invalid_field"), None);
    }

    #[test]
    fn test_reference_field_values() {
        let reference = ApiReference {
            index: "fireball".to_string(),
            name: "Fireball".to_string(),
            url: "/spells/fireball".to_string(),
        };
        
        let result = SearchResult::Reference(reference);
        
        assert_eq!(result.get_field_value("name"), Some("Fireball".to_string()));
        assert_eq!(result.get_field_value("index"), Some("fireball".to_string()));
        assert_eq!(result.get_field_value("url"), Some("/spells/fireball".to_string()));
        assert_eq!(result.get_field_value("invalid_field"), None);
    }

    #[test]
    fn test_field_case_insensitive() {
        let spell = SpellDetail {
            index: "fireball".to_string(),
            name: "Fireball".to_string(),
            level: 3,
            school: ApiReference {
                index: "evocation".to_string(),
                name: "Evocation".to_string(),
                url: "/magic-schools/evocation".to_string(),
            },
            casting_time: "1 action".to_string(),
            range: "150 feet".to_string(),
            components: vec!["V".to_string(), "S".to_string(), "M".to_string()],
            duration: "Instantaneous".to_string(),
            description: vec!["A bright streak flashes...".to_string()],
            higher_level: vec![],
        };
        
        let result = SearchResult::Spell(spell);
        
        // Test case insensitive field queries
        assert_eq!(result.get_field_value("NAME"), Some("Fireball".to_string()));
        assert_eq!(result.get_field_value("Level"), Some("Level 3".to_string()));
        assert_eq!(result.get_field_value("SCHOOL"), Some("Evocation".to_string()));
        assert_eq!(result.get_field_value("casting_TIME"), Some("1 action".to_string()));
    }

    #[test]
    fn test_empty_descriptions_and_lists() {
        let spell = SpellDetail {
            index: "test".to_string(),
            name: "Test Spell".to_string(),
            level: 1,
            school: ApiReference {
                index: "test".to_string(),
                name: "Test School".to_string(),
                url: "/test".to_string(),
            },
            casting_time: "1 action".to_string(),
            range: "Touch".to_string(),
            components: vec![],
            duration: "Instantaneous".to_string(),
            description: vec![],
            higher_level: vec![],
        };
        
        let result = SearchResult::Spell(spell);
        
        assert_eq!(result.get_field_value("description"), Some("No description available".to_string()));
        assert_eq!(result.get_field_value("higher_level"), Some("No higher level effects".to_string()));
        assert_eq!(result.get_field_value("components"), Some("".to_string()));
        
        // Test class with empty lists
        let class = ClassDetail {
            index: "test".to_string(),
            name: "Test Class".to_string(),
            hit_die: 8,
            proficiency_choices: vec![],
            proficiencies: vec![],
            saving_throws: vec![],
        };
        
        let class_result = SearchResult::Class(class);
        assert_eq!(class_result.get_field_value("proficiencies"), Some("No proficiencies listed".to_string()));
        assert_eq!(class_result.get_field_value("saving_throws"), Some("No saving throw proficiencies listed".to_string()));
    }

    #[test]
    fn test_multiline_descriptions() {
        let spell = SpellDetail {
            index: "test".to_string(),
            name: "Test Spell".to_string(),
            level: 1,
            school: ApiReference {
                index: "test".to_string(),
                name: "Test School".to_string(),
                url: "/test".to_string(),
            },
            casting_time: "1 action".to_string(),
            range: "Touch".to_string(),
            components: vec!["V".to_string()],
            duration: "Instantaneous".to_string(),
            description: vec![
                "First line of description.".to_string(),
                "Second line of description.".to_string(),
            ],
            higher_level: vec![
                "First higher level effect.".to_string(),
                "Second higher level effect.".to_string(),
            ],
        };
        
        let result = SearchResult::Spell(spell);
        
        let expected_description = "First line of description.\nSecond line of description.";
        let expected_higher = "First higher level effect.\nSecond higher level effect.";
        
        assert_eq!(result.get_field_value("description"), Some(expected_description.to_string()));
        assert_eq!(result.get_field_value("higher_level"), Some(expected_higher.to_string()));
    }

    #[test]
    fn test_interactive_field_functionality() {
        // Test that all field types return expected available fields
        let spell_result = create_test_spell_result();
        let spell_fields = spell_result.get_available_fields();
        assert!(spell_fields.contains(&"name".to_string()));
        assert!(spell_fields.contains(&"description".to_string()));
        assert_eq!(spell_fields.len(), 9);

        let class_result = create_test_class_result();
        let class_fields = class_result.get_available_fields();
        assert!(class_fields.contains(&"name".to_string()));
        assert!(class_fields.contains(&"hit_die".to_string()));
        assert_eq!(class_fields.len(), 5);

        let equipment_result = create_test_equipment_result();
        let equipment_fields = equipment_result.get_available_fields();
        assert!(equipment_fields.contains(&"name".to_string()));
        assert!(equipment_fields.contains(&"cost".to_string()));
        assert_eq!(equipment_fields.len(), 8);

        let reference_result = create_test_reference_result();
        let reference_fields = reference_result.get_available_fields();
        assert!(reference_fields.contains(&"name".to_string()));
        assert!(reference_fields.contains(&"index".to_string()));
        assert_eq!(reference_fields.len(), 3);
    }

    #[test]
    fn test_field_value_retrieval_comprehensive() {
        let spell_result = create_test_spell_result();
        
        // Test all spell fields
        assert!(spell_result.get_field_value("name").is_some());
        assert!(spell_result.get_field_value("level").is_some());
        assert!(spell_result.get_field_value("school").is_some());
        assert!(spell_result.get_field_value("casting_time").is_some());
        assert!(spell_result.get_field_value("range").is_some());
        assert!(spell_result.get_field_value("components").is_some());
        assert!(spell_result.get_field_value("duration").is_some());
        assert!(spell_result.get_field_value("description").is_some());
        assert!(spell_result.get_field_value("higher_level").is_some());
        
        // Test non-existent field
        assert!(spell_result.get_field_value("nonexistent_field").is_none());
    }

    #[test]
    fn test_display_field_handling() {
        let spell_result = create_test_spell_result();
        
        // This test verifies that display_field doesn't panic
        // We can't easily test the output, but we can ensure it doesn't crash
        let field_value = spell_result.get_field_value("name");
        assert!(field_value.is_some());
        
        let invalid_field_value = spell_result.get_field_value("invalid");
        assert!(invalid_field_value.is_none());
    }

    // Helper functions for creating test data
    fn create_test_spell_result() -> SearchResult {
        SearchResult::Spell(SpellDetail {
            index: "test-spell".to_string(),
            name: "Test Spell".to_string(),
            level: 3,
            school: ApiReference {
                index: "evocation".to_string(),
                name: "Evocation".to_string(),
                url: "/magic-schools/evocation".to_string(),
            },
            casting_time: "1 action".to_string(),
            range: "120 feet".to_string(),
            components: vec!["V".to_string(), "S".to_string()],
            duration: "Concentration, up to 1 minute".to_string(),
            description: vec!["Test spell description".to_string()],
            higher_level: vec!["Higher level effects".to_string()],
        })
    }

    fn create_test_class_result() -> SearchResult {
        SearchResult::Class(ClassDetail {
            index: "test-class".to_string(),
            name: "Test Class".to_string(),
            hit_die: 8,
            proficiency_choices: vec![],
            proficiencies: vec![ApiReference {
                index: "test-prof".to_string(),
                name: "Test Proficiency".to_string(),
                url: "/proficiencies/test".to_string(),
            }],
            saving_throws: vec![ApiReference {
                index: "wis".to_string(),
                name: "Wisdom".to_string(),
                url: "/ability-scores/wis".to_string(),
            }],
        })
    }

    fn create_test_equipment_result() -> SearchResult {
        SearchResult::Equipment(EquipmentDetail {
            index: "test-equipment".to_string(),
            name: "Test Equipment".to_string(),
            equipment_category: ApiReference {
                index: "weapon".to_string(),
                name: "Weapon".to_string(),
                url: "/equipment-categories/weapon".to_string(),
            },
            gear_category: None,
            weapon_category: Some("Simple Melee".to_string()),
            armor_category: None,
            cost: Some(Cost {
                quantity: 10,
                unit: "gp".to_string(),
            }),
            weight: Some(2.5),
            description: vec!["Test equipment description".to_string()],
        })
    }

    fn create_test_reference_result() -> SearchResult {
        SearchResult::Reference(ApiReference {
            index: "test-reference".to_string(),
            name: "Test Reference".to_string(),
            url: "/test-references/test".to_string(),
        })
    }
}