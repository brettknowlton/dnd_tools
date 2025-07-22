# D&D Tools TUI Implementation

## Features Implemented

### 1. Ratatui TUI Interface ✅
- **Dark blue theme** as requested:
  - Background: RGB(16, 24, 48) - Dark blue
  - Menu areas: RGB(32, 48, 96) - Medium blue  
  - Selected items: RGB(64, 96, 192) - Lighter blue
  - Text: White
  - Borders: RGB(128, 144, 192) - Light blue-gray

### 2. Navigation ✅
- **Crossterm implementation** for cross-platform input handling
- Arrow keys (↑↓) for menu navigation
- Enter to select menu items
- Esc to go back to previous menu
- Q to quit the application

### 3. Menu Structure ✅
- **Main Menu**: Characters, Tools, Exit
- **Characters Menu**: Creation, Display single, Display all, Deletion, Back
- **Tools Menu**: Initiative tracker, NPC randomizer, Dice, Combat tracker, Search D&D 5e API, Back

### 4. Button-like Interface for Generators ✅
- All generator tools (NPC randomizer, dice, etc.) use button-style menu navigation
- Selected items highlighted with ► symbol and different background color
- Clean, organized layout with borders and proper spacing

### 5. CLI-like Interface Preserved ✅
- **Search functionality** maintains CLI-like experience as requested
- **Combat tracker** keeps CLI-like interface as requested
- These modes temporarily disable TUI and run in full CLI mode
- Automatic return to TUI after completing CLI operations

### 6. Search in Combat Tracker ✅
- Added `search <query>` command to combat tracker
- **Integrated with D&D 5e Wikidot API** for live data
- User can search for spells, items, monsters, etc. during combat
- **Returns to combat after search** - no disruption to combat flow
- Example usage: `search fireball`, `search longsword`, `search troll`

### 7. Crossterm Input Handling ✅
- Uses crossterm for terminal control and input
- Works across different platforms (Linux, macOS, Windows)
- Proper terminal state management (raw mode, alternate screen)

## Usage Examples

### Starting the Application
```bash
cargo run --release
```

### TUI Navigation
- Use ↑↓ arrow keys to navigate menus
- Press Enter to select items
- Press Esc to go back
- Press Q to quit

### Search in Combat
1. Navigate to Tools > Combat Tracker
2. Set up combat encounters
3. During any combat turn, use: `search <query>`
4. View search results
5. Press Enter to return to combat

### Commands Available in Combat
- `search fireball` - Search for the fireball spell
- `search longsword` - Look up weapon stats  
- `search troll` - Find monster information
- `attack <target>` - Roll attack vs target's AC
- `stats <name>` - Show character stats
- `next` - Advance to next combatant's turn
- `help` - Show all available commands

## Technical Implementation

### Dependencies Added
- `ratatui = "0.28"` - TUI framework
- `crossterm = "0.28"` - Cross-platform terminal control

### Key Files
- `src/tui.rs` - Main TUI implementation with dark blue theme
- `src/main.rs` - Updated to use TUI, added search to combat
- `Cargo.toml` - Added new dependencies

### Theme Colors Used
```rust
BACKGROUND_COLOR: RGB(16, 24, 48)    // Dark blue background
MENU_COLOR: RGB(32, 48, 96)          // Medium blue for menu areas  
SELECTED_COLOR: RGB(64, 96, 192)     // Lighter blue for selections
TEXT_COLOR: White                     // White text
BORDER_COLOR: RGB(128, 144, 192)     // Light blue-gray borders
```

The implementation successfully meets all requirements:
✅ Ratatui interface with dark blue theme
✅ Button navigation for generators  
✅ CLI-like interface for search and combat
✅ Crossterm for cross-platform compatibility
✅ Search functionality in combat tracker with return capability