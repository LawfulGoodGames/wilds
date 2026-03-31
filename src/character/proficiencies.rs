pub const MAX_LEVEL: i32 = 20;
pub const STAT_POINTS: i32 = 16;
pub const MAX_PROFICIENCY_LEVEL: u32 = 100;
pub const MAX_COMBAT_PROFICIENCY_RANK: i32 = 99;

#[derive(Debug, Clone, Copy)]
pub struct StudyPlan {
    pub hours: i32,
    pub success_chance: i32,
    pub success_xp: i32,
    pub failure_xp: i32,
    pub governing_stat: MajorSkill,
}

#[derive(Debug, Clone, Copy)]
pub struct TrainingSessionPlan {
    pub governing_stat: MajorSkill,
    pub beats: usize,
    pub response_ticks: u32,
    pub poor_xp: i32,
    pub solid_xp: i32,
    pub great_xp: i32,
    pub poor_hours: i32,
    pub solid_hours: i32,
    pub great_hours: i32,
}

pub fn xp_for_level(level: i32) -> i32 {
    if level <= 1 {
        0
    } else {
        (level - 1) * (level - 1) * 120
    }
}

pub fn level_from_xp(xp: i32) -> i32 {
    for level in (1..=MAX_LEVEL).rev() {
        if xp >= xp_for_level(level) {
            return level;
        }
    }
    1
}

pub fn xp_to_next_level(xp: i32) -> i32 {
    let current = level_from_xp(xp);
    if current >= MAX_LEVEL {
        0
    } else {
        xp_for_level(current + 1) - xp
    }
}

pub fn level_progress_pct(xp: i32) -> f64 {
    let current = level_from_xp(xp);
    if current >= MAX_LEVEL {
        return 1.0;
    }
    let start = xp_for_level(current) as f64;
    let end = xp_for_level(current + 1) as f64;
    ((xp as f64 - start) / (end - start)).clamp(0.0, 1.0)
}

pub fn proficiency_xp_for_level(level: u32) -> u32 {
    if level <= 1 {
        0
    } else {
        (level - 1) * (level - 1) * 75
    }
}

pub fn proficiency_level_from_xp(xp: i32) -> u32 {
    for level in (1..=MAX_PROFICIENCY_LEVEL).rev() {
        if xp as u32 >= proficiency_xp_for_level(level) {
            return level;
        }
    }
    1
}

pub fn proficiency_progress_pct(xp: i32) -> f64 {
    let current = proficiency_level_from_xp(xp);
    if current >= MAX_PROFICIENCY_LEVEL {
        return 1.0;
    }
    let start = proficiency_xp_for_level(current) as f64;
    let end = proficiency_xp_for_level(current + 1) as f64;
    ((xp as f64 - start) / (end - start)).clamp(0.0, 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MinorSkill {
    Vitality,
    Agility,
    Alchemy,
    Larceny,
    Runecraft,
    Crafting,
}

impl MinorSkill {
    pub const ALL: [MinorSkill; 6] = [
        MinorSkill::Vitality,
        MinorSkill::Agility,
        MinorSkill::Alchemy,
        MinorSkill::Larceny,
        MinorSkill::Runecraft,
        MinorSkill::Crafting,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Vitality => "Vitality",
            Self::Agility => "Agility",
            Self::Alchemy => "Alchemy",
            Self::Larceny => "Larceny",
            Self::Runecraft => "Runecraft",
            Self::Crafting => "Crafting",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Vitality => "Condition the body to endure rough travel and attrition.",
            Self::Agility => "Move cleanly through rough ground, locks, and ledges.",
            Self::Alchemy => "Brew compounds, tonics, and useful reagents from the wild.",
            Self::Larceny => "Lift valuables, work fine mechanisms, and exploit openings.",
            Self::Runecraft => "Bind signs of power into runes, wards, and catalysts.",
            Self::Crafting => "Assemble leatherwork, charms, and fine practical tools.",
        }
    }

    pub fn governing_stat(self) -> MajorSkill {
        match self {
            Self::Vitality => MajorSkill::Constitution,
            Self::Agility => MajorSkill::Dexterity,
            Self::Alchemy => MajorSkill::Intelligence,
            Self::Larceny => MajorSkill::Dexterity,
            Self::Runecraft => MajorSkill::Intelligence,
            Self::Crafting => MajorSkill::Intelligence,
        }
    }

    pub fn effects_summary(self) -> &'static str {
        match self {
            Self::Vitality => {
                "Improves survivability themes, harsh travel endurance, and physical staying power."
            }
            Self::Agility => "Improves movement-heavy challenges, finesse, and evasive fieldcraft.",
            Self::Alchemy => "Improves brews, tonics, reagents, and technical concoctions.",
            Self::Larceny => "Improves theft, lockwork, trap handling, and exploiting openings.",
            Self::Runecraft => "Improves wards, rune work, catalysts, and arcane preparation.",
            Self::Crafting => {
                "Improves practical gearwork, leatherwork, charms, and item assembly."
            }
        }
    }

    pub fn training_focus(self) -> &'static str {
        match self {
            Self::Vitality => "Breath control and endurance drills",
            Self::Agility => "Balance steps and rapid repositioning",
            Self::Alchemy => "Formula recall and mixing cadence",
            Self::Larceny => "Precision timing and sleight rhythm",
            Self::Runecraft => "Glyph focus and pattern recall",
            Self::Crafting => "Hand rhythm and exact tool placement",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProficiencyData {
    pub kind: MinorSkill,
    pub xp: i32,
}

impl ProficiencyData {
    pub fn level(&self) -> u32 {
        proficiency_level_from_xp(self.xp)
    }

    pub fn progress(&self) -> f64 {
        proficiency_progress_pct(self.xp)
    }

    pub fn xp_to_next(&self) -> u32 {
        let level = self.level();
        if level >= MAX_PROFICIENCY_LEVEL {
            0
        } else {
            proficiency_xp_for_level(level + 1).saturating_sub(self.xp as u32)
        }
    }
}

#[derive(Debug, Clone)]
pub struct MajorProficiencyData {
    pub kind: MajorSkill,
    pub xp: i32,
}

impl MajorProficiencyData {
    pub fn level(&self) -> u32 {
        proficiency_level_from_xp(self.xp)
    }

    pub fn progress(&self) -> f64 {
        proficiency_progress_pct(self.xp)
    }

    pub fn xp_to_next(&self) -> u32 {
        let level = self.level();
        if level >= MAX_PROFICIENCY_LEVEL {
            0
        } else {
            proficiency_xp_for_level(level + 1).saturating_sub(self.xp as u32)
        }
    }
}

pub fn study_plan(skill: MinorSkill, xp: i32, stats: &Stats) -> StudyPlan {
    let level = proficiency_level_from_xp(xp) as i32;
    let aptitude = stats.modifier(skill.governing_stat());
    let success_chance = (88 - level * 4 + aptitude * 5).clamp(8, 92);
    let hours = match level {
        1..=9 => 4,
        10..=24 => 8,
        25..=39 => 12,
        40..=59 => 18,
        60..=79 => 28,
        80..=94 => 40,
        _ => 55,
    };
    let success_xp = (20 + aptitude.max(0) * 3 + (level / 8)).clamp(8, 36);
    let failure_xp = (success_xp / 5).max(1);
    StudyPlan {
        hours,
        success_chance,
        success_xp,
        failure_xp,
        governing_stat: skill.governing_stat(),
    }
}

pub fn major_study_plan(skill: MajorSkill, current_rank: i32, stats: &Stats) -> StudyPlan {
    let level = current_rank.clamp(1, MAX_PROFICIENCY_LEVEL as i32) as u32;
    major_study_plan_for_xp(skill, proficiency_xp_for_level(level) as i32, stats)
}

pub fn major_study_plan_for_xp(skill: MajorSkill, xp: i32, stats: &Stats) -> StudyPlan {
    let level = proficiency_level_from_xp(xp) as i32;
    let aptitude = stats.modifier(skill);
    let success_chance = (88 - level * 4 + aptitude * 5).clamp(8, 92);
    let hours = match level {
        1..=9 => 4,
        10..=24 => 8,
        25..=39 => 12,
        40..=59 => 18,
        60..=79 => 28,
        80..=94 => 40,
        _ => 55,
    };
    let success_xp = (20 + aptitude.max(0) * 3 + (level / 8)).clamp(8, 36);
    let failure_xp = (success_xp / 5).max(1);
    StudyPlan {
        hours,
        success_chance,
        success_xp,
        failure_xp,
        governing_stat: skill,
    }
}

pub fn training_session_plan_for_major(
    skill: MajorSkill,
    xp: i32,
    stats: &Stats,
) -> TrainingSessionPlan {
    training_session_plan(major_study_plan_for_xp(skill, xp, stats))
}

pub fn training_session_plan_for_minor(
    skill: MinorSkill,
    xp: i32,
    stats: &Stats,
) -> TrainingSessionPlan {
    training_session_plan(study_plan(skill, xp, stats))
}

fn training_session_plan(plan: StudyPlan) -> TrainingSessionPlan {
    let beats = if plan.success_chance <= 35 { 6 } else { 5 };
    let response_ticks = (8 + (plan.success_chance.max(8) as u32 / 7)).clamp(8, 20);
    let solid_xp = ((plan.success_xp + plan.failure_xp.max(1)) / 2).max(plan.failure_xp + 1);
    let poor_hours = plan.hours + (plan.hours / 3).max(1);
    let great_hours = (plan.hours - (plan.hours / 4).max(1)).max(1);
    TrainingSessionPlan {
        governing_stat: plan.governing_stat,
        beats,
        response_ticks,
        poor_xp: plan.failure_xp.max(1),
        solid_xp,
        great_xp: plan.success_xp,
        poor_hours,
        solid_hours: plan.hours,
        great_hours,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MajorSkill {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl MajorSkill {
    pub const ALL: [MajorSkill; 6] = [
        MajorSkill::Charisma,
        MajorSkill::Strength,
        MajorSkill::Constitution,
        MajorSkill::Dexterity,
        MajorSkill::Wisdom,
        MajorSkill::Intelligence,
    ];

    pub fn short_name(self) -> &'static str {
        match self {
            Self::Strength => "STR",
            Self::Dexterity => "RNG",
            Self::Constitution => "DEF",
            Self::Intelligence => "MAG",
            Self::Wisdom => "PRY",
            Self::Charisma => "ATK",
        }
    }

    pub fn full_name(self) -> &'static str {
        match self {
            Self::Strength => "Strength",
            Self::Dexterity => "Ranged",
            Self::Constitution => "Defence",
            Self::Intelligence => "Magic",
            Self::Wisdom => "Prayer",
            Self::Charisma => "Attack",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Strength => "Raw force behind heavy blows, labor, and stamina-heavy exertion.",
            Self::Dexterity => "Control at range that improves initiative, aim, and evasion.",
            Self::Constitution => "Toughness that hardens your guard and lets you stay upright.",
            Self::Intelligence => "Arcane command that improves spell power and mana craft.",
            Self::Wisdom => "Sacred focus used for rites, healing, and holy resolve.",
            Self::Charisma => "Martial timing and precision that sharpen direct attacks.",
        }
    }

    pub fn effects_summary(self) -> &'static str {
        match self {
            Self::Charisma => "Affects weapon accuracy, crit pressure, and direct martial tempo.",
            Self::Strength => "Affects physical damage, stamina-heavy actions, and melee scaling.",
            Self::Constitution => {
                "Affects defence, max HP growth, and staying power in long fights."
            }
            Self::Dexterity => "Affects ranged pressure, initiative, dodge, and battlefield tempo.",
            Self::Wisdom => {
                "Affects healing power, holy resilience, and support-oriented combat value."
            }
            Self::Intelligence => "Affects spell power, mana scaling, and arcane effectiveness.",
        }
    }

    pub fn training_focus(self) -> &'static str {
        match self {
            Self::Charisma => "Tempo drills and attack timing",
            Self::Strength => "Power bursts and impact control",
            Self::Constitution => "Guard stance and endurance breathing",
            Self::Dexterity => "Reaction snaps and ranged rhythm",
            Self::Wisdom => "Calm focus and restorative cadence",
            Self::Intelligence => "Arcane concentration and memory drills",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            strength: 8,
            dexterity: 8,
            constitution: 8,
            intelligence: 8,
            wisdom: 8,
            charisma: 8,
        }
    }
}

impl Stats {
    pub fn get(&self, idx: usize) -> i32 {
        match idx {
            0 => self.strength,
            1 => self.dexterity,
            2 => self.constitution,
            3 => self.intelligence,
            4 => self.wisdom,
            5 => self.charisma,
            _ => 0,
        }
    }

    pub fn add(&mut self, idx: usize, delta: i32) {
        match idx {
            0 => self.strength += delta,
            1 => self.dexterity += delta,
            2 => self.constitution += delta,
            3 => self.intelligence += delta,
            4 => self.wisdom += delta,
            5 => self.charisma += delta,
            _ => {}
        }
    }

    pub fn add_skill(&mut self, skill: MajorSkill, delta: i32) {
        match skill {
            MajorSkill::Strength => {
                self.strength = (self.strength + delta).clamp(1, MAX_COMBAT_PROFICIENCY_RANK)
            }
            MajorSkill::Dexterity => {
                self.dexterity = (self.dexterity + delta).clamp(1, MAX_COMBAT_PROFICIENCY_RANK)
            }
            MajorSkill::Constitution => {
                self.constitution =
                    (self.constitution + delta).clamp(1, MAX_COMBAT_PROFICIENCY_RANK)
            }
            MajorSkill::Intelligence => {
                self.intelligence =
                    (self.intelligence + delta).clamp(1, MAX_COMBAT_PROFICIENCY_RANK)
            }
            MajorSkill::Wisdom => {
                self.wisdom = (self.wisdom + delta).clamp(1, MAX_COMBAT_PROFICIENCY_RANK)
            }
            MajorSkill::Charisma => {
                self.charisma = (self.charisma + delta).clamp(1, MAX_COMBAT_PROFICIENCY_RANK)
            }
        }
    }

    pub fn add_bonuses(&self, other: &Stats) -> Stats {
        Stats {
            strength: self.strength + other.strength,
            dexterity: self.dexterity + other.dexterity,
            constitution: self.constitution + other.constitution,
            intelligence: self.intelligence + other.intelligence,
            wisdom: self.wisdom + other.wisdom,
            charisma: self.charisma + other.charisma,
        }
    }

    pub fn modifier(&self, skill: MajorSkill) -> i32 {
        (self.by_skill(skill) - 10).div_euclid(2)
    }

    pub fn by_skill(&self, skill: MajorSkill) -> i32 {
        match skill {
            MajorSkill::Strength => self.strength,
            MajorSkill::Dexterity => self.dexterity,
            MajorSkill::Constitution => self.constitution,
            MajorSkill::Intelligence => self.intelligence,
            MajorSkill::Wisdom => self.wisdom,
            MajorSkill::Charisma => self.charisma,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResistanceProfile {
    pub physical: i32,
    pub fire: i32,
    pub frost: i32,
    pub lightning: i32,
    pub poison: i32,
    pub holy: i32,
    pub shadow: i32,
}

impl Default for ResistanceProfile {
    fn default() -> Self {
        Self {
            physical: 0,
            fire: 0,
            frost: 0,
            lightning: 0,
            poison: 0,
            holy: 0,
            shadow: 0,
        }
    }
}

#[cfg(test)]
mod tests;
