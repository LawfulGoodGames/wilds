pub const MAX_LEVEL: i32 = 20;
pub const STAT_POINTS: i32 = 6;
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
    Bowcraft,
    Slayer,
    Hunting,
    Kindling,
    Runecraft,
    Building,
    Cooking,
    Mining,
    Smithing,
    Fishing,
    Farming,
    Crafting,
    Woodcutting,
}

impl MinorSkill {
    pub const ALL: [MinorSkill; 17] = [
        MinorSkill::Vitality,
        MinorSkill::Agility,
        MinorSkill::Alchemy,
        MinorSkill::Larceny,
        MinorSkill::Bowcraft,
        MinorSkill::Slayer,
        MinorSkill::Hunting,
        MinorSkill::Mining,
        MinorSkill::Smithing,
        MinorSkill::Fishing,
        MinorSkill::Cooking,
        MinorSkill::Farming,
        MinorSkill::Crafting,
        MinorSkill::Kindling,
        MinorSkill::Woodcutting,
        MinorSkill::Runecraft,
        MinorSkill::Building,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Vitality => "Vitality",
            Self::Agility => "Agility",
            Self::Alchemy => "Alchemy",
            Self::Larceny => "Larceny",
            Self::Bowcraft => "Bowcraft",
            Self::Slayer => "Slayer",
            Self::Hunting => "Hunting",
            Self::Kindling => "Kindling",
            Self::Runecraft => "Runecraft",
            Self::Building => "Building",
            Self::Cooking => "Cooking",
            Self::Mining => "Mining",
            Self::Smithing => "Smithing",
            Self::Fishing => "Fishing",
            Self::Farming => "Farming",
            Self::Crafting => "Crafting",
            Self::Woodcutting => "Woodcutting",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Vitality => "Condition the body to endure rough travel and attrition.",
            Self::Agility => "Move cleanly through rough ground, locks, and ledges.",
            Self::Alchemy => "Brew compounds, tonics, and useful reagents from the wild.",
            Self::Larceny => "Lift valuables, work fine mechanisms, and exploit openings.",
            Self::Bowcraft => "Shape shafts, string bows, and prepare ranged kit.",
            Self::Slayer => "Study monsters and learn the habits of dangerous prey.",
            Self::Hunting => "Track game, set snares, and read movement in the brush.",
            Self::Kindling => "Raise campfires fast and keep them burning in bad weather.",
            Self::Runecraft => "Bind signs of power into runes, wards, and catalysts.",
            Self::Building => "Raise shelters, repairs, and sturdy frontier fixtures.",
            Self::Cooking => "Prepare field meals and restorative dishes.",
            Self::Mining => "Recover ore, stone, and buried valuables.",
            Self::Smithing => "Shape metal gear and understand armor quality.",
            Self::Fishing => "Gather food and supplies from rivers and lakes.",
            Self::Farming => "Raise staple goods and maintain camp stores.",
            Self::Crafting => "Assemble leatherwork, charms, and fine practical tools.",
            Self::Woodcutting => "Harvest timber and break down useful hardwood.",
        }
    }

    pub fn governing_stat(self) -> MajorSkill {
        match self {
            Self::Vitality => MajorSkill::Constitution,
            Self::Agility => MajorSkill::Dexterity,
            Self::Alchemy => MajorSkill::Intelligence,
            Self::Larceny => MajorSkill::Dexterity,
            Self::Bowcraft => MajorSkill::Dexterity,
            Self::Slayer => MajorSkill::Charisma,
            Self::Hunting => MajorSkill::Dexterity,
            Self::Kindling => MajorSkill::Strength,
            Self::Runecraft => MajorSkill::Intelligence,
            Self::Building => MajorSkill::Strength,
            Self::Cooking => MajorSkill::Wisdom,
            Self::Mining => MajorSkill::Strength,
            Self::Smithing => MajorSkill::Strength,
            Self::Fishing => MajorSkill::Wisdom,
            Self::Farming => MajorSkill::Wisdom,
            Self::Crafting => MajorSkill::Intelligence,
            Self::Woodcutting => MajorSkill::Strength,
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
    let aptitude = stats.modifier(skill);
    let success_chance = (92 - current_rank * 4 + aptitude * 4).clamp(8, 92);
    let hours = match current_rank {
        ..=9 => 4,
        10..=12 => 8,
        13..=15 => 14,
        16..=18 => 22,
        _ => 32,
    };
    StudyPlan {
        hours,
        success_chance,
        success_xp: 1,
        failure_xp: 0,
        governing_stat: skill,
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

    pub fn derived_stats(
        &self,
        equipment_armor: i32,
        _attack_bonus: i32,
        spell_power_bonus: i32,
        crit_bonus: i32,
        initiative_bonus: i32,
    ) -> DerivedStats {
        let ranged = self.stats.modifier(MajorSkill::Dexterity);
        let prayer = self.stats.modifier(MajorSkill::Wisdom);
        let magic = self.stats.modifier(MajorSkill::Intelligence);
        let attack = self.stats.modifier(MajorSkill::Charisma);
        let defence = self.stats.modifier(MajorSkill::Constitution);
        DerivedStats {
            defense: 10 + equipment_armor + defence + prayer.max(0) / 2,
            initiative: ranged + initiative_bonus + self.level / 3,
            crit_chance: 5 + crit_bonus + attack.max(0) * 2 + ranged.max(0),
            dodge: ranged * 2 + self.level / 2,
            spell_power: magic * 2 + spell_power_bonus + self.level,
            healing_power: prayer * 2 + magic.max(0) + self.level / 2,
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
        let hp_gain =
            8 * levels_gained + self.stats.modifier(MajorSkill::Constitution).max(1) * levels_gained;
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
        Class::Warrior => vec![
            (1, "guard_stance"),
            (2, "cleaving_blow"),
            (4, "shield_bash"),
        ],
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

#[cfg(test)]
mod tests {
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
}
