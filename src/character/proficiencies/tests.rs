use super::{MajorSkill, MinorSkill, Stats, proficiency_xp_for_level, study_plan};

#[test]
fn studying_gets_harder_at_higher_ranks() {
    let stats = Stats {
        strength: 12,
        dexterity: 12,
        constitution: 12,
        intelligence: 12,
        wisdom: 12,
        charisma: 12,
    };
    let novice = study_plan(MinorSkill::Runecraft, 0, &stats);
    let veteran = study_plan(
        MinorSkill::Runecraft,
        proficiency_xp_for_level(25) as i32,
        &stats,
    );
    assert!(novice.success_chance > veteran.success_chance);
    assert!(novice.hours < veteran.hours);
}

#[test]
fn study_plan_uses_skill_governing_stat() {
    let plan = study_plan(MinorSkill::Larceny, 0, &Stats::default());
    assert_eq!(plan.governing_stat, MajorSkill::Dexterity);
}

#[test]
fn proficiency_roster_is_twelve_total() {
    assert_eq!(MajorSkill::ALL.len() + MinorSkill::ALL.len(), 12);
}
