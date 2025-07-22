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
    Search,
    Exit,
}

#[derive(Debug)]
pub struct App {
    pub mode: AppMode,
    pub selected_index: usize,
    pub characters: Vec<Character>,
    pub should_quit: bool,
    pub message: Option<String>,
}

impl App {
    pub fn new(characters: Vec<Character>) -> Self {
        Self {
            mode: AppMode::MainMenu,
            selected_index: 0,
            characters,
            should_quit: false,
            message: None,
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
        match key {
            KeyCode::Up => self.previous_item(),
            KeyCode::Down => self.next_item(),
            KeyCode::Enter => self.select_current(),
            KeyCode::Esc => self.go_back(),
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
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
                    3 => self.mode = AppMode::CombatTracker,
                    4 => self.mode = AppMode::Search,
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
            _ => {}
        }
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
            AppMode::CombatTracker => {
                // Disable TUI temporarily and run CLI - this will include search functionality
                disable_raw_mode()?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
                
                combat_tracker_tui_mode();
                
                println!("Press Enter to return to menu...");
                let mut _buffer = String::new();
                let _ = std::io::stdin().read_line(&mut _buffer);
                
                // Re-enable TUI
                enable_raw_mode()?;
                execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
                
                app.mode = AppMode::ToolsMenu;
                app.selected_index = 0;
            }
            AppMode::Search => {
                // Disable TUI temporarily and run CLI
                disable_raw_mode()?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
                
                search_tui_mode();
                
                println!("Press Enter to return to menu...");
                let mut _buffer = String::new();
                let _ = std::io::stdin().read_line(&mut _buffer);
                
                // Re-enable TUI
                enable_raw_mode()?;
                execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
                
                app.mode = AppMode::ToolsMenu;
                app.selected_index = 0;
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
    render_main_content(f, chunks[1], app);

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
        AppMode::Search => "🔍 D&D 5e Search 🔍",
        AppMode::Exit => "👋 Goodbye! 👋",
    };
    Text::from(title)
}

fn get_help_text(mode: &AppMode) -> Text {
    let help = match mode {
        AppMode::MainMenu | AppMode::CharactersMenu | AppMode::ToolsMenu => 
            "↑↓ Navigate • Enter Select • Esc Back • Q Quit",
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