pub fn roll_dice(input: &str) -> Result<(Vec<u8>, u32), String> {
    // Remove 'r' prefix if present
    let input = input.strip_prefix('r').unwrap_or(input);
    
    let mut split = input.split('d');
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
    let mut total = 0u32;
    
    for _ in 0..num {
        let roll = (rand::random::<u8>() % sides) + 1;
        rolls.push(roll);
        total += roll as u32;
    }
    
    Ok((rolls, total))
}

pub fn roll_dice_mode() {
    println!("Dice Rolling Mode");
    println!("Commands: r<num>d<sides> (e.g., r3d6), q to quit");
    
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
                match roll_dice(input) {
                    Ok((rolls, total)) => {
                        for (i, roll) in rolls.iter().enumerate() {
                            println!("Roll {}: {}", i + 1, roll);
                        }
                        println!("Total: {}", total);
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            Some('q') => ending = true,
            Some('h') | Some('?') => {
                println!("Commands:");
                println!("  r<num>d<sides> - Roll dice (e.g., r3d6 rolls 3 six-sided dice)");
                println!("  q - Quit dice mode");
                println!("  h or ? - Show this help");
            }
            _ => println!("Invalid command. Type 'h' for help."),
        }
    }
}