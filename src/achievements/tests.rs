use super::achievement_defs;

#[test]
fn contains_exactly_one_hundred_achievements() {
    assert_eq!(achievement_defs().len(), 100);
}
