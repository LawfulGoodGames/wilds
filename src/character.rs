pub const MAX_LEVEL: i32 = 20;
pub const STAT_POINTS: i32 = 6;
pub const MAX_PROFICIENCY_LEVEL: u32 = 99;

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
            Self::Cooking => "Cooking",
            Self::Blacksmithing => "Blacksmithing",
            Self::Mining => "Mining",
            Self::Woodcutting => "Woodcutting",
            Self::Fishing => "Fishing",
            Self::Herbalism => "Herbalism",
            Self::Farming => "Farming",
            Self::Crafting => "Crafting",
            Self::Enchanting => "Enchanting",
            Self::Thieving => "Thieving",
            Self::Prayer => "Prayer",
            Self::Runecrafting => "Runecrafting",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Cooking => "Prepare field meals and restorative dishes.",
            Self::Blacksmithing => "Shape metal gear and understand armor quality.",
            Self::Mining => "Recover ore, stone, and buried valuables.",
            Self::Woodcutting => "Harvest timber and survive the deep wilds.",
            Self::Fishing => "Gather food and supplies from rivers and lakes.",
            Self::Herbalism => "Identify herbs and distill useful tonics.",
            Self::Farming => "Raise staple goods and maintain camp stores.",
            Self::Crafting => "Assemble leatherwork, talismans, and tools.",
            Self::Enchanting => "Improve magical gear and stabilize relics.",
            Self::Thieving => "Slip through danger and work with light hands.",
            Self::Prayer => "Hold to rites of warding, healing, and resolve.",
            Self::Runecrafting => "Channel raw magical script into useful power.",
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
            Self::Strength => "STR",
            Self::Dexterity => "DEX",
            Self::Constitution => "CON",
            Self::Intelligence => "INT",
            Self::Wisdom => "WIS",
            Self::Charisma => "CHA",
        }
    }

    pub fn full_name(self) -> &'static str {
        match self {
            Self::Strength => "Strength",
            Self::Dexterity => "Dexterity",
            Self::Constitution => "Constitution",
            Self::Intelligence => "Intelligence",
            Self::Wisdom => "Wisdom",
            Self::Charisma => "Charisma",
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

pub const STAT_LABELS: [&str; 6] = ["STR", "DEX", "CON", "INT", "WIS", "CHA"];
pub const STAT_FULL: [&str; 6] = [
    "Strength",
    "Dexterity",
    "Constitution",
    "Intelligence",
    "Wisdom",
    "Charisma",
];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourcePool {
    pub hp: i32,
    pub max_hp: i32,
    pub mana: i32,
    pub max_mana: i32,
    pub stamina: i32,
    pub max_stamina: i32,
}

impl ResourcePool {
    pub fn full(max_hp: i32, max_mana: i32, max_stamina: i32) -> Self {
        Self {
            hp: max_hp,
            max_hp,
            mana: max_mana,
            max_mana,
            stamina: max_stamina,
            max_stamina,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DerivedStats {
    pub defense: i32,
    pub initiative: i32,
    pub crit_chance: i32,
    pub dodge: i32,
    pub spell_power: i32,
    pub healing_power: i32,
}

#[derive(Debug, Clone)]
pub struct KnownAbility {
    pub ability_id: String,
    pub rank: i32,
    pub unlocked: bool,
    pub cooldown_remaining: i32,
}

#[derive(Debug, Clone)]
pub struct LevelUpReward {
    pub levels_gained: i32,
    pub attribute_points_awarded: i32,
    pub new_ability_ids: Vec<String>,
    pub hp_gain: i32,
    pub mana_gain: i32,
    pub stamina_gain: i32,
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
            Self::Human => "+1 to all stats",
            Self::Elf => "+2 DEX, +1 INT",
            Self::Dwarf => "+2 CON, +1 STR",
            Self::Halfling => "+2 DEX, +1 CHA",
            Self::Orc => "+2 STR, +1 CON",
            Self::Tiefling => "+2 CHA, +1 INT",
            Self::Gnome => "+2 INT, +1 WIS",
            Self::Dragonborn => "+2 STR, +1 CHA",
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
            Self::Warrior => "STR, CON",
            Self::Ranger => "DEX, WIS",
            Self::Mage => "INT, WIS",
            Self::Rogue => "DEX, CHA",
            Self::Paladin => "STR, CHA",
            Self::Cleric => "WIS, CHA",
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
            Self::Stats => "Stats",
            Self::Gear => "Gear",
            Self::Confirm => "Confirm",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SavedCharacter {
    pub id: i64,
    pub name: String,
    pub race: Race,
    pub class: Class,
    pub gear: String,
    pub level: i32,
    pub xp: i32,
    pub gold: i32,
    pub unspent_stat_points: i32,
    pub stats: Stats,
    pub resources: ResourcePool,
    pub proficiencies: Vec<ProficiencyData>,
    pub known_abilities: Vec<KnownAbility>,
}

impl SavedCharacter {
    pub fn major_skill(&self, kind: MajorSkill) -> i32 {
        self.stats.by_skill(kind)
    }

    pub fn derived_stats(&self, equipment_armor: i32, _attack_bonus: i32, spell_power_bonus: i32, crit_bonus: i32, initiative_bonus: i32) -> DerivedStats {
        let dex = self.stats.modifier(MajorSkill::Dexterity);
        let wis = self.stats.modifier(MajorSkill::Wisdom);
        let int = self.stats.modifier(MajorSkill::Intelligence);
        let cha = self.stats.modifier(MajorSkill::Charisma);
        DerivedStats {
            defense: 10 + equipment_armor + dex + wis.max(0) / 2,
            initiative: dex + initiative_bonus + self.level / 3,
            crit_chance: 5 + crit_bonus + dex.max(0) * 2 + cha.max(0),
            dodge: dex * 2 + self.level / 2,
            spell_power: int * 2 + spell_power_bonus + self.level,
            healing_power: wis * 2 + cha.max(0) + self.level / 2,
        }
    }

    pub fn apply_xp_gain(&mut self, gained_xp: i32) -> LevelUpReward {
        let before_level = self.level;
        self.xp += gained_xp.max(0);
        let new_level = level_from_xp(self.xp);
        self.level = new_level;

        if new_level <= before_level {
            return LevelUpReward {
                levels_gained: 0,
                attribute_points_awarded: 0,
                new_ability_ids: vec![],
                hp_gain: 0,
                mana_gain: 0,
                stamina_gain: 0,
            };
        }

        let levels_gained = new_level - before_level;
        let hp_gain = 8 * levels_gained + self.stats.modifier(MajorSkill::Constitution).max(1) * levels_gained;
        let mana_gain = mana_growth(self.class) * levels_gained;
        let stamina_gain = stamina_growth(self.class) * levels_gained;
        self.resources.max_hp += hp_gain;
        self.resources.max_mana += mana_gain;
        self.resources.max_stamina += stamina_gain;
        self.resources.hp = self.resources.max_hp;
        self.resources.mana = self.resources.max_mana;
        self.resources.stamina = self.resources.max_stamina;

        let stat_points = (before_level + 1..=new_level)
            .filter(|level| level % 2 == 0)
            .count() as i32;
        self.unspent_stat_points += stat_points;

        let mut unlocked = vec![];
        for (_, ability_id) in class_progression(self.class).unlocks {
            if self
                .known_abilities
                .iter()
                .any(|known| known.ability_id == ability_id)
            {
                continue;
            }
            if ability_unlock_level(self.class, ability_id) <= new_level {
                self.known_abilities.push(KnownAbility {
                    ability_id: ability_id.to_string(),
                    rank: 1,
                    unlocked: true,
                    cooldown_remaining: 0,
                });
                unlocked.push(ability_id.to_string());
            }
        }

        LevelUpReward {
            levels_gained,
            attribute_points_awarded: stat_points,
            new_ability_ids: unlocked,
            hp_gain,
            mana_gain,
            stamina_gain,
        }
    }
}

pub fn mana_growth(class: Class) -> i32 {
    match class {
        Class::Mage => 7,
        Class::Cleric => 6,
        Class::Paladin => 3,
        _ => 2,
    }
}

pub fn stamina_growth(class: Class) -> i32 {
    match class {
        Class::Warrior => 6,
        Class::Ranger => 5,
        Class::Rogue => 5,
        Class::Paladin => 4,
        _ => 2,
    }
}

pub fn class_progression(class: Class) -> CharacterClassProgression {
    let unlocks = match class {
        Class::Warrior => vec![(1, "guard_stance"), (2, "cleaving_blow"), (4, "shield_bash")],
        Class::Ranger => vec![(1, "hunters_mark"), (2, "volley"), (4, "crippling_shot")],
        Class::Mage => vec![(1, "ember_burst"), (2, "frost_lance"), (4, "storm_surge")],
        Class::Rogue => vec![(1, "dirty_cut"), (2, "evasion"), (4, "shadow_flurry")],
        Class::Paladin => vec![(1, "radiant_slam"), (2, "vow_guard"), (4, "lay_on_hands")],
        Class::Cleric => vec![(1, "healing_prayer"), (2, "smite_undead"), (4, "purge")],
    };
    CharacterClassProgression { class, unlocks }
}

pub fn ability_unlock_level(class: Class, ability_id: &str) -> i32 {
    class_progression(class)
        .unlocks
        .into_iter()
        .find(|(_, id)| *id == ability_id)
        .map(|(level, _)| level)
        .unwrap_or(99)
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
        self.base_stats.add_bonuses(&self.selected_race().stat_bonuses())
    }

    pub fn adjust_stat(&mut self, dir: i32) {
        let current = self.base_stats.get(self.stat_cursor);
        if dir > 0 && self.points_remaining > 0 && current < 13 {
            self.base_stats.add(self.stat_cursor, 1);
            self.points_remaining -= 1;
        } else if dir < 0 && current > 8 {
            self.base_stats.add(self.stat_cursor, -1);
            self.points_remaining += 1;
        }
    }
}
