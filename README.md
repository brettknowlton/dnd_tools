# D&D Tools - Wikidot Scraping for D&D 5e Reference Data

A comprehensive D&D 5e tools application with both TUI (Terminal User Interface) and CLI support for quick reference lookups from dnd5e.wikidot.com.

## Features

### 🔍 Wikidot Scraping for D&D 5e Reference Data
- **Fast, offline-friendly lookup** of races, classes, spells, items, and more
- **Live data fetching** from https://dnd5e.wikidot.com via HTTP/HTML scraping
- **Intelligent caching** system for instant offline reuse
- **CLI-friendly formatting** with clean plain text output
- **Legal compliance** with CC BY-SA 3.0 attribution

### 🎮 Interactive TUI Mode
- **Dark blue theme** with intuitive navigation
- **Character management** (creation, display, deletion)
- **Combat tracker** with initiative, status effects, and damage tracking
- **NPC generator** with random stats and custom options
- **Dice rolling** system with critical hit detection
- **Search integration** directly in combat mode

### ⚡ Command-Line Interface
Direct access to D&D 5e reference data from the terminal:

```bash
# Search for any content
dnd_tools fireball
dnd_tools "magic missile"

# Search specific categories
dnd_tools spell fireball
dnd_tools class wizard
dnd_tools equipment longsword
dnd_tools monster troll
dnd_tools race elf

# Force refresh cache
dnd_tools --refresh spell fireball

# Get help
dnd_tools --help
```

### 🗂️ Intelligent Caching System
- **Automatic caching** to `$HOME/.cache/dnd_tools/` (or `.cache/dnd_tools/` fallback)
- **Instant offline access** to previously searched content
- **Cache refresh** option with `--refresh` flag
- **Safe filename handling** for all query types

### 📄 Legal Attribution & Compliance
- **Source attribution** displayed with every search result
- **CC BY-SA 3.0 compliance** with proper license notices
- **Personal/educational use** guidelines clearly stated
- **Community content** respect and proper crediting

## Installation & Usage

### Prerequisites
- Rust 1.70+ (uses 2024 edition)
- Internet connection (for initial data fetching)

### Building from Source
```bash
git clone https://github.com/brettknowlton/dnd_tools
cd dnd_tools
cargo build --release
```

### Running the Application

#### Interactive TUI Mode (Default)
```bash
cargo run
# or
./target/release/dnd_tools
```

#### Command-Line Mode
```bash
# Quick spell lookup
cargo run -- spell fireball

# Equipment search
cargo run -- equipment "chain mail"

# Monster information
cargo run -- monster "ancient red dragon"

# Force cache refresh
cargo run -- --refresh spell "magic missile"
```

## How It Works

### 1. Data Fetching & Extraction
- Builds page URLs from user queries (e.g., 'spell:fireball' → https://dnd5e.wikidot.com/spell:fireball)
- Fetches pages using `reqwest` with timeout handling
- Extracts main content using `scraper` CSS selectors (`#page-content`)
- Converts HTML to readable text using `html2text` for better formatting

### 2. Caching System
- Uses `dirs` crate for cross-platform cache directory location
- Saves pages as safe filenames (e.g., `spell_fireball.txt`)
- Checks cache first on every query for instant responses
- Automatically creates cache directory structure

### 3. Display & Attribution
- Formats content with proper headings and structure
- Displays required legal attribution:
  ```
  📄 Source: dnd5e.wikidot.com | CC BY-SA 3.0
  🔗 https://creativecommons.org/licenses/by-sa/3.0/
  ℹ️  Content used under Creative Commons Attribution-ShareAlike 3.0 license
     for personal/educational use only.
  ```

### 4. Error Handling & Fallbacks
- Robust error handling with `anyhow` for descriptive messages
- Multiple URL pattern attempts for different content types
- Graceful fallback to suggestions when exact matches aren't found
- Network timeout handling with user-friendly messages

## Supported Content Categories

| Category | Examples | URL Pattern |
|----------|----------|-------------|
| **Spells** | fireball, magic missile, cure wounds | `/spell:name` |
| **Classes** | wizard, fighter, paladin | `/name` |
| **Equipment** | longsword, chain mail, leather armor | `/equipment:name`, `/weapon:name`, `/armor:name` |
| **Monsters** | troll, dragon, goblin | `/monster:name` |
| **Races** | elf, dwarf, halfling | `/name` |

## Examples

### Basic Searches
```bash
# Spell information
dnd_tools fireball
dnd_tools spell "magic missile"

# Class details
dnd_tools class wizard
dnd_tools fighter

# Equipment stats
dnd_tools equipment longsword
dnd_tools weapon "great sword"
dnd_tools armor "plate armor"

# Monster information
dnd_tools monster troll
dnd_tools "ancient red dragon"

# Race details
dnd_tools race elf
dnd_tools halfling
```

### Cache Management
```bash
# Refresh cached spell data
dnd_tools --refresh spell fireball

# Force re-fetch of class information
dnd_tools --refresh class wizard
```

### Interactive Features
1. **Start TUI mode**: `dnd_tools` (no arguments)
2. **Navigate with arrow keys**: ↑↓ to move, Enter to select
3. **Search in combat**: Use `search <query>` command during combat
4. **Character management**: Create, view, edit D&D characters
5. **NPC generation**: Random or custom NPCs with full stats

## Dependencies

The application uses these key dependencies for wikidot scraping:

```toml
reqwest = { version = "0.12", features = ["json", "blocking"] }  # HTTP client
scraper = "0.20"           # HTML parsing and CSS selectors
anyhow = "1.0"             # Error handling
dirs = "5.0"               # Cross-platform directories
html2text = "0.6"          # HTML to text conversion
serde = { version = "1.0", features = ["derive"] }  # Serialization
serde_json = "1.0"         # JSON handling
```

Plus TUI dependencies for the interactive interface:
```toml
ratatui = "0.28"          # Terminal UI framework
crossterm = "0.28"        # Cross-platform terminal control
```

## Legal Notice

**Important**: This tool is for personal and educational use only.

- **All D&D content** is sourced from dnd5e.wikidot.com
- **Licensed under** Creative Commons Attribution-ShareAlike 3.0 Unported License
- **License URL**: https://creativecommons.org/licenses/by-sa/3.0/
- **Attribution**: Content is properly attributed to the source community
- **No commercial use** - this tool is provided free for personal reference

The D&D 5e wikidot community has generously made this content available under CC BY-SA 3.0. Please respect their work and the license terms.

## Technical Implementation

### Architecture
- **Modular design** with separate concerns (TUI, CLI, search, character management)
- **Async/await** support for non-blocking network operations
- **Cross-platform** compatibility (Linux, macOS, Windows)
- **Error resilience** with graceful degradation

### Performance
- **Fast local cache** for instant repeat searches
- **Efficient HTML parsing** with targeted CSS selectors
- **Minimal network requests** through intelligent caching
- **Quick startup** with lazy initialization

### Security
- **No credentials stored** - all data is public community content
- **Safe file operations** with proper path sanitization
- **Timeout handling** prevents hanging requests
- **Input validation** for all user queries

## Contributing

Contributions are welcome! Please ensure:

1. **Respect the CC BY-SA 3.0 license** of source content
2. **Maintain attribution** requirements in any modifications
3. **Test thoroughly** - all existing tests must pass
4. **Follow Rust conventions** and maintain code quality
5. **Document changes** clearly in commit messages

## License

This application code is released under MIT License. However, all D&D content accessed through this tool remains under the Creative Commons Attribution-ShareAlike 3.0 license as provided by the dnd5e.wikidot.com community.