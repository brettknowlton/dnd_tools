// Import all modules for tests
use crate::character::*;
use crate::dice::*;
use crate::initiative::*;
use crate::events::*;
use crate::combat::*;

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

    // New tests for combat functionality
    #[test]
    fn test_combatant_from_character() {
        let mut character = Character::new("TestFighter");
        character.hp = Some(25);
        character.max_hp = Some(25);
        character.ac = Some(16);
        
        let combatant = Combatant::from_character(character, 15);
        
        assert_eq!(combatant.name, "TestFighter");
        assert_eq!(combatant.current_hp, 25);
        assert_eq!(combatant.max_hp, 25);
        assert_eq!(combatant.ac, 16);
        assert_eq!(combatant.initiative, 15);
        assert_eq!(combatant.is_player, true);
        assert!(combatant.status_effects.is_empty());
    }

    #[test]
    fn test_combatant_new_npc() {
        let combatant = Combatant::new_npc("Orc".to_string(), 15, 13, 10);
        
        assert_eq!(combatant.name, "Orc");
        assert_eq!(combatant.current_hp, 15);
        assert_eq!(combatant.max_hp, 15);
        assert_eq!(combatant.ac, 13);
        assert_eq!(combatant.initiative, 10);
        assert_eq!(combatant.is_player, false);
        assert!(combatant.status_effects.is_empty());
    }

    #[test]
    fn test_status_effects() {
        let mut combatant = Combatant::new_npc("TestNPC".to_string(), 20, 14, 12);
        
        // Add status
        let poison_status = StatusEffect {
            name: "Poisoned".to_string(),
            description: Some("Taking poison damage".to_string()),
            duration: Some(3),
        };
        combatant.add_status(poison_status);
        assert_eq!(combatant.status_effects.len(), 1);
        assert_eq!(combatant.status_effects[0].name, "Poisoned");
        
        // Remove status
        assert!(combatant.remove_status("Poisoned"));
        assert!(combatant.status_effects.is_empty());
        
        // Try to remove non-existent status
        assert!(!combatant.remove_status("Charmed"));
    }

    #[test]
    fn test_combat_tracker_basic() {
        let mut tracker = CombatTracker::new();
        
        let combatant1 = Combatant::new_npc("Fighter".to_string(), 30, 18, 20);
        let combatant2 = Combatant::new_npc("Wizard".to_string(), 15, 12, 15);
        
        tracker.add_combatant(combatant1);
        tracker.add_combatant(combatant2);
        
        // Should be sorted by initiative (highest first)
        assert_eq!(tracker.combatants[0].name, "Fighter");
        assert_eq!(tracker.combatants[1].name, "Wizard");
        
        // Test next turn
        let current = tracker.next_turn();
        assert!(current.is_some());
        assert_eq!(current.unwrap().name, "Fighter");
    }

    #[test]
    fn test_combat_tracker_skip_zero_initiative() {
        let mut tracker = CombatTracker::new();
        
        let combatant1 = Combatant::new_npc("Active".to_string(), 20, 15, 10);
        let mut combatant2 = Combatant::new_npc("Inactive".to_string(), 20, 15, 0);
        combatant2.initiative = 0; // This should be skipped
        
        tracker.add_combatant(combatant1);
        tracker.add_combatant(combatant2);
        
        // Should skip combatant with initiative 0
        let current = tracker.next_turn();
        assert!(current.is_some());
        assert_eq!(current.unwrap().name, "Active");
        
        // Next turn should still be "Active" since "Inactive" is skipped
        let next = tracker.next_turn();
        assert!(next.is_some());
        assert_eq!(next.unwrap().name, "Active");
        
        // Should be round 2 now
        assert_eq!(tracker.round_number, 2);
    }

    #[test]
    fn test_combat_tracker_remove_combatant() {
        let mut tracker = CombatTracker::new();
        
        let combatant1 = Combatant::new_npc("Fighter".to_string(), 30, 18, 20);
        let combatant2 = Combatant::new_npc("Wizard".to_string(), 15, 12, 15);
        
        tracker.add_combatant(combatant1);
        tracker.add_combatant(combatant2);
        
        // Remove existing combatant
        assert!(tracker.remove_combatant("Fighter"));
        assert_eq!(tracker.combatants.len(), 1);
        assert_eq!(tracker.combatants[0].name, "Wizard");
        
        // Try to remove non-existent combatant
        assert!(!tracker.remove_combatant("NonExistent"));
    }

    #[test]
    fn test_combatant_get_methods() {
        let mut tracker = CombatTracker::new();
        let combatant = Combatant::new_npc("TestNPC".to_string(), 20, 14, 12);
        tracker.add_combatant(combatant);
        
        // Test get_combatant (immutable)
        let found = tracker.get_combatant("TestNPC");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "TestNPC");
        
        // Test case insensitive search
        let found_case = tracker.get_combatant("testnpc");
        assert!(found_case.is_some());
        
        // Test get_combatant_mut (mutable)
        let found_mut = tracker.get_combatant_mut("TestNPC");
        assert!(found_mut.is_some());
        
        // Test not found
        let not_found = tracker.get_combatant("NotExist");
        assert!(not_found.is_none());
    }
}