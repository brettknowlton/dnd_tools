// Import all modules for tests
use crate::character::*;
use crate::dice::*;
use crate::initiative::*;
use crate::events::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_creation() {
        let character = Character::new("Test Character");
        assert_eq!(character.name, "Test Character");
        assert_eq!(character.level, None);
        assert_eq!(character.hp, None);
    }

    #[test]
    fn test_character_as_vec() {
        let character = Character::new("Test");
        let vec = character.as_vec();
        assert_eq!(vec[0], "Test");
        assert_eq!(vec[1], "0"); // level default
    }

    #[test]
    fn test_roll_dice_valid() {
        let result = roll_dice("r2d6");
        assert!(result.is_ok());
        let (rolls, total) = result.unwrap();
        assert_eq!(rolls.len(), 2);
        assert!(total >= 2 && total <= 12);
        for roll in rolls {
            assert!(roll >= 1 && roll <= 6);
        }
    }

    #[test]
    fn test_roll_dice_invalid() {
        assert!(roll_dice("invalid").is_err());
        assert!(roll_dice("r0d6").is_err());
        assert!(roll_dice("r2d0").is_err());
        assert!(roll_dice("r101d6").is_err());
    }

    #[test]
    fn test_initiative_tracker() {
        let mut tracker = InitiativeTracker::new();
        tracker.add_entry("Player1".to_string(), 15, true);
        tracker.add_entry("NPC1".to_string(), 10, false);
        
        // Should be sorted by initiative (highest first)
        let entries = tracker.get_entries();
        assert_eq!(entries[0].name, "Player1");
        assert_eq!(entries[1].name, "NPC1");
        
        // Test next turn
        let current = tracker.next_turn().unwrap();
        assert_eq!(current.name, "Player1");
        
        // Test remove
        assert!(tracker.remove_entry("Player1"));
        assert!(!tracker.remove_entry("NonExistent"));
    }

    #[test]
    fn test_data_creation() {
        let data = Data::new();
        assert!(data.data.is_empty());
    }
}