// ── XP / Level math ──────────────────────────────────────────────────────────

pub const MAX_SKILL_LEVEL: u32 = 99;

/// Total XP required to reach `level` (RuneScape formula).
/// Level 1 = 0 XP, Level 2 = 83 XP, Level 99 = 13,034,431 XP.
pub fn xp_for_level(level: u32) -> u32 {
    if level <= 1 {
        return 0;
    }
    let mut points: f64 = 0.0;
    for i in 1..(level as usize) {
        points += f64::floor(i as f64 + 300.0 * f64::powf(2.0, i as f64 / 7.0));
    }
    f64::floor(points / 4.0) as u32
}

/// Current level for a given amount of XP (1–99).
pub fn level_from_xp(xp: i32) -> u32 {
    for lvl in (1..=MAX_SKILL_LEVEL).rev() {
        if xp as u32 >= xp_for_level(lvl) {
            return lvl;
        }
    }
    1
}

/// XP still needed to reach the next level.
pub fn xp_to_next_level(xp: i32) -> u32 {
    let current = level_from_xp(xp);
    if current >= MAX_SKILL_LEVEL {
        return 0;
    }
    xp_for_level(current + 1).saturating_sub(xp as u32)
}

/// Progress fraction (0.0–1.0) through the current level.
pub fn level_progress_pct(xp: i32) -> f64 {
    let current = level_from_xp(xp);
    if current >= MAX_SKILL_LEVEL {
        return 1.0;
    }
    let start = xp_for_level(current) as f64;
    let end   = xp_for_level(current + 1) as f64;
    ((xp as f64 - start) / (end - start)).clamp(0.0, 1.0)
}

// ── Skills ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MinorSkill {
    Cooking,
    Blacksmithing,
    Mining,
    Woodcutting,
    Fishing,
    Herbalism,
    Farming,
    Crafting,
    Enchanting,
    Thieving,
    Prayer,
    Runecrafting,
}

impl MinorSkill {
    pub const ALL: [MinorSkill; 12] = [
        MinorSkill::Cooking,
        MinorSkill::Blacksmithing,
        MinorSkill::Mining,
        MinorSkill::Woodcutting,
        MinorSkill::Fishing,
        MinorSkill::Herbalism,
        MinorSkill::Farming,
        MinorSkill::Crafting,
        MinorSkill::Enchanting,
        MinorSkill::Thieving,
        MinorSkill::Prayer,
        MinorSkill::Runecrafting,
    ];

    pub fn name(self) -> &'static str {
        match self {
            MinorSkill::Cooking       => "Cooking",
            MinorSkill::Blacksmithing => "Blacksmithing",
            MinorSkill::Mining        => "Mining",
            MinorSkill::Woodcutting   => "Woodcutting",
            MinorSkill::Fishing       => "Fishing",
            MinorSkill::Herbalism     => "Herbalism",
            MinorSkill::Farming       => "Farming",
            MinorSkill::Crafting      => "Crafting",
            MinorSkill::Enchanting    => "Enchanting",
            MinorSkill::Thieving      => "Thieving",
            MinorSkill::Prayer        => "Prayer",
            MinorSkill::Runecrafting  => "Runecrafting",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            MinorSkill::Cooking       => "Prepare food that restores HP and grants buffs.",
            MinorSkill::Blacksmithing => "Forge weapons and armor from raw ore.",
            MinorSkill::Mining        => "Extract ore, gems, and stone from the earth.",
            MinorSkill::Woodcutting   => "Fell trees and gather wood for crafting.",
            MinorSkill::Fishing       => "Catch fish from rivers, lakes, and seas.",
            MinorSkill::Herbalism     => "Gather herbs and brew potions.",
            MinorSkill::Farming       => "Grow crops and tend livestock.",
            MinorSkill::Crafting      => "Create items from leather, cloth, and bone.",
            MinorSkill::Enchanting    => "Imbue items with magical properties.",
            MinorSkill::Thieving      => "Pick pockets, crack locks, and move unseen.",
            MinorSkill::Prayer        => "Channel divine favour for blessings.",
            MinorSkill::Runecrafting  => "Craft runes used in spellcasting.",
        }
    }

    pub fn from_str(s: &str) -> Option<MinorSkill> {
        match s {
            "Cooking"       => Some(MinorSkill::Cooking),
            "Blacksmithing" => Some(MinorSkill::Blacksmithing),
            "Mining"        => Some(MinorSkill::Mining),
            "Woodcutting"   => Some(MinorSkill::Woodcutting),
            "Fishing"       => Some(MinorSkill::Fishing),
            "Herbalism"     => Some(MinorSkill::Herbalism),
            "Farming"       => Some(MinorSkill::Farming),
            "Crafting"      => Some(MinorSkill::Crafting),
            "Enchanting"    => Some(MinorSkill::Enchanting),
            "Thieving"      => Some(MinorSkill::Thieving),
            "Prayer"        => Some(MinorSkill::Prayer),
            "Runecrafting"  => Some(MinorSkill::Runecrafting),
            _               => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MinorSkillData {
    pub kind: MinorSkill,
    pub xp:   i32,
}

impl MinorSkillData {
    pub fn level(&self) -> u32    { level_from_xp(self.xp) }
    pub fn xp_to_next(&self) -> u32 { xp_to_next_level(self.xp) }
    pub fn progress(&self) -> f64 { level_progress_pct(self.xp) }
}

// ── Major Skills ──────────────────────────────────────────────────────────────

/// The 6 core stats set at character creation. Governed by combat and class.
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
        MajorSkill::Strength,
        MajorSkill::Dexterity,
        MajorSkill::Constitution,
        MajorSkill::Intelligence,
        MajorSkill::Wisdom,
        MajorSkill::Charisma,
    ];

    pub fn short_name(self) -> &'static str {
        match self {
            MajorSkill::Strength     => "STR",
            MajorSkill::Dexterity    => "DEX",
            MajorSkill::Constitution => "CON",
            MajorSkill::Intelligence => "INT",
            MajorSkill::Wisdom       => "WIS",
            MajorSkill::Charisma     => "CHA",
        }
    }

    pub fn full_name(self) -> &'static str {
        match self {
            MajorSkill::Strength     => "Strength",
            MajorSkill::Dexterity    => "Dexterity",
            MajorSkill::Constitution => "Constitution",
            MajorSkill::Intelligence => "Intelligence",
            MajorSkill::Wisdom       => "Wisdom",
            MajorSkill::Charisma     => "Charisma",
        }
    }

    /// The DB column name for this skill in the `characters` table.
    pub fn db_column(self) -> &'static str {
        match self {
            MajorSkill::Strength     => "str_stat",
            MajorSkill::Dexterity    => "dex_stat",
            MajorSkill::Constitution => "con_stat",
            MajorSkill::Intelligence => "int_stat",
            MajorSkill::Wisdom       => "wis_stat",
            MajorSkill::Charisma     => "cha_stat",
        }
    }
}

/// A major skill value for a loaded character.
#[derive(Debug, Clone)]
pub struct MajorSkillData {
    pub kind:   MajorSkill,
    pub points: i32,
}

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
    /// The 6 core stats in `MajorSkill::ALL` order.
    pub major_skills: Vec<MajorSkillData>,
    /// The 12 non-combat skills in `MinorSkill::ALL` order.
    pub minor_skills: Vec<MinorSkillData>,
}

impl SavedCharacter {
    /// Convenience getter — returns the point value for a major skill.
    pub fn major_skill(&self, kind: MajorSkill) -> i32 {
        self.major_skills
            .iter()
            .find(|s| s.kind == kind)
            .map(|s| s.points)
            .unwrap_or(5)
    }
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
