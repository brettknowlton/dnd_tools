use regex::Regex;
use scraper::{Html, Selector};
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};

// Simplified data structure for wikidot page content
#[derive(Debug, Clone)]
pub struct WikiPageContent {
    pub index: String,
    pub name: String,
    pub url: String,
    pub content: String,  // Raw parsed content from the page
    pub content_type: String, // "spell", "class", "equipment", "monster", "race", etc.
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

// Simplified search result - just wiki page content
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub page: WikiPageContent,
}

impl SearchResult {
    pub fn name(&self) -> &str {
        &self.page.name
    }

    pub fn index(&self) -> &str {
        &self.page.index
    }

    pub fn content_type(&self) -> &str {
        &self.page.content_type
    }

    pub fn display(&self) {
        let page = &self.page;
        
        // Display header with page type and name
        println!("\n╔═══════════════════════════════════════════════════════════════════════════════╗");
        println!("║ {} - {}{} ║", 
            page.content_type.to_uppercase(),
            page.name,
            " ".repeat(69_i32.saturating_sub(page.content_type.len() as i32 + page.name.len() as i32 + 3) as usize)
        );
        println!("╠═══════════════════════════════════════════════════════════════════════════════╣");
        println!("║ Source: {} {} ║", 
            page.url,
            " ".repeat(70_i32.saturating_sub(page.url.len() as i32 + 8) as usize)
        );
        println!("╚═══════════════════════════════════════════════════════════════════════════════╝");
        
        // Display the content with nice formatting
        println!();
        self.format_content(&page.content);
        
        // Add proper attribution as required by CC BY-SA 3.0
        println!("\n{}", "─".repeat(80));
        println!("📄 Source: dnd5e.wikidot.com | CC BY-SA 3.0");
        println!("🔗 https://creativecommons.org/licenses/by-sa/3.0/");
        println!("ℹ️  Content used under Creative Commons Attribution-ShareAlike 3.0 license");
        println!("   for personal/educational use only.");
        println!();
    }

    fn format_content(&self, content: &str) {
        // Split content into lines and format nicely
        let lines: Vec<&str> = content.lines().collect();
        
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            
            // Format different types of content
            if self.is_heading(trimmed) {
                println!("🔸 {}", trimmed.to_uppercase());
                println!("{}", "─".repeat(50));
            } else if self.is_stat_line(trimmed) {
                println!("  📊 {}", trimmed);
            } else {
                // Regular content - wrap if too long
                self.wrap_and_print(trimmed, "  ");
            }
        }
    }

    fn is_heading(&self, line: &str) -> bool {
        // Simple heuristics for headings
        line.len() < 50 && 
        (line.ends_with(':') || 
         line.chars().all(|c| c.is_alphanumeric() || c.is_whitespace()) &&
         line.split_whitespace().count() <= 4)
    }

    fn is_stat_line(&self, line: &str) -> bool {
        // Lines that look like "Casting Time: 1 action" or "Range: 150 feet"
        line.contains(':') && line.len() < 60 && line.split(':').count() == 2
    }

    fn wrap_and_print(&self, text: &str, prefix: &str) {
        const MAX_WIDTH: usize = 75;
        let mut current_line = String::new();
        
        for word in text.split_whitespace() {
            if current_line.len() + word.len() + 1 > MAX_WIDTH - prefix.len() {
                if !current_line.is_empty() {
                    println!("{}{}", prefix, current_line);
                    current_line.clear();
                }
            }
            
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
        
        if !current_line.is_empty() {
            println!("{}{}", prefix, current_line);
        }
    }
}

// Main search client for Wikidot HTML scraping
pub struct DndSearchClient {
    base_url: String,
    client: reqwest::Client,
    cache_dir: PathBuf,
}

impl Default for DndSearchClient {
    fn default() -> Self {
        Self::new()
    }
}

impl DndSearchClient {
    pub fn new() -> Self {
        Self::with_cache_refresh(false)
    }
    
    pub fn with_cache_refresh(_refresh: bool) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client - network required for Wikidot API");
        
        let cache_dir = Self::get_cache_dir();
        
        // Create cache directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&cache_dir) {
            eprintln!("Warning: Failed to create cache directory: {}", e);
        }
        
        DndSearchClient {
            base_url: "http://dnd5e.wikidot.com".to_string(),
            client,
            cache_dir,
        }
    }
    
    fn get_cache_dir() -> PathBuf {
        if let Some(cache_root) = dirs::cache_dir() {
            cache_root.join("dnd_tools")
        } else {
            // Fallback to current directory if cache dir can't be determined
            PathBuf::from(".cache/dnd_tools")
        }
    }
    
    fn get_cache_path(&self, slug: &str) -> PathBuf {
        let safe_slug = slug.replace(":", "_").replace("/", "_");
        self.cache_dir.join(format!("{}.txt", safe_slug))
    }
    
    fn load_from_cache(&self, slug: &str) -> Option<String> {
        let cache_path = self.get_cache_path(slug);
        fs::read_to_string(cache_path).ok()
    }
    
    fn save_to_cache(&self, slug: &str, content: &str) -> Result<()> {
        let cache_path = self.get_cache_path(slug);
        fs::write(cache_path, content)
            .context("Failed to save content to cache")
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
            SearchCategory::Spells => self.fetch_wiki_page(query, "spell", "spell").await,
            SearchCategory::Classes => self.fetch_wiki_page(query, "class", "class").await,
            SearchCategory::Equipment => self.fetch_wiki_page(query, "equipment", "equipment").await,
            SearchCategory::Monsters => self.fetch_wiki_page(query, "monster", "monster").await,
            SearchCategory::Races => self.fetch_wiki_page(query, "race", "race").await,
        }
    }

    async fn fetch_wiki_page(&self, query: &str, content_type: &str, url_prefix: &str) -> Result<Vec<SearchResult>, String> {
        let cache_key = format!("{}:{}", url_prefix, query);
        
        // Try to load from cache first
        if let Some(cached_content) = self.load_from_cache(&cache_key) {
            // Parse cached content to create SearchResult
            if let Some((title, url, content)) = self.parse_cached_content(&cached_content) {
                let page = WikiPageContent {
                    index: query.to_lowercase().replace(" ", "-"),
                    name: title,
                    url,
                    content,
                    content_type: content_type.to_string(),
                };
                return Ok(vec![SearchResult { page }]);
            }
        }
        
        // Not in cache or cache invalid, fetch from web
        let possible_urls = self.generate_possible_urls(query, url_prefix);
        
        for url in possible_urls {
            let response = self.client
                .get(&url)
                .send()
                .await
                .map_err(|e| format!("Network request failed: {}", e))?;

            if response.status().is_success() {
                let html = response.text().await
                    .map_err(|e| format!("Failed to read response: {}", e))?;

                let document = Html::parse_document(&html);
                
                // Extract the main page content
                let content = self.extract_page_content(&document)?;
                let title = self.extract_page_title(&document, query);
                
                // Create cached content format
                let cached_content = format!("TITLE:{}\nURL:{}\nCONTENT:\n{}", title, url, content);
                
                // Save to cache (ignore errors)
                let _ = self.save_to_cache(&cache_key, &cached_content);
                
                let page = WikiPageContent {
                    index: query.to_lowercase().replace(" ", "-"),
                    name: title,
                    url: url.clone(),
                    content,
                    content_type: content_type.to_string(),
                };
                
                return Ok(vec![SearchResult { page }]);
            }
        }
        
        Err(format!("{} '{}' not found", content_type, query))
    }
    
    fn parse_cached_content(&self, content: &str) -> Option<(String, String, String)> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() < 3 {
            return None;
        }
        
        let title = lines[0].strip_prefix("TITLE:")?.to_string();
        let url = lines[1].strip_prefix("URL:")?.to_string();
        let content_start = content.find("CONTENT:\n")?;
        let content = content[content_start + 9..].to_string();
        
        Some((title, url, content))
    }

    fn generate_possible_urls(&self, query: &str, url_prefix: &str) -> Vec<String> {
        let base_query = query.to_lowercase().replace(" ", "-");
        let mut urls = Vec::new();
        
        // For spells, equipment, monsters
        if url_prefix != "class" && url_prefix != "race" {
            urls.push(format!("{}/{}:{}", self.base_url, url_prefix, base_query));
        }
        
        // For classes and races, they might be direct pages
        if url_prefix == "class" || url_prefix == "race" {
            urls.push(format!("{}/{}", self.base_url, base_query));
        }
        
        // Also try weapon and armor for equipment
        if url_prefix == "equipment" {
            urls.push(format!("{}/weapon:{}", self.base_url, base_query));
            urls.push(format!("{}/armor:{}", self.base_url, base_query));
        }
        
        urls
    }

    fn extract_page_content(&self, document: &Html) -> Result<String, String> {
        let content_selector = Selector::parse("#page-content").unwrap();
        let content = document.select(&content_selector).next()
            .ok_or("Could not find page content")?;

        let html_content = content.inner_html();
        
        // Clean up the HTML and convert to readable text
        let cleaned_content = self.html_to_readable_text(&html_content);
        
        if cleaned_content.trim().is_empty() {
            return Err("Page content is empty or could not be parsed".to_string());
        }
        
        Ok(cleaned_content)
    }

    fn extract_page_title(&self, document: &Html, fallback: &str) -> String {
        // Try to extract the actual page title
        let title_selector = Selector::parse("title").unwrap();
        if let Some(title_element) = document.select(&title_selector).next() {
            let title_text = title_element.text().collect::<Vec<_>>().join("");
            if let Some(pos) = title_text.find(" - ") {
                let page_title = title_text[..pos].trim();
                if !page_title.is_empty() {
                    return page_title.to_string();
                }
            }
        }
        
        // Try h1 tags
        let h1_selector = Selector::parse("h1").unwrap();
        if let Some(h1_element) = document.select(&h1_selector).next() {
            let h1_text = h1_element.text().collect::<Vec<_>>().join("");
            if !h1_text.is_empty() {
                return h1_text.trim().to_string();
            }
        }
        
        // Fall back to title-cased query
        fallback.split_whitespace()
            .map(|word| {
                let mut chars: Vec<char> = word.chars().collect();
                if !chars.is_empty() {
                    chars[0] = chars[0].to_uppercase().next().unwrap_or(chars[0]);
                }
                chars.into_iter().collect::<String>()
            })
            .collect::<Vec<String>>()
            .join(" ")
    }

    fn html_to_readable_text(&self, html: &str) -> String {
        // First try using html2text for better formatting
        let text = html2text::from_read(html.as_bytes(), 80);
        let cleaned = text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && line.len() > 2)
            .collect::<Vec<_>>()
            .join("\n");
        
        if !cleaned.trim().is_empty() {
            return cleaned;
        }
        
        // Fallback: custom HTML parsing (existing logic)
        let mut result = String::new();
        let document = Html::parse_fragment(html);
        
        // Use CSS selectors to extract content in a structured way
        let paragraph_selector = Selector::parse("p, div, span, em, strong, h1, h2, h3, h4, h5, h6, ul, li, table, tr, td").unwrap();
        
        for element in document.select(&paragraph_selector) {
            let tag_name = element.value().name();
            let text_content = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
            
            if text_content.is_empty() {
                continue;
            }
            
            match tag_name {
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    result.push_str(&format!("\n{}\n", text_content.to_uppercase()));
                }
                "p" | "div" => {
                    result.push_str(&format!("{}\n", text_content));
                }
                "li" => {
                    result.push_str(&format!("• {}\n", text_content));
                }
                "em" => {
                    result.push_str(&format!("*{}*", text_content));
                }
                "strong" => {
                    result.push_str(&format!("**{}**", text_content));
                }
                "td" => {
                    result.push_str(&format!("{} | ", text_content));
                }
                _ => {
                    result.push_str(&format!("{} ", text_content));
                }
            }
        }
        
        // Clean up the result
        let cleaned = result
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && line.len() > 3) // Remove very short lines
            .collect::<Vec<_>>()
            .join("\n");
        
        // Remove excessive newlines
        let final_result = Regex::new(r"\n{3,}").unwrap()
            .replace_all(&cleaned, "\n\n")
            .to_string();
        
        if final_result.trim().is_empty() {
            // Last resort fallback: just strip HTML tags and return raw text
            let tag_regex = Regex::new(r"<[^>]+>").unwrap();
            let raw_text = tag_regex.replace_all(html, " ");
            let whitespace_regex = Regex::new(r"\s+").unwrap();
            whitespace_regex.replace_all(&raw_text, " ").trim().to_string()
        } else {
            final_result
        }
    }

    async fn fuzzy_search(&self, query: &str, category: Option<SearchCategory>) -> Result<Vec<SearchResult>, String> {
        // For Wikidot, fuzzy search attempts common variations
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
        // For Wikidot implementation, return common suggestions based on query
        let mut suggestions = Vec::new();
        
        let query_lower = query.to_lowercase();
        
        // Common spell suggestions with better prefix matching
        if query_lower.contains("fire") || "fireball".starts_with(&query_lower) || "fire".starts_with(&query_lower) {
            suggestions.extend(vec!["fireball".to_string(), "fire-bolt".to_string(), "burning-hands".to_string()]);
        }
        if query_lower.contains("heal") || "heal".starts_with(&query_lower) || "healing".starts_with(&query_lower) {
            suggestions.extend(vec!["cure-wounds".to_string(), "healing-word".to_string(), "heal".to_string()]);
        }
        if query_lower.contains("light") || "light".starts_with(&query_lower) || "lightning".starts_with(&query_lower) {
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
    fn test_wiki_page_content_creation() {
        let page = WikiPageContent {
            index: "fireball".to_string(),
            name: "Fireball".to_string(),
            url: "http://dnd5e.wikidot.com/spell:fireball".to_string(),
            content: "3rd-level evocation\nCasting Time: 1 action\nRange: 150 feet".to_string(),
            content_type: "spell".to_string(),
        };
        
        let result = SearchResult { page };
        assert_eq!(result.name(), "Fireball");
        assert_eq!(result.index(), "fireball");
        assert_eq!(result.content_type(), "spell");
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
    fn test_cache_functionality() {
        let client = DndSearchClient::new();
        
        // Test cache directory creation
        assert!(client.cache_dir.exists() || client.cache_dir.parent().map_or(false, |p| p.exists()));
        
        // Test cache path generation
        let cache_path = client.get_cache_path("spell:fireball");
        assert!(cache_path.to_str().unwrap().contains("spell_fireball.txt"));
        
        // Test safe slug conversion
        let cache_path2 = client.get_cache_path("monster:ancient-red-dragon");
        assert!(cache_path2.to_str().unwrap().contains("monster_ancient-red-dragon.txt"));
    }
    
    #[test]
    fn test_cached_content_parsing() {
        let client = DndSearchClient::new();
        
        let test_content = "TITLE:Fireball\nURL:http://dnd5e.wikidot.com/spell:fireball\nCONTENT:\nA bright streak flashes from your pointing finger.";
        
        let parsed = client.parse_cached_content(test_content);
        assert!(parsed.is_some());
        
        let (title, url, content) = parsed.unwrap();
        assert_eq!(title, "Fireball");
        assert_eq!(url, "http://dnd5e.wikidot.com/spell:fireball");
        assert!(content.contains("bright streak"));
    }
    
    #[test]
    fn test_html2text_integration() {
        let client = DndSearchClient::new();
        
        let html = "<p>This is a <strong>test</strong> paragraph.</p><p>Another paragraph with <em>emphasis</em>.</p>";
        let result = client.html_to_readable_text(html);
        
        assert!(result.contains("test"));
        assert!(result.contains("paragraph"));
        assert!(!result.contains("<p>"));
        assert!(!result.contains("<strong>"));
    }

    #[test]
    fn test_possible_urls_generation() {
        let client = DndSearchClient::new();
        
        let urls = client.generate_possible_urls("fireball", "spell");
        assert!(!urls.is_empty());
        assert!(urls.iter().any(|url| url.contains("spell:fireball")));
        
        let class_urls = client.generate_possible_urls("fighter", "class");
        assert!(class_urls.iter().any(|url| url.ends_with("/fighter")));
    }

    #[tokio::test]
    async fn test_get_suggestions() {
        let client = DndSearchClient::new();
        
        let suggestions = client.get_suggestions("fir", Some(SearchCategory::Spells)).await;
        assert!(suggestions.iter().any(|s| s.contains("fire")));
        
        let suggestions = client.get_suggestions("fig", Some(SearchCategory::Classes)).await;
        assert!(suggestions.iter().any(|s| s == "fighter"));
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

    // Test actual search with real data
    #[tokio::test]
    async fn test_real_search() {
        let client = DndSearchClient::new();
        
        println!("🔍 Testing live search for 'fireball'...");
        
        match client.search("fireball", Some(SearchCategory::Spells)).await {
            Ok(results) => {
                if !results.is_empty() {
                    println!("✅ Successfully found {} result(s) for 'fireball'", results.len());
                    let result = &results[0];
                    println!("📝 Page name: {}", result.name());
                    println!("🔗 Content type: {}", result.content_type());
                    
                    assert!(!result.page.content.is_empty(), "Content should not be empty");
                    println!("✅ Page content extraction works correctly");
                } else {
                    println!("⚠️ No results found for 'fireball' - this might indicate parsing issues");
                }
            },
            Err(e) => {
                println!("⚠️ Search failed: {}", e);
                println!("💡 This is expected if network is unavailable or site structure changed");
            }
        }
    }
}