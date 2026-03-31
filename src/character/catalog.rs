use super::proficiencies::{MajorSkill, MinorSkill, STAT_POINTS, Stats, proficiency_xp_for_level};

const CREATION_BASE_PROFICIENCY: i32 = 8;
const CREATION_MAX_PROFICIENCY: i32 = 13;

#[derive(Debug, Clone, Copy, Default)]
pub struct MinorSkillBonuses {
    pub vitality: i32,
    pub agility: i32,
    pub alchemy: i32,
    pub larceny: i32,
    pub runecraft: i32,
    pub crafting: i32,
}

impl MinorSkillBonuses {
    pub fn by_skill(&self, skill: MinorSkill) -> i32 {
        match skill {
            MinorSkill::Vitality => self.vitality,
            MinorSkill::Agility => self.agility,
            MinorSkill::Alchemy => self.alchemy,
            MinorSkill::Larceny => self.larceny,
            MinorSkill::Runecraft => self.runecraft,
            MinorSkill::Crafting => self.crafting,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Race {
    Human,
    Elf,
    Dwarf,
    Halfling,
    Orc,
    Tiefling,
    Gnome,
    Dragonborn,
}

impl Race {
    pub const ALL: [Race; 8] = [
        Race::Human,
        Race::Elf,
        Race::Dwarf,
        Race::Halfling,
        Race::Orc,
        Race::Tiefling,
        Race::Gnome,
        Race::Dragonborn,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Human => "Human",
            Self::Elf => "Elf",
            Self::Dwarf => "Dwarf",
            Self::Halfling => "Halfling",
            Self::Orc => "Orc",
            Self::Tiefling => "Tiefling",
            Self::Gnome => "Gnome",
            Self::Dragonborn => "Dragonborn",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Human => "Adaptable frontier survivors with broad training.",
            Self::Elf => "Quick and disciplined, gifted in sight and spellcraft.",
            Self::Dwarf => "Durable keepers of steel, stone, and stubborn will.",
            Self::Halfling => "Small-footed opportunists with calm nerve under pressure.",
            Self::Orc => "Relentless warriors built for impact and endurance.",
            Self::Tiefling => "Canny and forceful, comfortable around dangerous power.",
            Self::Gnome => "Sharp-minded planners with excellent magical instinct.",
            Self::Dragonborn => "Bold champions with commanding presence and martial pride.",
        }
    }

    pub fn bonus_label(self) -> &'static str {
        match self {
            Self::Human => "+1 to all proficiencies",
            Self::Elf => "+2 Ranged, +1 Magic, +1 Agility, +1 Runecraft",
            Self::Dwarf => "+2 Defence, +1 Strength, +1 Vitality, +1 Crafting, +1 Alchemy",
            Self::Halfling => "+2 Ranged, +1 Attack, +1 Agility, +1 Larceny",
            Self::Orc => "+2 Strength, +1 Defence, +1 Vitality, +1 Crafting",
            Self::Tiefling => "+2 Attack, +1 Magic, +1 Alchemy, +1 Larceny, +1 Runecraft",
            Self::Gnome => "+2 Magic, +1 Prayer, +1 Alchemy, +1 Runecraft, +1 Crafting",
            Self::Dragonborn => "+2 Strength, +1 Attack, +1 Vitality, +1 Crafting",
        }
    }

    pub fn stat_bonuses(self) -> Stats {
        let mut out = Stats {
            strength: 0,
            dexterity: 0,
            constitution: 0,
            intelligence: 0,
            wisdom: 0,
            charisma: 0,
        };
        match self {
            Self::Human => {
                out.strength = 1;
                out.dexterity = 1;
                out.constitution = 1;
                out.intelligence = 1;
                out.wisdom = 1;
                out.charisma = 1;
            }
            Self::Elf => {
                out.dexterity = 2;
                out.intelligence = 1;
            }
            Self::Dwarf => {
                out.constitution = 2;
                out.strength = 1;
            }
            Self::Halfling => {
                out.dexterity = 2;
                out.charisma = 1;
            }
            Self::Orc => {
                out.strength = 2;
                out.constitution = 1;
            }
            Self::Tiefling => {
                out.charisma = 2;
                out.intelligence = 1;
            }
            Self::Gnome => {
                out.intelligence = 2;
                out.wisdom = 1;
            }
            Self::Dragonborn => {
                out.strength = 2;
                out.charisma = 1;
            }
        }
        out
    }

    pub fn minor_skill_bonuses(self) -> MinorSkillBonuses {
        let mut out = MinorSkillBonuses::default();
        match self {
            Self::Human => {
                out.vitality = 1;
                out.agility = 1;
                out.alchemy = 1;
                out.larceny = 1;
                out.runecraft = 1;
                out.crafting = 1;
            }
            Self::Elf => {
                out.agility = 1;
                out.runecraft = 1;
            }
            Self::Dwarf => {
                out.vitality = 1;
                out.crafting = 1;
                out.alchemy = 1;
            }
            Self::Halfling => {
                out.agility = 1;
                out.larceny = 1;
            }
            Self::Orc => {
                out.vitality = 1;
                out.crafting = 1;
            }
            Self::Tiefling => {
                out.alchemy = 1;
                out.larceny = 1;
                out.runecraft = 1;
            }
            Self::Gnome => {
                out.alchemy = 1;
                out.runecraft = 1;
                out.crafting = 1;
            }
            Self::Dragonborn => {
                out.vitality = 1;
                out.crafting = 1;
            }
        }
        out
    }

    pub fn from_name(name: &str) -> Self {
        Self::ALL
            .iter()
            .copied()
            .find(|race| race.name() == name)
            .unwrap_or_else(|| panic!("Unknown race stored in save data: {name}"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Warrior,
    Ranger,
    Mage,
    Rogue,
    Paladin,
    Cleric,
}

impl Class {
    pub const ALL: [Class; 6] = [
        Class::Warrior,
        Class::Ranger,
        Class::Mage,
        Class::Rogue,
        Class::Paladin,
        Class::Cleric,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Warrior => "Warrior",
            Self::Ranger => "Ranger",
            Self::Mage => "Mage",
            Self::Rogue => "Rogue",
            Self::Paladin => "Paladin",
            Self::Cleric => "Cleric",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Warrior => "Front-line bruiser with shields, stamina, and crushing blows.",
            Self::Ranger => "Fast skirmisher with pressure shots and marks.",
            Self::Mage => "Elemental caster with strong mana scaling and control.",
            Self::Rogue => "Precise striker built around bleed, evasion, and tempo.",
            Self::Paladin => "Armored zealot with holy strikes and self-sustain.",
            Self::Cleric => "Support caster with healing, cleansing, and radiant damage.",
        }
    }

    pub fn primary_stats(self) -> &'static str {
        match self {
            Self::Warrior => "Attack, Strength",
            Self::Ranger => "Ranged, Defence",
            Self::Mage => "Magic, Prayer",
            Self::Rogue => "Attack, Ranged",
            Self::Paladin => "Strength, Prayer",
            Self::Cleric => "Prayer, Magic",
        }
    }

    pub fn from_name(name: &str) -> Self {
        Self::ALL
            .iter()
            .copied()
            .find(|class| class.name() == name)
            .unwrap_or_else(|| panic!("Unknown class stored in save data: {name}"))
    }
}

#[derive(Debug, Clone)]
pub struct CharacterClassProgression {
    pub class: Class,
    pub unlocks: Vec<(i32, &'static str)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GearPackage {
    Melee,
    Ranged,
    Arcane,
    Stealth,
}

impl GearPackage {
    pub const ALL: [GearPackage; 4] = [
        GearPackage::Melee,
        GearPackage::Ranged,
        GearPackage::Arcane,
        GearPackage::Stealth,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Melee => "Melee Kit",
            Self::Ranged => "Ranged Kit",
            Self::Arcane => "Arcane Kit",
            Self::Stealth => "Stealth Kit",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Melee => "Shield and steel for holding the line.",
            Self::Ranged => "Distance, movement, and controlled shots.",
            Self::Arcane => "Focus, robes, and mana-forward combat tools.",
            Self::Stealth => "Light gear and quick kills from bad angles.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreationStep {
    Name,
    Race,
    Class,
    Stats,
    Gear,
    Confirm,
}

impl CreationStep {
    pub const ALL: [CreationStep; 6] = [
        Self::Name,
        Self::Race,
        Self::Class,
        Self::Stats,
        Self::Gear,
        Self::Confirm,
    ];

    pub fn next(self) -> Self {
        match self {
            Self::Name => Self::Race,
            Self::Race => Self::Class,
            Self::Class => Self::Stats,
            Self::Stats => Self::Gear,
            Self::Gear => Self::Confirm,
            Self::Confirm => Self::Confirm,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Name => Self::Name,
            Self::Race => Self::Name,
            Self::Class => Self::Race,
            Self::Stats => Self::Class,
            Self::Gear => Self::Stats,
            Self::Confirm => Self::Gear,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::Name => 0,
            Self::Race => 1,
            Self::Class => 2,
            Self::Stats => 3,
            Self::Gear => 4,
            Self::Confirm => 5,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::Race => "Race",
            Self::Class => "Class",
            Self::Stats => "Proficiencies",
            Self::Gear => "Gear",
            Self::Confirm => "Confirm",
        }
    }
}

#[derive(Debug)]
pub struct CharacterCreation {
    pub step: CreationStep,
    pub name: String,
    pub race_cursor: usize,
    pub class_cursor: usize,
    pub stat_cursor: usize,
    pub base_stats: Stats,
    pub minor_proficiencies: [i32; MinorSkill::ALL.len()],
    pub points_remaining: i32,
    pub gear_cursor: usize,
}

impl Default for CharacterCreation {
    fn default() -> Self {
        Self {
            step: CreationStep::Name,
            name: String::new(),
            race_cursor: 0,
            class_cursor: 0,
            stat_cursor: 0,
            base_stats: Stats::default(),
            minor_proficiencies: [CREATION_BASE_PROFICIENCY; MinorSkill::ALL.len()],
            points_remaining: STAT_POINTS,
            gear_cursor: 0,
        }
    }
}

impl CharacterCreation {
    pub fn selected_race(&self) -> Race {
        Race::ALL[self.race_cursor]
    }

    pub fn selected_class(&self) -> Class {
        Class::ALL[self.class_cursor]
    }

    pub fn selected_gear(&self) -> GearPackage {
        GearPackage::ALL[self.gear_cursor]
    }

    pub fn final_stats(&self) -> Stats {
        self.base_stats
            .add_bonuses(&self.selected_race().stat_bonuses())
    }

    pub fn proficiency_count(&self) -> usize {
        MajorSkill::ALL.len() + MinorSkill::ALL.len()
    }

    pub fn minor_proficiency_rank(&self, skill: MinorSkill) -> i32 {
        self.minor_proficiencies[Self::minor_index(skill)]
    }

    pub fn starting_proficiency_xp(&self, skill: MinorSkill) -> i32 {
        let total_rank = self.final_minor_proficiency_rank(skill);
        proficiency_xp_for_level(total_rank as u32) as i32
    }

    pub fn starting_major_proficiency_xp(&self, skill: MajorSkill) -> i32 {
        let total_rank = self.final_stats().by_skill(skill);
        proficiency_xp_for_level(total_rank as u32) as i32
    }

    pub fn final_minor_proficiency_rank(&self, skill: MinorSkill) -> i32 {
        self.minor_proficiency_rank(skill)
            + self.selected_race().minor_skill_bonuses().by_skill(skill)
    }

    fn minor_index(skill: MinorSkill) -> usize {
        MinorSkill::ALL
            .iter()
            .position(|candidate| *candidate == skill)
            .unwrap_or(0)
    }

    pub fn adjust_stat(&mut self, dir: i32) {
        if self.stat_cursor < MajorSkill::ALL.len() {
            let skill = MajorSkill::ALL[self.stat_cursor];
            let current = self.base_stats.by_skill(skill);
            if dir > 0 && self.points_remaining > 0 && current < CREATION_MAX_PROFICIENCY {
                self.base_stats.add_skill(skill, 1);
                self.points_remaining -= 1;
            } else if dir < 0 && current > CREATION_BASE_PROFICIENCY {
                self.base_stats.add_skill(skill, -1);
                self.points_remaining += 1;
            }
            return;
        }

        let minor_idx = self.stat_cursor - MajorSkill::ALL.len();
        if let Some(rank) = self.minor_proficiencies.get_mut(minor_idx) {
            if dir > 0 && self.points_remaining > 0 && *rank < CREATION_MAX_PROFICIENCY {
                *rank += 1;
                self.points_remaining -= 1;
            } else if dir < 0 && *rank > CREATION_BASE_PROFICIENCY {
                *rank -= 1;
                self.points_remaining += 1;
            }
        }
    }
}
