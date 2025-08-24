pub fn roll_dice(input: &str) -> Result<(Vec<u8>, u32), String> {
    // Remove 'r' prefix if present
    let input = input.strip_prefix('r').unwrap_or(input);
    
    // Handle modifier (e.g., "2d6+3")
    let (dice_part, modifier) = if let Some(plus_pos) = input.find('+') {
        let (dice, mod_str) = input.split_at(plus_pos);
        let modifier = mod_str[1..].parse::<i32>().unwrap_or(0);
        (dice, modifier)
    } else if let Some(minus_pos) = input.find('-') {
        let (dice, mod_str) = input.split_at(minus_pos);
        let modifier = -mod_str[1..].parse::<i32>().unwrap_or(0);
        (dice, modifier)
    } else {
        (input, 0)
    };
    
    let mut split = dice_part.split('d');
    let num_str = split.next().ok_or("Invalid dice format")?;
    let sides_str = split.next().ok_or("Invalid dice format")?;
    
    let num = num_str.parse::<u8>().map_err(|_| "Invalid number of dice")?;
    let sides = sides_str.parse::<u8>().map_err(|_| "Invalid number of sides")?;
    
    if num == 0 || sides == 0 {
        return Err("Number of dice and sides must be greater than 0".to_string());
    }
    
    if num > 100 {
        return Err("Too many dice (maximum 100)".to_string());
    }
    
    let mut rolls = Vec::new();
    let mut dice_total = 0u32;
    
    for _ in 0..num {
        let roll = (rand::random::<u8>() % sides) + 1;
        rolls.push(roll);
        dice_total += roll as u32;
    }
    
    // Apply modifier as post-roll addition/subtraction
    let final_total = (dice_total as i32 + modifier).max(0) as u32;
    
    Ok((rolls, final_total))
}

pub fn roll_dice_with_crits(input: &str) -> Result<(Vec<u8>, u32, Option<String>), String> {
    let (rolls, total) = roll_dice(input)?;
    
    // Check for critical results on d20 rolls
    let crit_message = if input.contains("d20") && rolls.len() == 1 {
        match rolls[0] {
            1 => Some("🎲💀 CRITICAL FAILURE! 💀🎲".to_string()),
            20 => Some("🎲⭐ CRITICAL SUCCESS! ⭐🎲".to_string()),
            _ => None,
        }
    } else {
        None
    };
    
    Ok((rolls, total, crit_message))
}

/// Get ASCII art for a dice roll based on the number of sides
pub fn get_dice_ascii_art(sides: u8, value: u8) -> Vec<String> {
    match sides {
        4 => get_d4_ascii(value),
        6 => get_d6_ascii(value),
        8 => get_d8_ascii(value),
        10 => get_d10_ascii(value),
        12 => get_d12_ascii(value),
        20 => get_d20_ascii(value),
        _ => vec![format!("d{}: {}", sides, value)],
    }
}

/// Get color for a dice value based on the range (red=low, green=high)
pub fn get_dice_color_code(value: u8, max_value: u8) -> &'static str {
    match value {
        1 => "\x1b[30m", // Black for 1
        v if v == max_value && max_value == 20 => "\x1b[33m", // Gold for nat 20
        v => {
            let ratio = (v as f32) / (max_value as f32);
            if ratio <= 0.33 {
                "\x1b[31m" // Red for low rolls
            } else if ratio <= 0.66 {
                "\x1b[33m" // Yellow for medium rolls
            } else {
                "\x1b[32m" // Green for high rolls
            }
        }
    }
}

/// Reset color to default
pub fn reset_color() -> &'static str {
    "\x1b[0m"
}

// ASCII art for d4 (Triangle)
fn get_d4_ascii(value: u8) -> Vec<String> {
    vec![
        "    /\\    ".to_string(),
        "   /  \\   ".to_string(),
        format!("  / {} \\  ", if value < 10 { format!(" {}", value) } else { value.to_string() }),
        " /____\\  ".to_string(),
    ]
}

// ASCII art for d6 (Square)
fn get_d6_ascii(value: u8) -> Vec<String> {
    vec![
        "┌─────┐".to_string(),
        "│     │".to_string(),
        format!("│  {}  │", if value < 10 { format!(" {}", value) } else { value.to_string() }),
        "│     │".to_string(),
        "└─────┘".to_string(),
    ]
}

// ASCII art for d8 (Hexagon)
fn get_d8_ascii(value: u8) -> Vec<String> {
    vec![
        "  /---\\  ".to_string(),
        " /     \\ ".to_string(),
        format!("|   {}   |", if value < 10 { format!(" {}", value) } else { value.to_string() }),
        " \\     / ".to_string(),
        "  \\---/  ".to_string(),
    ]
}

// ASCII art for d10 (Pentagon)
fn get_d10_ascii(value: u8) -> Vec<String> {
    vec![
        "   /^\\   ".to_string(),
        "  /   \\  ".to_string(),
        format!(" | {} | ", if value < 10 { format!(" {}", value) } else { value.to_string() }),
        "  \\___/  ".to_string(),
    ]
}

// ASCII art for d12 (Decagon)
fn get_d12_ascii(value: u8) -> Vec<String> {
    vec![
        "  /‾‾‾\\  ".to_string(),
        " /     \\ ".to_string(),
        format!("|  {}  |", if value < 10 { format!(" {}", value) } else { value.to_string() }),
        " \\     / ".to_string(),
        "  \\___/  ".to_string(),
    ]
}

// ASCII art for d20 (Hexagonal design inside hexagon)
fn get_d20_ascii(value: u8) -> Vec<String> {
    vec![
        "   /‾‾‾\\   ".to_string(),
        "  /  ◊  \\  ".to_string(),
        format!(" | {} | ", if value < 10 { format!(" {}", value) } else { value.to_string() }),
        "  \\  ◊  /  ".to_string(),
        "   \\___/   ".to_string(),
    ]
}

pub fn roll_dice_mode() {
    println!("🎲 Enhanced Dice Rolling Mode 🎲");
    println!("Commands: r<num>d<sides>[+/-modifier] (e.g., r3d6+2, r1d20-1), q to quit");
    
    let mut ending = false;
    while !ending {
        println!("\nDice > Enter command:");
        let mut buffer = String::new();
        if std::io::stdin().read_line(&mut buffer).is_err() {
            println!("Failed to read input");
            continue;
        }
        
        let input = buffer.trim();
        match input.chars().next() {
            Some('r') => {
                match roll_dice_with_crits(input) {
                    Ok((rolls, total, crit_message)) => {
                        println!("\n🎲 DICE ROLL RESULTS 🎲");
                        println!("{}", "═".repeat(40));
                        
                        // Extract dice type from input for ASCII art
                        let dice_type = if let Some(d_pos) = input.find('d') {
                            let after_d = &input[d_pos + 1..];
                            let sides_str = after_d.chars()
                                .take_while(|c| c.is_ascii_digit())
                                .collect::<String>();
                            sides_str.parse::<u8>().unwrap_or(6)
                        } else {
                            6
                        };
                        
                        // Display each dice with ASCII art and colors
                        for (i, &roll) in rolls.iter().enumerate() {
                            let color = get_dice_color_code(roll, dice_type);
                            let reset = reset_color();
                            let ascii_art = get_dice_ascii_art(dice_type, roll);
                            
                            println!("\nDie #{} (d{}):", i + 1, dice_type);
                            for line in ascii_art {
                                println!("{}{}{}", color, line, reset);
                            }
                        }
                        
                        println!("\n📊 Summary:");
                        println!("   Individual Rolls: {:?}", rolls);
                        println!("   TOTAL: {}", total);
                        
                        // Display critical message if applicable
                        if let Some(message) = crit_message {
                            println!("\n🌟 {}", message);
                        }
                        
                        println!("{}", "═".repeat(40));
                    }
                    Err(e) => println!("❌ Error: {}", e),
                }
            }
            Some('q') => ending = true,
            Some('h') | Some('?') => {
                println!("\n🎯 DICE ROLLING COMMANDS:");
                println!("  r<num>d<sides>         - Roll dice (e.g., r3d6 rolls 3 six-sided dice)");
                println!("  r<num>d<sides>+<mod>   - Roll with positive modifier (e.g., r1d20+5)");
                println!("  r<num>d<sides>-<mod>   - Roll with negative modifier (e.g., r2d6-2)");
                println!("  q                      - Quit dice mode");
                println!("  h or ?                 - Show this help");
                println!("\n🎨 COLOR CODING:");
                println!("  🔴 Red: Low rolls (bottom 33%)");
                println!("  🟡 Yellow: Medium rolls (middle 33%)");
                println!("  🟢 Green: High rolls (top 33%)");
                println!("  ⚫ Black: Natural 1");
                println!("  🟨 Gold: Natural 20");
            }
            _ => println!("❌ Invalid command. Type 'h' for help."),
        }
    }
}