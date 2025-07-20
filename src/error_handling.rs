use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum AppError {
    IoError(std::io::Error),
    ParseError(String),
    ValidationError(String),
    FileError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::IoError(e) => write!(f, "IO Error: {}", e),
            AppError::ParseError(e) => write!(f, "Parse Error: {}", e),
            AppError::ValidationError(e) => write!(f, "Validation Error: {}", e),
            AppError::FileError(e) => write!(f, "File Error: {}", e),
        }
    }
}

impl Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::IoError(error)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

pub fn validate_character_name(name: &str) -> Result<()> {
    if name.trim().is_empty() {
        return Err(AppError::ValidationError("Character name cannot be empty".to_string()));
    }
    if name.len() > 50 {
        return Err(AppError::ValidationError("Character name too long (max 50 characters)".to_string()));
    }
    if name.chars().any(|c| c.is_control() || c == '/' || c == '\\') {
        return Err(AppError::ValidationError("Character name contains invalid characters".to_string()));
    }
    Ok(())
}

pub fn validate_numeric_input(input: &str, field_name: &str, min: Option<u8>, max: Option<u8>) -> Result<u8> {
    match input.parse::<u8>() {
        Ok(value) => {
            if let Some(min_val) = min {
                if value < min_val {
                    return Err(AppError::ValidationError(format!("{} must be at least {}", field_name, min_val)));
                }
            }
            if let Some(max_val) = max {
                if value > max_val {
                    return Err(AppError::ValidationError(format!("{} must be at most {}", field_name, max_val)));
                }
            }
            Ok(value)
        }
        Err(_) => Err(AppError::ParseError(format!("Invalid number for {}", field_name)))
    }
}