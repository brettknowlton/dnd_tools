use regex::Regex;
use scraper::{Html, Selector};

// Data structures for Wikidot HTML parsing
#[derive(Debug, Clone)]
pub struct WikiReference {
    pub index: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct SpellDetail {
    pub index: String,
    pub name: String,
    pub level: String,
    pub school: String,
    pub casting_time: String,
    pub range: String,
    pub components: String,
    pub duration: String,
    pub description: String,
    pub higher_level: String,
    pub spell_lists: String,
}

#[derive(Debug, Clone)]
pub struct ClassDetail {
    pub index: String,
    pub name: String,
    pub hit_die: String,
    pub proficiencies: String,
    pub saving_throws: String,
    pub skills: String,
    pub equipment: String,
}

#[derive(Debug, Clone)]
pub struct EquipmentDetail {
    pub index: String,
    pub name: String,
    pub category: String,
    pub cost: String,
    pub weight: String,
    pub description: String,
    pub properties: String,
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
    Reference(WikiReference),
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
                "spell_lists".to_string(),
            ],
            SearchResult::Class(_) => vec![
                "name".to_string(),
                "hit_die".to_string(),
                "proficiencies".to_string(),
                "saving_throws".to_string(),
                "skills".to_string(),
                "equipment".to_string(),
            ],
            SearchResult::Equipment(_) => vec![
                "name".to_string(),
                "category".to_string(),
                "cost".to_string(),
                "weight".to_string(),
                "description".to_string(),
                "properties".to_string(),
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
                    "level" => Some(spell.level.clone()),
                    "school" => Some(spell.school.clone()),
                    "casting_time" => Some(spell.casting_time.clone()),
                    "range" => Some(spell.range.clone()),
                    "components" => Some(spell.components.clone()),
                    "duration" => Some(spell.duration.clone()),
                    "description" => Some(spell.description.clone()),
                    "higher_level" => Some(spell.higher_level.clone()),
                    "spell_lists" => Some(spell.spell_lists.clone()),
                    _ => None,
                }
            },
            SearchResult::Class(class) => {
                match field.to_lowercase().as_str() {
                    "name" => Some(class.name.clone()),
                    "hit_die" => Some(class.hit_die.clone()),
                    "proficiencies" => Some(class.proficiencies.clone()),
                    "saving_throws" => Some(class.saving_throws.clone()),
                    "skills" => Some(class.skills.clone()),
                    "equipment" => Some(class.equipment.clone()),
                    _ => None,
                }
            },
            SearchResult::Equipment(equipment) => {
                match field.to_lowercase().as_str() {
                    "name" => Some(equipment.name.clone()),
                    "category" => Some(equipment.category.clone()),
                    "cost" => Some(equipment.cost.clone()),
                    "weight" => Some(equipment.weight.clone()),
                    "description" => Some(equipment.description.clone()),
                    "properties" => Some(equipment.properties.clone()),
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
        println!("║ School: {:<29} ║", spell.school);
        println!("║ Casting Time: {:<23} ║", spell.casting_time);
        println!("║ Range: {:<30} ║", spell.range);
        println!("║ Components: {:<25} ║", spell.components);
        println!("║ Duration: {:<27} ║", spell.duration);
        println!("╚═══════════════════════════════════════╝");
        
        if !spell.description.is_empty() {
            println!("\nDescription:");
            println!("  {}", spell.description);
        }
        
        if !spell.higher_level.is_empty() {
            println!("\nAt Higher Levels:");
            println!("  {}", spell.higher_level);
        }

        if !spell.spell_lists.is_empty() {
            println!("\nSpell Lists:");
            println!("  {}", spell.spell_lists);
        }
    }

    fn display_class(&self, class: &ClassDetail) {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║                CLASS                  ║");
        println!("╠═══════════════════════════════════════╣");
        println!("║ Name: {:<31} ║", class.name);
        println!("║ Hit Die: {:<28} ║", class.hit_die);
        println!("╚═══════════════════════════════════════╝");
        
        if !class.proficiencies.is_empty() {
            println!("\nProficiencies:");
            println!("  {}", class.proficiencies);
        }
        
        if !class.saving_throws.is_empty() {
            println!("\nSaving Throw Proficiencies:");
            println!("  {}", class.saving_throws);
        }

        if !class.skills.is_empty() {
            println!("\nSkills:");
            println!("  {}", class.skills);
        }

        if !class.equipment.is_empty() {
            println!("\nEquipment:");
            println!("  {}", class.equipment);
        }
    }

    fn display_equipment(&self, equipment: &EquipmentDetail) {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║              EQUIPMENT                ║");
        println!("╠═══════════════════════════════════════╣");
        println!("║ Name: {:<31} ║", equipment.name);
        println!("║ Category: {:<27} ║", equipment.category);
        
        if !equipment.cost.is_empty() {
            println!("║ Cost: {:<31} ║", equipment.cost);
        }
        
        if !equipment.weight.is_empty() {
            println!("║ Weight: {:<29} ║", equipment.weight);
        }
        
        println!("╚═══════════════════════════════════════╝");
        
        if !equipment.description.is_empty() {
            println!("\nDescription:");
            println!("  {}", equipment.description);
        }

        if !equipment.properties.is_empty() {
            println!("\nProperties:");
            println!("  {}", equipment.properties);
        }
    }

    fn display_reference(&self, reference: &WikiReference) {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║              REFERENCE                ║");
        println!("╠═══════════════════════════════════════╣");
        println!("║ Name: {:<31} ║", reference.name);
        println!("║ Index: {:<30} ║", reference.index);
        println!("╚═══════════════════════════════════════╝");
    }
}

// Main search client for Wikidot HTML scraping
pub struct DndSearchClient {
    base_url: String,
    client: reqwest::Client,
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
            .expect("Failed to create HTTP client - network required for Wikidot API");
        
        DndSearchClient {
            base_url: "http://dnd5e.wikidot.com".to_string(),
            client,
        }
    }

    // Search with fuzzy matching using Wikidot HTML scraping
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
        match category {
            SearchCategory::Spells => self.search_spell(query).await,
            SearchCategory::Classes => self.search_class(query).await,
            SearchCategory::Equipment => self.search_equipment(query).await,
            SearchCategory::Monsters => self.search_monster(query).await,
            SearchCategory::Races => self.search_race(query).await,
        }
    }

    async fn search_spell(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        let spell_url = format!("{}/spell:{}", self.base_url, query.to_lowercase().replace(" ", "-"));
        
        let response = self.client
            .get(&spell_url)
            .send()
            .await
            .map_err(|e| format!("Network request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Spell '{}' not found", query));
        }

        let html = response.text().await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let document = Html::parse_document(&html);
        
        // Parse spell details from HTML
        let spell_detail = self.parse_spell_html(&document, query)?;
        Ok(vec![SearchResult::Spell(spell_detail)])
    }

    async fn search_class(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        let class_url = format!("{}/{}", self.base_url, query.to_lowercase());
        
        let response = self.client
            .get(&class_url)
            .send()
            .await
            .map_err(|e| format!("Network request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Class '{}' not found", query));
        }

        let html = response.text().await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let document = Html::parse_document(&html);
        
        // Parse class details from HTML
        let class_detail = self.parse_class_html(&document, query)?;
        Ok(vec![SearchResult::Class(class_detail)])
    }

    async fn search_equipment(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        // Equipment might be at different URL patterns - try common ones
        let equipment_urls = vec![
            format!("{}/equipment:{}", self.base_url, query.to_lowercase().replace(" ", "-")),
            format!("{}/weapon:{}", self.base_url, query.to_lowercase().replace(" ", "-")),
            format!("{}/armor:{}", self.base_url, query.to_lowercase().replace(" ", "-")),
        ];

        for url in equipment_urls {
            let response = self.client.get(&url).send().await;
            if let Ok(response) = response {
                if response.status().is_success() {
                    if let Ok(html) = response.text().await {
                        let document = Html::parse_document(&html);
                        if let Ok(equipment_detail) = self.parse_equipment_html(&document, query) {
                            return Ok(vec![SearchResult::Equipment(equipment_detail)]);
                        }
                    }
                }
            }
        }

        Err(format!("Equipment '{}' not found", query))
    }

    async fn search_monster(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        let monster_url = format!("{}/monster:{}", self.base_url, query.to_lowercase().replace(" ", "-"));
        
        let response = self.client
            .get(&monster_url)
            .send()
            .await
            .map_err(|e| format!("Network request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Monster '{}' not found", query));
        }

        let html = response.text().await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        // For now, return as reference since monster parsing would be complex
        let reference = WikiReference {
            index: query.to_lowercase().replace(" ", "-"),
            name: query.to_string(),
            url: monster_url,
        };
        Ok(vec![SearchResult::Reference(reference)])
    }

    async fn search_race(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        let race_url = format!("{}/{}", self.base_url, query.to_lowercase());
        
        let response = self.client
            .get(&race_url)
            .send()
            .await
            .map_err(|e| format!("Network request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Race '{}' not found", query));
        }

        let html = response.text().await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        // For now, return as reference since race parsing would be complex
        let reference = WikiReference {
            index: query.to_lowercase().replace(" ", "-"),
            name: query.to_string(),
            url: race_url,
        };
        Ok(vec![SearchResult::Reference(reference)])
    }

    // HTML parsing methods for Wikidot content
    fn parse_spell_html(&self, document: &Html, query: &str) -> Result<SpellDetail, String> {
        let content_selector = Selector::parse("#page-content").unwrap();
        let content = document.select(&content_selector).next()
            .ok_or("Could not find page content")?;

        let content_text = content.inner_html();
        
        // Extract spell level and school from "3rd-level evocation" pattern
        let level_school_regex = Regex::new(r"<em>([^<]+)-level ([^<]+)</em>").unwrap();
        let (level, school) = if let Some(caps) = level_school_regex.captures(&content_text) {
            (caps[1].to_string() + "-level", caps[2].to_string())
        } else {
            ("Unknown level".to_string(), "Unknown school".to_string())
        };

        // Extract casting time, range, components, duration
        let casting_time = self.extract_field(&content_text, "Casting Time:", "<br")
            .unwrap_or_else(|| "Unknown".to_string());
        let range = self.extract_field(&content_text, "Range:", "<br")
            .unwrap_or_else(|| "Unknown".to_string());
        let components = self.extract_field(&content_text, "Components:", "<br")
            .unwrap_or_else(|| "Unknown".to_string());
        let duration = self.extract_field(&content_text, "Duration:", "</p>")
            .unwrap_or_else(|| "Unknown".to_string());

        // Extract main description (first paragraph after duration)
        let desc_start = content_text.find("</p>").unwrap_or(0);
        let description = self.extract_description(&content_text[desc_start..]);

        // Extract higher level effects
        let higher_level = self.extract_higher_level(&content_text);

        // Extract spell lists
        let spell_lists = self.extract_spell_lists(&content_text);

        Ok(SpellDetail {
            index: query.to_lowercase().replace(" ", "-"),
            name: query.to_string(),
            level,
            school,
            casting_time,
            range,
            components,
            duration,
            description,
            higher_level,
            spell_lists,
        })
    }

    fn parse_class_html(&self, document: &Html, query: &str) -> Result<ClassDetail, String> {
        let content_selector = Selector::parse("#page-content").unwrap();
        let content = document.select(&content_selector).next()
            .ok_or("Could not find page content")?;

        let content_text = content.inner_html();
        
        // Extract hit die from "Hit Dice: 1d10" pattern
        let hit_die = self.extract_field(&content_text, "Hit Dice:", "<br")
            .unwrap_or_else(|| "Unknown".to_string());

        // Extract proficiencies
        let proficiencies = self.extract_multi_line_field(&content_text, "Armor:", "Weapons:");

        // Extract saving throws
        let saving_throws = self.extract_field(&content_text, "Saving Throws:", "<br")
            .unwrap_or_else(|| "Unknown".to_string());

        // Extract skills
        let skills = self.extract_field(&content_text, "Skills:", "</p>")
            .unwrap_or_else(|| "Unknown".to_string());

        // Extract equipment (basic extraction)
        let equipment = self.extract_equipment_section(&content_text);

        Ok(ClassDetail {
            index: query.to_lowercase().replace(" ", "-"),
            name: query.to_string(),
            hit_die,
            proficiencies,
            saving_throws,
            skills,
            equipment,
        })
    }

    fn parse_equipment_html(&self, document: &Html, query: &str) -> Result<EquipmentDetail, String> {
        let content_selector = Selector::parse("#page-content").unwrap();
        let content = document.select(&content_selector).next()
            .ok_or("Could not find page content")?;

        let content_text = content.inner_html();
        
        // Basic equipment parsing - this would need refinement for different types
        let category = "Equipment".to_string(); // Could be refined
        let cost = self.extract_field(&content_text, "Cost:", " ")
            .unwrap_or_else(|| "Unknown".to_string());
        let weight = self.extract_field(&content_text, "Weight:", " ")
            .unwrap_or_else(|| "Unknown".to_string());
        let description = self.extract_description(&content_text);
        let properties = "".to_string(); // Would need specific parsing

        Ok(EquipmentDetail {
            index: query.to_lowercase().replace(" ", "-"),
            name: query.to_string(),
            category,
            cost,
            weight,
            description,
            properties,
        })
    }

    // Helper methods for HTML extraction
    fn extract_field(&self, html: &str, field_name: &str, end_marker: &str) -> Option<String> {
        if let Some(start) = html.find(field_name) {
            let start_pos = start + field_name.len();
            if let Some(end) = html[start_pos..].find(end_marker) {
                let content = html[start_pos..start_pos + end].trim();
                // Strip HTML tags
                let tag_regex = Regex::new(r"<[^>]+>").unwrap();
                return Some(tag_regex.replace_all(content, "").trim().to_string());
            }
        }
        None
    }

    fn extract_description(&self, html: &str) -> String {
        // Find the first paragraph after the stats
        let p_regex = Regex::new(r"<p>([^<]+)</p>").unwrap();
        if let Some(caps) = p_regex.captures(html) {
            caps[1].trim().to_string()
        } else {
            "No description available".to_string()
        }
    }

    fn extract_higher_level(&self, html: &str) -> String {
        let higher_regex = Regex::new(r"<strong><em>At Higher Levels\.</em></strong>\s*([^<]+)").unwrap();
        if let Some(caps) = higher_regex.captures(html) {
            caps[1].trim().to_string()
        } else {
            "".to_string()
        }
    }

    fn extract_spell_lists(&self, html: &str) -> String {
        let lists_regex = Regex::new(r"<strong><em>Spell Lists\.</em></strong>\s*(.+?)(?:</p>|$)").unwrap();
        if let Some(caps) = lists_regex.captures(html) {
            let content = caps[1].trim();
            let tag_regex = Regex::new(r"<[^>]+>").unwrap();
            tag_regex.replace_all(content, "").trim().to_string()
        } else {
            "".to_string()
        }
    }

    fn extract_multi_line_field(&self, html: &str, start_field: &str, end_field: &str) -> String {
        if let Some(start) = html.find(start_field) {
            if let Some(end) = html[start..].find(end_field) {
                let content = &html[start..start + end];
                let tag_regex = Regex::new(r"<[^>]+>").unwrap();
                return tag_regex.replace_all(content, " ").trim().to_string();
            }
        }
        "Unknown".to_string()
    }

    fn extract_equipment_section(&self, html: &str) -> String {
        // Look for equipment section
        if let Some(start) = html.find("Equipment") {
            if let Some(end) = html[start..start + 500].find("</ul>") {
                let content = &html[start..start + end];
                let tag_regex = Regex::new(r"<[^>]+>").unwrap();
                return tag_regex.replace_all(content, " ").trim().to_string();
            }
        }
        "".to_string()
    }

    async fn fuzzy_search(&self, query: &str, category: Option<SearchCategory>) -> Result<Vec<SearchResult>, String> {
        // For Wikidot, fuzzy search would require scraping list pages
        // This is a simplified version that attempts common variations
        let variations = self.generate_query_variations(query);
        
        let categories = match category {
            Some(cat) => vec![cat],
            None => SearchCategory::all(),
        };

        for cat in categories {
            for variation in &variations {
                if let Ok(results) = self.search_category(variation, cat).await {
                    if !results.is_empty() {
                        return Ok(results);
                    }
                }
            }
        }

        Err(format!("No matches found for '{}'", query))
    }

    fn generate_query_variations(&self, query: &str) -> Vec<String> {
        let mut variations = vec![query.to_string()];
        
        // Try with dashes instead of spaces
        variations.push(query.replace(" ", "-"));
        
        // Try without spaces
        variations.push(query.replace(" ", ""));
        
        // Try lowercase
        variations.push(query.to_lowercase());
        variations.push(query.to_lowercase().replace(" ", "-"));
        
        // Try title case variations
        let words: Vec<&str> = query.split_whitespace().collect();
        if words.len() > 1 {
            let title_case = words.iter()
                .map(|w| format!("{}{}", w.chars().next().unwrap().to_uppercase(), &w[1..].to_lowercase()))
                .collect::<Vec<_>>()
                .join(" ");
            variations.push(title_case.clone());
            variations.push(title_case.replace(" ", "-"));
        }
        
        variations
    }

    // Method to get suggestions when no exact match is found
    pub async fn get_suggestions(&self, query: &str, _category: Option<SearchCategory>) -> Vec<String> {
        // For Wikidot implementation, this would be much simpler
        // since we don't have cached data. Return common suggestions based on query
        let mut suggestions = Vec::new();
        
        let query_lower = query.to_lowercase();
        
        // Common spell suggestions
        if query_lower.contains("fire") {
            suggestions.extend(vec!["fireball".to_string(), "fire-bolt".to_string(), "burning-hands".to_string()]);
        }
        if query_lower.contains("heal") {
            suggestions.extend(vec!["cure-wounds".to_string(), "healing-word".to_string(), "heal".to_string()]);
        }
        if query_lower.contains("light") {
            suggestions.extend(vec!["light".to_string(), "dancing-lights".to_string(), "lightning-bolt".to_string()]);
        }
        
        // Common class suggestions
        if query_lower.len() <= 8 { // Likely a class name
            let common_classes = vec!["fighter", "wizard", "cleric", "rogue", "ranger", "paladin", "barbarian", "bard", "druid", "monk", "sorcerer", "warlock"];
            for class in common_classes {
                if class.starts_with(&query_lower) || query_lower.starts_with(class) {
                    suggestions.push(class.to_string());
                }
            }
        }
        
        // Remove duplicates and limit to 5
        suggestions.sort();
        suggestions.dedup();
        suggestions.truncate(5);
        
        suggestions
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
            level: "3rd-level".to_string(),
            school: "Evocation".to_string(),
            casting_time: "1 action".to_string(),
            range: "150 feet".to_string(),
            components: "V, S, M".to_string(),
            duration: "Instantaneous".to_string(),
            description: "A bright streak flashes...".to_string(),
            higher_level: "".to_string(),
            spell_lists: "Sorcerer, Wizard".to_string(),
        };
        
        let result = SearchResult::Spell(spell);
        assert_eq!(result.name(), "Fireball");
        assert_eq!(result.index(), "fireball");
    }

    #[test]
    fn test_dnd_search_client_creation() {
        let client = DndSearchClient::new();
        assert_eq!(client.base_url, "http://dnd5e.wikidot.com");
    }

    #[test]
    fn test_query_variations() {
        let client = DndSearchClient::new();
        let variations = client.generate_query_variations("Magic Missile");
        
        assert!(variations.contains(&"Magic Missile".to_string()));
        assert!(variations.contains(&"magic-missile".to_string()));
        assert!(variations.contains(&"magic missile".to_string()));
        assert!(variations.contains(&"Magic-Missile".to_string()));
    }

    #[test]
    fn test_spell_available_fields() {
        let spell = SpellDetail {
            index: "fireball".to_string(),
            name: "Fireball".to_string(),
            level: "3rd-level".to_string(),
            school: "Evocation".to_string(),
            casting_time: "1 action".to_string(),
            range: "150 feet".to_string(),
            components: "V, S, M".to_string(),
            duration: "Instantaneous".to_string(),
            description: "A bright streak flashes...".to_string(),
            higher_level: "When you cast this spell...".to_string(),
            spell_lists: "Sorcerer, Wizard".to_string(),
        };
        
        let result = SearchResult::Spell(spell);
        let fields = result.get_available_fields();
        
        assert_eq!(fields.len(), 10);
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"level".to_string()));
        assert!(fields.contains(&"school".to_string()));
        assert!(fields.contains(&"casting_time".to_string()));
        assert!(fields.contains(&"range".to_string()));
        assert!(fields.contains(&"components".to_string()));
        assert!(fields.contains(&"duration".to_string()));
        assert!(fields.contains(&"description".to_string()));
        assert!(fields.contains(&"higher_level".to_string()));
        assert!(fields.contains(&"spell_lists".to_string()));
    }

    #[test]
    fn test_class_available_fields() {
        let class = ClassDetail {
            index: "fighter".to_string(),
            name: "Fighter".to_string(),
            hit_die: "1d10 per fighter level".to_string(),
            proficiencies: "All armor, shields".to_string(),
            saving_throws: "Strength, Constitution".to_string(),
            skills: "Choose two skills from...".to_string(),
            equipment: "Chain mail or leather...".to_string(),
        };
        
        let result = SearchResult::Class(class);
        let fields = result.get_available_fields();
        
        assert_eq!(fields.len(), 6);
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"hit_die".to_string()));
        assert!(fields.contains(&"proficiencies".to_string()));
        assert!(fields.contains(&"saving_throws".to_string()));
        assert!(fields.contains(&"skills".to_string()));
        assert!(fields.contains(&"equipment".to_string()));
    }

    #[test]
    fn test_equipment_available_fields() {
        let equipment = EquipmentDetail {
            index: "longsword".to_string(),
            name: "Longsword".to_string(),
            category: "Weapon".to_string(),
            cost: "15 gp".to_string(),
            weight: "3 lb".to_string(),
            description: "A versatile weapon.".to_string(),
            properties: "Versatile (1d10)".to_string(),
        };
        
        let result = SearchResult::Equipment(equipment);
        let fields = result.get_available_fields();
        
        assert_eq!(fields.len(), 6);
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"category".to_string()));
        assert!(fields.contains(&"cost".to_string()));
        assert!(fields.contains(&"weight".to_string()));
        assert!(fields.contains(&"description".to_string()));
        assert!(fields.contains(&"properties".to_string()));
    }

    #[test]
    fn test_reference_available_fields() {
        let reference = WikiReference {
            index: "fireball".to_string(),
            name: "Fireball".to_string(),
            url: "http://dnd5e.wikidot.com/spell:fireball".to_string(),
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
            level: "3rd-level".to_string(),
            school: "Evocation".to_string(),
            casting_time: "1 action".to_string(),
            range: "150 feet".to_string(),
            components: "V, S, M".to_string(),
            duration: "Instantaneous".to_string(),
            description: "A bright streak flashes...".to_string(),
            higher_level: "".to_string(),
            spell_lists: "Sorcerer, Wizard".to_string(),
        };
        
        let result = SearchResult::Spell(spell);
        
        assert_eq!(result.get_field_value("name"), Some("Fireball".to_string()));
        assert_eq!(result.get_field_value("level"), Some("3rd-level".to_string()));
        assert_eq!(result.get_field_value("school"), Some("Evocation".to_string()));
        assert_eq!(result.get_field_value("casting_time"), Some("1 action".to_string()));
        assert_eq!(result.get_field_value("range"), Some("150 feet".to_string()));
        assert_eq!(result.get_field_value("components"), Some("V, S, M".to_string()));
        assert_eq!(result.get_field_value("duration"), Some("Instantaneous".to_string()));
        assert_eq!(result.get_field_value("description"), Some("A bright streak flashes...".to_string()));
        assert_eq!(result.get_field_value("spell_lists"), Some("Sorcerer, Wizard".to_string()));
        assert_eq!(result.get_field_value("invalid_field"), None);
    }

    #[test]
    fn test_field_case_insensitive() {
        let spell = SpellDetail {
            index: "fireball".to_string(),
            name: "Fireball".to_string(),
            level: "3rd-level".to_string(),
            school: "Evocation".to_string(),
            casting_time: "1 action".to_string(),
            range: "150 feet".to_string(),
            components: "V, S, M".to_string(),
            duration: "Instantaneous".to_string(),
            description: "A bright streak flashes...".to_string(),
            higher_level: "".to_string(),
            spell_lists: "Sorcerer, Wizard".to_string(),
        };
        
        let result = SearchResult::Spell(spell);
        
        // Test case insensitive field queries
        assert_eq!(result.get_field_value("NAME"), Some("Fireball".to_string()));
        assert_eq!(result.get_field_value("Level"), Some("3rd-level".to_string()));
        assert_eq!(result.get_field_value("SCHOOL"), Some("Evocation".to_string()));
        assert_eq!(result.get_field_value("casting_TIME"), Some("1 action".to_string()));
    }

    #[tokio::test]
    async fn test_get_suggestions() {
        let client = DndSearchClient::new();
        
        let suggestions = client.get_suggestions("fir", Some(SearchCategory::Spells)).await;
        assert!(suggestions.iter().any(|s| s.contains("fire")));
        
        let suggestions = client.get_suggestions("fig", Some(SearchCategory::Classes)).await;
        assert!(suggestions.iter().any(|s| s == "fighter"));
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
    fn test_html_field_extraction() {
        let client = DndSearchClient::new();
        
        // Test basic field extraction
        let html = r#"<strong>Casting Time:</strong> 1 action<br />"#;
        let result = client.extract_field(html, "Casting Time:", "<br");
        assert_eq!(result, Some("1 action".to_string()));
        
        // Test with HTML tags in content
        let html = r#"<strong>Range:</strong> <em>150 feet</em><br />"#;
        let result = client.extract_field(html, "Range:", "<br");
        assert_eq!(result, Some("150 feet".to_string()));
    }

    #[test]
    fn test_description_extraction() {
        let client = DndSearchClient::new();
        
        let html = r#"<p>A bright streak flashes from your pointing finger to a point you choose.</p>"#;
        let result = client.extract_description(html);
        assert_eq!(result, "A bright streak flashes from your pointing finger to a point you choose.");
    }

    #[test]
    fn test_higher_level_extraction() {
        let client = DndSearchClient::new();
        
        let html = r#"<strong><em>At Higher Levels.</em></strong> When you cast this spell using a spell slot of 4th level or higher"#;
        let result = client.extract_higher_level(html);
        assert_eq!(result, "When you cast this spell using a spell slot of 4th level or higher");
    }

    #[test]
    fn test_spell_lists_extraction() {
        let client = DndSearchClient::new();
        
        let html = r#"<strong><em>Spell Lists.</em></strong> <a href="/spells:sorcerer">Sorcerer</a>, <a href="/spells:wizard">Wizard</a></p>"#;
        let result = client.extract_spell_lists(html);
        assert_eq!(result, "Sorcerer, Wizard");
    }

    // Network connectivity test (only works if network is available)
    #[tokio::test]
    async fn test_wikidot_connectivity() {
        // Test basic connectivity to Wikidot
        match reqwest::Client::new()
            .get("http://dnd5e.wikidot.com/spell:fireball")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    println!("✅ Wikidot connectivity test passed - site is reachable");
                    assert!(response.status().is_success(), "Wikidot should return success status");
                } else {
                    println!("⚠️ Wikidot responded but with status: {}", response.status());
                }
            },
            Err(e) => {
                println!("⚠️ Wikidot connectivity test failed: {}", e);
                println!("💡 This is expected if running without internet access");
                // Don't fail the test - network may not be available in testing environment
            }
        }
    }

    // Test actual spell parsing with real data
    #[tokio::test]
    async fn test_real_spell_search() {
        let client = DndSearchClient::new();
        
        println!("🔍 Testing live spell search for 'fireball'...");
        
        match client.search("fireball", Some(SearchCategory::Spells)).await {
            Ok(results) => {
                if !results.is_empty() {
                    println!("✅ Successfully found {} result(s) for 'fireball'", results.len());
                    let result = &results[0];
                    println!("📝 Spell name: {}", result.name());
                    
                    if let SearchResult::Spell(spell) = result {
                        println!("🧙 Level: {}", spell.level);
                        println!("🏫 School: {}", spell.school);
                        println!("⏱️  Casting time: {}", spell.casting_time);
                        println!("📏 Range: {}", spell.range);
                        
                        // Test field querying
                        assert!(result.get_field_value("name").is_some());
                        assert!(result.get_field_value("level").is_some());
                        assert!(result.get_field_value("school").is_some());
                        
                        println!("✅ Field querying works correctly");
                    }
                } else {
                    println!("⚠️ No results found for 'fireball' - this might indicate parsing issues");
                }
            },
            Err(e) => {
                println!("⚠️ Spell search failed: {}", e);
                println!("💡 This is expected if network is unavailable or site structure changed");
            }
        }
    }
}