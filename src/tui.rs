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

#[derive(Debug, Clone)]
pub enum AppMode {
    MainMenu,
    CharactersMenu,
    ToolsMenu,
    CharacterCreation,
    CharacterDisplay,
    CharacterDeletion,
    InitiativeTracker,
    NpcGenerator,
    Dice,
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
            AppMode::CombatTrackerTUI | AppMode::SearchTUI => {
                self.handle_terminal_key(key);
            }
            _ => {
                match key {
                    KeyCode::Up => self.previous_item(),
                    KeyCode::Down => self.next_item(),
                    KeyCode::Enter => self.select_current(),
                    KeyCode::Esc => self.go_back(),
                    KeyCode::Char('q') => self.should_quit = true,
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
                    0 => self.mode = AppMode::CharacterCreation,
                    1 => self.mode = AppMode::CharacterDisplay,
                    2 => self.mode = AppMode::CharacterDisplay,
                    3 => self.mode = AppMode::CharacterDeletion,
                    4 => {
                        self.mode = AppMode::MainMenu;
                        self.selected_index = 0;
                    }
                    _ => {}
                }
            }
            AppMode::ToolsMenu => {
                match self.selected_index {
                    0 => self.mode = AppMode::InitiativeTracker,
                    1 => self.mode = AppMode::NpcGenerator,
                    2 => self.mode = AppMode::Dice,
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
            AppMode::CharacterCreation | AppMode::CharacterDisplay | AppMode::CharacterDeletion => {
                self.mode = AppMode::CharactersMenu;
                self.selected_index = 0;
            }
            AppMode::InitiativeTracker | AppMode::NpcGenerator | AppMode::Dice | AppMode::CombatTracker | AppMode::Search => {
                self.mode = AppMode::ToolsMenu;
                self.selected_index = 0;
            }
            AppMode::CombatTrackerTUI | AppMode::SearchTUI => {
                self.mode = AppMode::ToolsMenu;
                self.selected_index = 0;
                // Clear terminal state
                self.input_buffer.clear();
                self.output_history.clear();
                self.scroll_offset = 0;
                self.combat_tracker = None;
            }
            _ => {}
        }
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
            KeyCode::Char('q') if self.input_buffer.is_empty() => {
                self.should_quit = true;
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
            _ => {}
        }
    }

    fn process_combat_command(&mut self, command: String) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        let cmd = parts[0].to_lowercase();
        
        match cmd.as_str() {
            "help" | "h" => {
                self.add_output("Combat Mode Commands:".to_string());
                self.add_output("  stats [name] - Show character stats".to_string());
                self.add_output("  attack <target> - Roll d20 attack vs target's AC".to_string());
                self.add_output("  status [add|remove|list] [self|name] <status> - Manage status effects".to_string());
                self.add_output("  search <query> - Search D&D 5e API".to_string());
                self.add_output("  save [ability] [self|name] - Make saving throw".to_string());
                self.add_output("  next|continue - Advance to next combatant".to_string());
                self.add_output("  back - Go back to previous combatant's turn".to_string());
                self.add_output("  insert <name> - Add new combatant mid-fight".to_string());
                self.add_output("  remove <name> - Remove combatant from combat".to_string());
                self.add_output("  show|list - Display current initiative order".to_string());
                self.add_output("  quit|exit - Exit combat mode".to_string());
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
        
        // For now, create an empty combat tracker
        // In a full implementation, we'd need to set up combatants
        self.combat_tracker = Some(crate::combat::CombatTracker {
            combatants: Vec::new(),
            current_turn: 0,
            round_number: 1,
        });
        
        self.add_output("Combat initialized! Add combatants or type 'help' for commands.".to_string());
    }

    fn handle_combat_search(&mut self, query: &str) {
        self.add_output(format!("🔍 Searching for '{}'...", query));
        // For now, just show a placeholder
        // In a full implementation, this would use the search functionality
        self.add_output("Search functionality will be implemented here.".to_string());
        self.add_output("This would integrate with the D&D 5e API.".to_string());
    }

    fn handle_search_query(&mut self, query: &str) {
        self.add_output(format!("🔍 Searching for '{}'...", query));
        // For now, just show a placeholder
        // In a full implementation, this would use the search functionality
        self.add_output("Search functionality will be implemented here.".to_string());
        self.add_output("This would integrate with the D&D 5e API and show results.".to_string());
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
                    KeyCode::Char('q') => break,
                    _ => app.handle_key(key.code),
                }
            }
        }

        // Check for mode changes that require CLI fallback
        match app.mode {
            AppMode::Exit => break,
            AppMode::CharacterCreation => {
                // Disable TUI temporarily and run CLI
                disable_raw_mode()?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
                
                println!("Creating character...");
                let new_character = crate::input_handler::create_character();
                app.characters.push(new_character);
                crate::file_manager::save_characters(app.characters.clone());
                
                println!("Press Enter to return to menu...");
                let mut _buffer = String::new();
                let _ = std::io::stdin().read_line(&mut _buffer);
                
                // Re-enable TUI
                enable_raw_mode()?;
                execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
                
                app.mode = AppMode::CharactersMenu;
                app.selected_index = 0;
                app.message = Some("Character created successfully!".to_string());
            }
            AppMode::CharacterDisplay => {
                // Disable TUI temporarily and run CLI
                disable_raw_mode()?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
                
                if app.selected_index == 1 { // Display single
                    crate::file_manager::display_single_character(&app.characters);
                } else { // Display all
                    crate::file_manager::display_all_characters(&app.characters);
                }
                
                println!("Press Enter to return to menu...");
                let mut _buffer = String::new();
                let _ = std::io::stdin().read_line(&mut _buffer);
                
                // Re-enable TUI
                enable_raw_mode()?;
                execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
                
                app.mode = AppMode::CharactersMenu;
                app.selected_index = 0;
            }
            AppMode::CharacterDeletion => {
                // Disable TUI temporarily and run CLI
                disable_raw_mode()?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
                
                crate::file_manager::delete_character_menu(&mut app.characters);
                
                println!("Press Enter to return to menu...");
                let mut _buffer = String::new();
                let _ = std::io::stdin().read_line(&mut _buffer);
                
                // Re-enable TUI
                enable_raw_mode()?;
                execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
                
                app.mode = AppMode::CharactersMenu;
                app.selected_index = 0;
            }
            AppMode::InitiativeTracker => {
                // Disable TUI temporarily and run CLI
                disable_raw_mode()?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
                
                crate::initiative::initiative_tracker_mode();
                
                println!("Press Enter to return to menu...");
                let mut _buffer = String::new();
                let _ = std::io::stdin().read_line(&mut _buffer);
                
                // Re-enable TUI
                enable_raw_mode()?;
                execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
                
                app.mode = AppMode::ToolsMenu;
                app.selected_index = 0;
            }
            AppMode::NpcGenerator => {
                // Disable TUI temporarily and run CLI
                disable_raw_mode()?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
                
                // Instead of calling these functions, we should define them in this module
                // For now, let's create simplified versions that work with the TUI
                npc_randomizer_tui_mode();
                
                println!("Press Enter to return to menu...");
                let mut _buffer = String::new();
                let _ = std::io::stdin().read_line(&mut _buffer);
                
                // Re-enable TUI
                enable_raw_mode()?;
                execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
                
                app.mode = AppMode::ToolsMenu;
                app.selected_index = 0;
            }
            AppMode::Dice => {
                // Disable TUI temporarily and run CLI
                disable_raw_mode()?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
                
                crate::dice::roll_dice_mode();
                
                println!("Press Enter to return to menu...");
                let mut _buffer = String::new();
                let _ = std::io::stdin().read_line(&mut _buffer);
                
                // Re-enable TUI
                enable_raw_mode()?;
                execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
                
                app.mode = AppMode::ToolsMenu;
                app.selected_index = 0;
            }
            AppMode::CombatTrackerTUI => {
                // Initialize combat tracker if not already done
                if app.combat_tracker.is_none() {
                    app.add_output("⚔️ Combat Tracker - Interactive Mode ⚔️".to_string());
                    app.add_output("Type 'init' to initialize combat or 'help' for commands".to_string());
                }
            }
            AppMode::SearchTUI => {
                // Initialize search mode
                if app.output_history.is_empty() {
                    app.add_output("🔍 D&D 5e Search - Interactive Mode 🔍".to_string());
                    app.add_output("Type 'search <query>' to search or 'help' for commands".to_string());
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
        AppMode::CombatTrackerTUI | AppMode::SearchTUI => {
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
                    "Type 'help' for available commands".to_string(),
                    "Type 'init' to initialize combat".to_string(),
                    "".to_string(),
                ]
            },
            AppMode::SearchTUI => {
                vec![
                    "🔍 D&D 5e Search - Interactive Mode 🔍".to_string(),
                    "".to_string(),
                    "Type 'help' for available commands".to_string(),
                    "Type 'search <query>' to search".to_string(),
                    "Example: search fireball".to_string(),
                    "".to_string(),
                ]
            },
            _ => vec!["Ready.".to_string()],
        }
    } else {
        // Show recent output with scrolling
        let start_index = app.scroll_offset;
        let end_index = std::cmp::min(
            app.output_history.len(),
            start_index + (area.height as usize).saturating_sub(2)
        );
        
        if start_index < app.output_history.len() {
            app.output_history[start_index..end_index].to_vec()
        } else {
            app.output_history.clone()
        }
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
        AppMode::CharacterDisplay => "📋 Character Display 📋",
        AppMode::CharacterDeletion => "🗑️  Character Deletion 🗑️",
        AppMode::InitiativeTracker => "⚡ Initiative Tracker ⚡",
        AppMode::NpcGenerator => "🎭 NPC Generator 🎭",
        AppMode::Dice => "🎲 Dice Roller 🎲",
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
            "↑↓ Navigate • Enter Select • Esc Back • Q Quit",
        AppMode::CombatTrackerTUI => 
            "Type commands • Enter Execute • ↑↓ History • PgUp/PgDn Scroll • Esc Back",
        AppMode::SearchTUI => 
            "Type commands • Enter Execute • ↑↓ History • PgUp/PgDn Scroll • Esc Back",
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