use super::proficiencies::{MajorSkill, STAT_POINTS, Stats};

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
            Self::Elf => "+2 Ranged, +1 Magic",
            Self::Dwarf => "+2 Defence, +1 Strength",
            Self::Halfling => "+2 Ranged, +1 Attack",
            Self::Orc => "+2 Strength, +1 Defence",
            Self::Tiefling => "+2 Attack, +1 Magic",
            Self::Gnome => "+2 Magic, +1 Prayer",
            Self::Dragonborn => "+2 Strength, +1 Attack",
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

    pub fn from_name(name: &str) -> Self {
        Self::ALL
            .iter()
            .copied()
            .find(|race| race.name() == name)
            .unwrap_or(Self::Human)
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
            .unwrap_or(Self::Warrior)
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

    pub fn adjust_stat(&mut self, dir: i32) {
        let skill = MajorSkill::ALL[self.stat_cursor];
        let current = self.base_stats.by_skill(skill);
        if dir > 0 && self.points_remaining > 0 && current < 13 {
            self.base_stats.add_skill(skill, 1);
            self.points_remaining -= 1;
        } else if dir < 0 && current > 8 {
            self.base_stats.add_skill(skill, -1);
            self.points_remaining += 1;
        }
    }
}
