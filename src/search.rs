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

// Display functions
impl SearchResult {
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
}