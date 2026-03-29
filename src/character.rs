// ── Types ────────────────────────────────────────────────────────────────────

pub const STAT_LABELS: [&str; 6] = ["STR", "DEX", "CON", "INT", "WIS", "CHA"];
pub const STAT_FULL: [&str; 6] = [
    "Strength",
    "Dexterity",
    "Constitution",
    "Intelligence",
    "Wisdom",
    "Charisma",
];

// ── Stats ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
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
            strength: 5,
            dexterity: 5,
            constitution: 5,
            intelligence: 5,
            wisdom: 5,
            charisma: 5,
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

    /// Returns a new Stats that is self + other (for applying race bonuses).
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
}

// ── Race ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
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
            Race::Human => "Human",
            Race::Elf => "Elf",
            Race::Dwarf => "Dwarf",
            Race::Halfling => "Halfling",
            Race::Orc => "Orc",
            Race::Tiefling => "Tiefling",
            Race::Gnome => "Gnome",
            Race::Dragonborn => "Dragonborn",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Race::Human =>      "Adaptable and ambitious, found everywhere.",
            Race::Elf =>        "Ancient and graceful, attuned to nature.",
            Race::Dwarf =>      "Hardy mountain folk, masters of craft.",
            Race::Halfling =>   "Small and nimble, surprisingly lucky.",
            Race::Orc =>        "Fierce warriors born from wild lands.",
            Race::Tiefling =>   "Touched by infernal power, misunderstood.",
            Race::Gnome =>      "Inventive tinkerers with boundless curiosity.",
            Race::Dragonborn => "Proud draconic heritage, breath of fire.",
        }
    }

    pub fn bonus_label(self) -> &'static str {
        match self {
            Race::Human =>      "+1 to all stats",
            Race::Elf =>        "+2 DEX, +1 INT",
            Race::Dwarf =>      "+2 CON, +1 STR",
            Race::Halfling =>   "+2 DEX, +1 CHA",
            Race::Orc =>        "+2 STR, +1 CON",
            Race::Tiefling =>   "+2 CHA, +1 INT",
            Race::Gnome =>      "+2 INT, +1 WIS",
            Race::Dragonborn => "+2 STR, +1 CHA",
        }
    }

    pub fn stat_bonuses(self) -> Stats {
        let mut s = Stats {
            strength: 0,
            dexterity: 0,
            constitution: 0,
            intelligence: 0,
            wisdom: 0,
            charisma: 0,
        };
        match self {
            Race::Human => {
                s.strength = 1; s.dexterity = 1; s.constitution = 1;
                s.intelligence = 1; s.wisdom = 1; s.charisma = 1;
            }
            Race::Elf =>        { s.dexterity = 2; s.intelligence = 1; }
            Race::Dwarf =>      { s.constitution = 2; s.strength = 1; }
            Race::Halfling =>   { s.dexterity = 2; s.charisma = 1; }
            Race::Orc =>        { s.strength = 2; s.constitution = 1; }
            Race::Tiefling =>   { s.charisma = 2; s.intelligence = 1; }
            Race::Gnome =>      { s.intelligence = 2; s.wisdom = 1; }
            Race::Dragonborn => { s.strength = 2; s.charisma = 1; }
        }
        s
    }
}

// ── Class ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
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
            Class::Warrior => "Warrior",
            Class::Ranger  => "Ranger",
            Class::Mage    => "Mage",
            Class::Rogue   => "Rogue",
            Class::Paladin => "Paladin",
            Class::Cleric  => "Cleric",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Class::Warrior => "Master of arms and armor. Leads from the front.",
            Class::Ranger  => "Swift hunter and keen tracker of the wilds.",
            Class::Mage    => "Scholar of the arcane. Power through knowledge.",
            Class::Rogue   => "Quick, cunning, and deadly from the shadows.",
            Class::Paladin => "Holy knight bound by oath and divine power.",
            Class::Cleric  => "Servant of the divine. Healer and protector.",
        }
    }

    pub fn primary_stats(self) -> &'static str {
        match self {
            Class::Warrior => "STR, CON",
            Class::Ranger  => "DEX, WIS",
            Class::Mage    => "INT, WIS",
            Class::Rogue   => "DEX, CHA",
            Class::Paladin => "STR, CHA",
            Class::Cleric  => "WIS, CHA",
        }
    }
}

// ── Gear ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
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
            GearPackage::Melee   => "Melee Kit",
            GearPackage::Ranged  => "Ranged Kit",
            GearPackage::Arcane  => "Arcane Kit",
            GearPackage::Stealth => "Stealth Kit",
        }
    }

    pub fn items(self) -> &'static str {
        match self {
            GearPackage::Melee   => "Iron Sword, Wooden Shield, Leather Armor",
            GearPackage::Ranged  => "Hunting Bow, 20 Arrows, Leather Cloak",
            GearPackage::Arcane  => "Oak Staff, Spellbook, Mana Potion x3",
            GearPackage::Stealth => "Twin Daggers, Lockpicks, Shadow Cloak",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            GearPackage::Melee   => "Stand your ground and fight face-to-face.",
            GearPackage::Ranged  => "Strike from distance before they close in.",
            GearPackage::Arcane  => "Channel the arcane to overwhelm your foes.",
            GearPackage::Stealth => "Stay hidden, strike fast, leave no trace.",
        }
    }
}

// ── Creation Step ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CreationStep {
    Name,
    Race,
    Class,
    Stats,
    Gear,
    Confirm,
}

impl CreationStep {
    pub fn next(self) -> Self {
        match self {
            Self::Name    => Self::Race,
            Self::Race    => Self::Class,
            Self::Class   => Self::Stats,
            Self::Stats   => Self::Gear,
            Self::Gear    => Self::Confirm,
            Self::Confirm => Self::Confirm,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Name    => Self::Name,
            Self::Race    => Self::Name,
            Self::Class   => Self::Race,
            Self::Stats   => Self::Class,
            Self::Gear    => Self::Stats,
            Self::Confirm => Self::Gear,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::Name    => 0,
            Self::Race    => 1,
            Self::Class   => 2,
            Self::Stats   => 3,
            Self::Gear    => 4,
            Self::Confirm => 5,
        }
    }

    pub const ALL: [CreationStep; 6] = [
        Self::Name,
        Self::Race,
        Self::Class,
        Self::Stats,
        Self::Gear,
        Self::Confirm,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Name    => "Name",
            Self::Race    => "Race",
            Self::Class   => "Class",
            Self::Stats   => "Stats",
            Self::Gear    => "Gear",
            Self::Confirm => "Confirm",
        }
    }
}

// ── SavedCharacter ────────────────────────────────────────────────────────────

/// A fully persisted character loaded from the database.
#[derive(Debug, Clone)]
pub struct SavedCharacter {
    pub id:       i64,
    pub name:     String,
    pub race:     String,
    pub class:    String,
    pub gear:     String,
    pub level:    i32,
    pub xp:       i32,
    pub hp:       i32,
    pub max_hp:   i32,
    pub gold:     i32,
    pub str_stat: i32,
    pub dex_stat: i32,
    pub con_stat: i32,
    pub int_stat: i32,
    pub wis_stat: i32,
    pub cha_stat: i32,
}

// ── CharacterCreation state ───────────────────────────────────────────────────

pub const STAT_POINTS: i32 = 6;

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

    /// Final stats = base allocation + race bonuses.
    pub fn final_stats(&self) -> Stats {
        self.base_stats.add_bonuses(&self.selected_race().stat_bonuses())
    }

    pub fn adjust_stat(&mut self, dir: i32) {
        let current = self.base_stats.get(self.stat_cursor);
        if dir > 0 && self.points_remaining > 0 && current < 10 {
            self.base_stats.add(self.stat_cursor, 1);
            self.points_remaining -= 1;
        } else if dir < 0 && current > 5 {
            self.base_stats.add(self.stat_cursor, -1);
            self.points_remaining += 1;
        }
    }
}
