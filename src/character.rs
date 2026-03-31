mod catalog;
mod proficiencies;
mod progression;

pub use catalog::{
    CharacterClassProgression, CharacterCreation, Class, CreationStep, GearPackage, Race,
};
pub use proficiencies::{
    MAX_COMBAT_PROFICIENCY_RANK, MAX_LEVEL, MAX_PROFICIENCY_LEVEL, MajorProficiencyData,
    MajorSkill, MinorSkill, ProficiencyData, ResistanceProfile, STAT_POINTS, Stats, StudyPlan,
    TrainingSessionPlan, level_from_xp, level_progress_pct, major_study_plan,
    major_study_plan_for_xp, proficiency_level_from_xp, proficiency_progress_pct,
    proficiency_xp_for_level, study_plan, training_session_plan_for_major,
    training_session_plan_for_minor, xp_for_level, xp_to_next_level,
};
pub use progression::{
    DerivedStats, KnownAbility, LevelUpReward, ResourcePool, SavedCharacter, ability_unlock_level,
    class_progression, mana_growth, stamina_growth,
};
