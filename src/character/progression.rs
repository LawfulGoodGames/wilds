use super::catalog::{CharacterClassProgression, Class, Race};
use super::proficiencies::{
    MajorProficiencyData, MajorSkill, ProficiencyData, Stats, level_from_xp,
    proficiency_level_from_xp,
};

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
    pub melee_accuracy: i32,
    pub ranged_accuracy: i32,
    pub magic_accuracy: i32,
    pub prayer_accuracy: i32,
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
    pub major_proficiencies: Vec<MajorProficiencyData>,
    pub resources: ResourcePool,
    pub proficiencies: Vec<ProficiencyData>,
    pub known_abilities: Vec<KnownAbility>,
}

impl SavedCharacter {
    pub fn major_skill(&self, kind: MajorSkill) -> i32 {
        self.major_proficiencies
            .iter()
            .find(|skill| skill.kind == kind)
            .map(|skill| skill.level() as i32)
            .unwrap_or_else(|| panic!("Missing major proficiency data for {}", kind.full_name()))
    }

    pub fn major_skill_xp(&self, kind: MajorSkill) -> i32 {
        self.major_proficiencies
            .iter()
            .find(|skill| skill.kind == kind)
            .map(|skill| skill.xp)
            .unwrap_or_else(|| panic!("Missing major proficiency data for {}", kind.full_name()))
    }

    pub fn major_skill_progress(&self, kind: MajorSkill) -> f64 {
        self.major_proficiencies
            .iter()
            .find(|skill| skill.kind == kind)
            .map(|skill| skill.progress())
            .unwrap_or_else(|| panic!("Missing major proficiency data for {}", kind.full_name()))
    }

    pub fn major_skill_xp_to_next(&self, kind: MajorSkill) -> u32 {
        self.major_proficiencies
            .iter()
            .find(|skill| skill.kind == kind)
            .map(|skill| skill.xp_to_next())
            .unwrap_or_else(|| panic!("Missing major proficiency data for {}", kind.full_name()))
    }

    pub fn set_major_skill_xp(&mut self, kind: MajorSkill, xp: i32) {
        if let Some(skill) = self
            .major_proficiencies
            .iter_mut()
            .find(|skill| skill.kind == kind)
        {
            skill.xp = xp.max(0);
        } else {
            self.major_proficiencies.push(MajorProficiencyData {
                kind,
                xp: xp.max(0),
            });
        }
        self.sync_stats_from_major_proficiencies();
    }

    pub fn sync_stats_from_major_proficiencies(&mut self) {
        for skill in MajorSkill::ALL {
            let level = self
                .major_proficiencies
                .iter()
                .find(|entry| entry.kind == skill)
                .map(|entry| proficiency_level_from_xp(entry.xp) as i32)
                .unwrap_or_else(|| {
                    panic!("Missing major proficiency data for {}", skill.full_name())
                });
            match skill {
                MajorSkill::Strength => self.stats.strength = level,
                MajorSkill::Dexterity => self.stats.dexterity = level,
                MajorSkill::Constitution => self.stats.constitution = level,
                MajorSkill::Intelligence => self.stats.intelligence = level,
                MajorSkill::Wisdom => self.stats.wisdom = level,
                MajorSkill::Charisma => self.stats.charisma = level,
            }
        }
    }

    pub fn derived_stats(
        &self,
        equipment_armor: i32,
        attack_bonus: i32,
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
            melee_accuracy: attack + attack_bonus + self.level / 3,
            ranged_accuracy: ranged + attack_bonus + self.level / 3,
            magic_accuracy: magic + attack_bonus + spell_power_bonus / 2 + self.level / 3,
            prayer_accuracy: prayer + attack_bonus + self.level / 3,
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
        let hp_gain = 8 * levels_gained
            + self.stats.modifier(MajorSkill::Constitution).max(1) * levels_gained;
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
