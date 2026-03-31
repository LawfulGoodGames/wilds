use crate::character::{Class, MajorSkill, ResistanceProfile, ResourcePool, SavedCharacter};
use crate::inventory::{
    AttackOption, Equipment, InventoryItem, ItemEffect, LootTableEntry, WeaponKind, find_def,
};
use rand::{Rng, RngExt};

mod content;
mod enemy;
mod player;
mod systems;

use content::enemy_def_by_name;
pub use content::{ability_def, encounter_def, enemy_def};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
    Mana,
    Stamina,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Physical,
    Fire,
    Frost,
    Lightning,
    Poison,
    Holy,
    Shadow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbilityTarget {
    Enemy,
    SelfTarget,
    Ally,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Poison,
    Burn,
    Bleed,
    Stun,
    Guard,
    Regen,
    Weakness,
}

impl StatusKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Poison => "Poison",
            Self::Burn => "Burn",
            Self::Bleed => "Bleed",
            Self::Stun => "Stun",
            Self::Guard => "Guard",
            Self::Regen => "Regen",
            Self::Weakness => "Weakness",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusTiming {
    OnApply,
    TurnStart,
    TurnEnd,
    OnHit,
    OnReceiveHit,
    Expire,
}

#[derive(Debug, Clone)]
pub struct StatusEffect {
    pub kind: StatusKind,
    pub duration: i32,
    pub potency: i32,
    pub stacks: i32,
    pub source_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyRole {
    Brute,
    Skirmisher,
    Caster,
    Support,
}

#[derive(Debug, Clone)]
pub struct AbilityDef {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub resource_kind: Option<ResourceKind>,
    pub cost: i32,
    pub cooldown: i32,
    pub target: AbilityTarget,
    pub accuracy_bonus: i32,
    pub damage_min: i32,
    pub damage_max: i32,
    pub damage_type: DamageType,
    pub scaling_stat: MajorSkill,
    pub apply_status: Option<(StatusKind, i32, i32)>,
    pub self_status: Option<(StatusKind, i32, i32)>,
    pub heal_amount: i32,
}

#[derive(Debug, Clone)]
pub struct EnemyDef {
    pub id: &'static str,
    pub name: &'static str,
    pub family: &'static str,
    pub role: EnemyRole,
    pub level: i32,
    pub hp: i32,
    pub mana: i32,
    pub stamina: i32,
    pub attack_bonus: i32,
    pub defense: i32,
    pub initiative: i32,
    pub damage_min: i32,
    pub damage_max: i32,
    pub weapon_kind: WeaponKind,
    pub ability_ids: &'static [&'static str],
    pub loot: &'static [LootTableEntry],
    pub reward_xp: i32,
    pub reward_gold: i32,
}

#[derive(Debug, Clone)]
pub struct EncounterDef {
    pub id: &'static str,
    pub name: &'static str,
    pub environment_tags: &'static [&'static str],
    pub enemies: &'static [&'static str],
}

#[derive(Debug, Clone)]
pub enum CombatLogEvent {
    TurnStart {
        actor: String,
    },
    AttackResolved {
        actor: String,
        target: String,
        hit: bool,
        amount: i32,
        detail: String,
    },
    AbilityUsed {
        actor: String,
        ability: String,
        detail: String,
    },
    StatusApplied {
        actor: String,
        target: String,
        status: StatusKind,
        duration: i32,
    },
    StatusExpired {
        actor: String,
        status: StatusKind,
    },
    ResourceChanged {
        actor: String,
        label: &'static str,
        amount: i32,
    },
    LootGained {
        item_name: String,
        qty: i32,
    },
    QuestCredit {
        detail: String,
    },
    Info(String),
}

impl CombatLogEvent {
    pub fn to_line(&self) -> String {
        match self {
            Self::TurnStart { actor } => format!("{actor} takes the field."),
            Self::AttackResolved {
                actor,
                target,
                hit,
                amount,
                detail,
            } => {
                if *hit {
                    format!("{actor} hits {target} for {amount}. {detail}")
                } else {
                    format!("{actor} misses {target}. {detail}")
                }
            }
            Self::AbilityUsed {
                actor,
                ability,
                detail,
            } => format!("{actor} uses {ability}. {detail}"),
            Self::StatusApplied {
                actor,
                target,
                status,
                duration,
            } => {
                format!(
                    "{actor} inflicts {} on {target} for {duration} turns.",
                    status.label()
                )
            }
            Self::StatusExpired { actor, status } => {
                format!("{} fades from {actor}.", status.label())
            }
            Self::ResourceChanged {
                actor,
                label,
                amount,
            } => format!("{actor} restores {amount} {label}."),
            Self::LootGained { item_name, qty } => format!("Loot gained: {item_name} x{qty}."),
            Self::QuestCredit { detail } => detail.clone(),
            Self::Info(text) => text.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CombatantSnapshot {
    pub name: String,
    pub family: String,
    pub is_player: bool,
    pub class: Option<Class>,
    pub resources: ResourcePool,
    pub defense: i32,
    pub initiative: i32,
    pub attack_bonus: i32,
    pub ranged_attack_bonus: i32,
    pub magic_attack_bonus: i32,
    pub prayer_attack_bonus: i32,
    pub spell_power: i32,
    pub healing_power: i32,
    pub strength_bonus: i32,
    pub crit_chance: i32,
    pub dodge: i32,
    pub resistances: ResistanceProfile,
    pub weapon_kind: Option<WeaponKind>,
    pub weapon_attacks: Vec<AttackOption>,
    pub ability_ids: Vec<String>,
    pub statuses: Vec<StatusEffect>,
    pub role: Option<EnemyRole>,
    pub cooldowns: Vec<(String, i32)>,
}

impl CombatantSnapshot {
    fn is_alive(&self) -> bool {
        self.resources.hp > 0
    }

    fn has_status(&self, kind: StatusKind) -> bool {
        self.statuses.iter().any(|status| status.kind == kind)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnRef {
    Player,
    Enemy(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionTab {
    Weapon,
    Ability,
    Item,
}

impl ActionTab {
    pub fn label(self) -> &'static str {
        match self {
            Self::Weapon => "Weapon",
            Self::Ability => "Ability",
            Self::Item => "Item",
        }
    }
}

#[derive(Debug, Clone)]
pub enum PlayerAction {
    UseWeapon,
    UseAbility,
    UseItem,
    Defend,
    Flee,
}

#[derive(Debug, Clone)]
pub struct CombatReward {
    pub xp: i32,
    pub gold: i32,
    pub encounter_name: String,
    pub environment_tags: Vec<String>,
    pub drops: Vec<(String, i32)>,
    pub defeated_families: Vec<String>,
    pub enemies_defeated: i32,
    pub beast_kills: i32,
    pub bandit_kills: i32,
    pub undead_kills: i32,
    pub damage_dealt: i32,
    pub ability_uses: i32,
    pub weapon_attacks: i32,
    pub item_uses: i32,
}

#[derive(Debug, Clone)]
pub enum CombatOutcome {
    Ongoing,
    Won(CombatReward),
    Lost,
    Fled,
}

#[derive(Debug, Clone)]
pub struct CombatState {
    pub encounter_name: String,
    pub environment_tags: Vec<String>,
    pub player: CombatantSnapshot,
    pub enemies: Vec<CombatantSnapshot>,
    pub initiative: Vec<TurnRef>,
    pub turn_index: usize,
    pub selected_target: usize,
    pub action_tab: ActionTab,
    pub selected_weapon_attack: usize,
    pub selected_ability: usize,
    pub selected_item: usize,
    pub consumables: Vec<InventoryItem>,
    pub free_item_used: bool,
    pub log: Vec<CombatLogEvent>,
    pub new_log_entries: usize,
    pub last_roll_summary: Option<String>,
    pub player_damage_dealt: i32,
    pub player_ability_uses: i32,
    pub player_weapon_attacks: i32,
    pub player_item_uses: i32,
}

impl CombatState {
    pub fn from_character_and_encounter(
        character: &SavedCharacter,
        equipment: &Equipment,
        inventory: &[InventoryItem],
        encounter_id: &str,
    ) -> Self {
        let encounter = encounter_def(encounter_id);
        let eq_stats = equipment.total_equipment_stats();
        let derived = character.derived_stats(
            eq_stats.armor,
            eq_stats.attack_bonus,
            eq_stats.spell_power,
            eq_stats.crit_bonus,
            eq_stats.initiative_bonus,
        );
        let mut known_abilities = character
            .known_abilities
            .iter()
            .filter(|ability| ability.unlocked)
            .map(|ability| ability.ability_id.clone())
            .collect::<Vec<_>>();
        known_abilities.sort();

        let player = CombatantSnapshot {
            name: character.name.clone(),
            family: "Hero".to_string(),
            is_player: true,
            class: Some(character.class),
            resources: character.resources,
            defense: derived.defense,
            initiative: derived.initiative,
            attack_bonus: derived.melee_accuracy,
            ranged_attack_bonus: derived.ranged_accuracy,
            magic_attack_bonus: derived.magic_accuracy,
            prayer_attack_bonus: derived.prayer_accuracy,
            spell_power: derived.spell_power,
            healing_power: derived.healing_power,
            strength_bonus: character.stats.modifier(MajorSkill::Strength),
            crit_chance: derived.crit_chance,
            dodge: derived.dodge,
            resistances: eq_stats.resistances,
            weapon_kind: equipment.weapon_kind(),
            weapon_attacks: equipment.attack_options(),
            ability_ids: known_abilities,
            statuses: vec![],
            role: None,
            cooldowns: character
                .known_abilities
                .iter()
                .map(|ability| (ability.ability_id.clone(), ability.cooldown_remaining))
                .collect(),
        };

        let enemies = encounter
            .enemies
            .iter()
            .map(|enemy_id| {
                let def = enemy_def(enemy_id);
                CombatantSnapshot {
                    name: def.name.to_string(),
                    family: def.family.to_string(),
                    is_player: false,
                    class: None,
                    resources: ResourcePool::full(def.hp, def.mana, def.stamina),
                    defense: def.defense,
                    initiative: def.initiative,
                    attack_bonus: def.attack_bonus,
                    ranged_attack_bonus: def.attack_bonus,
                    magic_attack_bonus: def.attack_bonus,
                    prayer_attack_bonus: def.attack_bonus,
                    spell_power: def.level + if def.role == EnemyRole::Caster { 4 } else { 0 },
                    healing_power: def.level,
                    strength_bonus: def.level / 2,
                    crit_chance: 3 + def.level,
                    dodge: 2 + def.level,
                    resistances: ResistanceProfile::default(),
                    weapon_kind: Some(def.weapon_kind),
                    weapon_attacks: vec![AttackOption {
                        name: "Attack",
                        accuracy_bonus: def.attack_bonus,
                        min_damage: def.damage_min,
                        max_damage: def.damage_max,
                    }],
                    ability_ids: def.ability_ids.iter().map(|id| id.to_string()).collect(),
                    statuses: vec![],
                    role: Some(def.role),
                    cooldowns: def
                        .ability_ids
                        .iter()
                        .map(|id| (id.to_string(), 0))
                        .collect(),
                }
            })
            .collect::<Vec<_>>();

        let mut initiative = vec![TurnRef::Player];
        for idx in 0..enemies.len() {
            initiative.push(TurnRef::Enemy(idx));
        }
        initiative.sort_by_key(|turn_ref| {
            std::cmp::Reverse(match turn_ref {
                TurnRef::Player => player.initiative,
                TurnRef::Enemy(idx) => enemies[*idx].initiative,
            })
        });

        let consumables = inventory
            .iter()
            .filter(|item| {
                item.def()
                    .map(|def| {
                        def.kind == crate::inventory::ItemKind::Consumable && item.quantity > 0
                    })
                    .unwrap_or(false)
            })
            .cloned()
            .collect::<Vec<_>>();

        Self {
            encounter_name: encounter.name.to_string(),
            environment_tags: encounter
                .environment_tags
                .iter()
                .map(|tag| (*tag).to_string())
                .collect(),
            player,
            enemies,
            initiative,
            turn_index: 0,
            selected_target: 0,
            action_tab: ActionTab::Weapon,
            selected_weapon_attack: 0,
            selected_ability: 0,
            selected_item: 0,
            consumables,
            free_item_used: false,
            log: vec![CombatLogEvent::Info(format!(
                "{} begins amidst {}.",
                encounter.name,
                encounter.environment_tags.join(", ")
            ))],
            new_log_entries: 1,
            last_roll_summary: None,
            player_damage_dealt: 0,
            player_ability_uses: 0,
            player_weapon_attacks: 0,
            player_item_uses: 0,
        }
    }

    pub fn selected_target_name(&self) -> String {
        self.enemies
            .get(self.selected_target)
            .map(|enemy| enemy.name.clone())
            .unwrap_or_else(|| "None".to_string())
    }

    pub fn current_turn(&self) -> TurnRef {
        self.initiative
            .get(self.turn_index)
            .copied()
            .unwrap_or(TurnRef::Player)
    }

    pub fn selected_weapon_attack(&self) -> Option<AttackOption> {
        self.player
            .weapon_attacks
            .get(self.selected_weapon_attack)
            .copied()
    }

    pub fn selected_ability_def(&self) -> Option<&'static AbilityDef> {
        self.player
            .ability_ids
            .get(self.selected_ability)
            .and_then(|id| ability_def(id))
    }

    pub fn selected_item(&self) -> Option<&InventoryItem> {
        self.consumables.get(self.selected_item)
    }

    pub fn cycle_target(&mut self, dir: i32) {
        let alive = self
            .enemies
            .iter()
            .enumerate()
            .filter(|(_, enemy)| enemy.is_alive())
            .map(|(idx, _)| idx)
            .collect::<Vec<_>>();
        if alive.is_empty() {
            return;
        }
        let current_pos = alive
            .iter()
            .position(|idx| *idx == self.selected_target)
            .unwrap_or(0);
        let len = alive.len() as i32;
        let next = (current_pos as i32 + dir).rem_euclid(len) as usize;
        self.selected_target = alive[next];
    }

    fn auto_select_alive_target(&mut self) {
        if self
            .enemies
            .get(self.selected_target)
            .map(|enemy| enemy.is_alive())
            .unwrap_or(false)
        {
            return;
        }
        if let Some((idx, _)) = self
            .enemies
            .iter()
            .enumerate()
            .find(|(_, enemy)| enemy.is_alive())
        {
            self.selected_target = idx;
        }
    }

    pub fn cycle_selection(&mut self, dir: i32) {
        let len = match self.action_tab {
            ActionTab::Weapon => self.player.weapon_attacks.len(),
            ActionTab::Ability => self.player.ability_ids.len(),
            ActionTab::Item => self.consumables.len(),
        };
        if len == 0 {
            return;
        }
        let cursor = match self.action_tab {
            ActionTab::Weapon => &mut self.selected_weapon_attack,
            ActionTab::Ability => &mut self.selected_ability,
            ActionTab::Item => &mut self.selected_item,
        };
        *cursor = ((*cursor as i32 + dir).rem_euclid(len as i32)) as usize;
    }

    pub fn set_tab(&mut self, tab: ActionTab) {
        self.action_tab = tab;
    }

    pub fn cycle_tab(&mut self, dir: i32) {
        let tabs = [ActionTab::Weapon, ActionTab::Ability, ActionTab::Item];
        let current = tabs
            .iter()
            .position(|tab| *tab == self.action_tab)
            .unwrap_or(0);
        let next = (current as i32 + dir).rem_euclid(tabs.len() as i32) as usize;
        self.action_tab = tabs[next];
    }
}
