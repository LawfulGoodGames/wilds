use super::{random_training_sequence, training_tier};
use crate::app::TrainingTier;

#[test]
fn training_tier_breakpoints_are_stable() {
    assert!(matches!(training_tier(0, 15), TrainingTier::Poor));
    assert!(matches!(training_tier(6, 15), TrainingTier::Solid));
    assert!(matches!(training_tier(11, 15), TrainingTier::Great));
}

#[test]
fn generated_training_sequence_matches_requested_length() {
    let sequence = random_training_sequence(6);
    assert_eq!(sequence.len(), 6);
    assert!(sequence.iter().all(|ch| ch.is_ascii_lowercase()));
}
