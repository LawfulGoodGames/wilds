use crate::character::{Class, MajorSkill, ResourcePool, SavedCharacter};
use crate::inventory::{
    AttackOption, Equipment, InventoryItem, ItemEffect, ItemRarity, LootTableEntry, WeaponKind,
    find_def,
};
use rand::{Rng, RngExt};

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
    pub spell_power: i32,
    pub crit_chance: i32,
    pub dodge: i32,
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
    pub drops: Vec<(String, i32)>,
    pub defeated_families: Vec<String>,
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
            attack_bonus: eq_stats.attack_bonus
                + character
                    .stats
                    .modifier(MajorSkill::Strength)
                    .max(character.stats.modifier(MajorSkill::Dexterity)),
            spell_power: derived.spell_power,
            crit_chance: derived.crit_chance,
            dodge: derived.dodge,
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
                    spell_power: def.level + if def.role == EnemyRole::Caster { 4 } else { 0 },
                    crit_chance: 3 + def.level,
                    dodge: 2 + def.level,
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

    pub fn resolve_player_action(&mut self, action: PlayerAction) -> CombatOutcome {
        if self.current_turn() != TurnRef::Player || !self.player.is_alive() {
            return CombatOutcome::Ongoing;
        }

        let start_len = self.log.len();
        self.tick_statuses_for_player(StatusTiming::TurnStart);
        if self.player.has_status(StatusKind::Stun) {
            self.log.push(CombatLogEvent::Info(
                "You are stunned and lose the turn.".to_string(),
            ));
            return self.finish_player_phase(start_len);
        }

        let mut rng = rand::rng();
        match action {
            PlayerAction::UseWeapon => self.resolve_player_weapon(&mut rng),
            PlayerAction::UseAbility => self.resolve_player_ability(&mut rng),
            PlayerAction::UseItem => self.resolve_player_item(),
            PlayerAction::Defend => {
                self.apply_status_to_player(StatusKind::Guard, 1, 4, "Defend");
                self.log.push(CombatLogEvent::Info(
                    "You brace for the next strike.".to_string(),
                ));
            }
            PlayerAction::Flee => {
                let roll = rng.random_range(1..=20);
                let dc = 11 + self.enemies.iter().filter(|enemy| enemy.is_alive()).count() as i32;
                let total = roll + self.player.initiative;
                self.last_roll_summary =
                    Some(format!("Flee d20={} total={} vs DC {}", roll, total, dc));
                if total >= dc {
                    self.log.push(CombatLogEvent::Info(
                        "You break away from the encounter.".to_string(),
                    ));
                    self.update_new_entries(start_len);
                    return CombatOutcome::Fled;
                }
                self.log
                    .push(CombatLogEvent::Info("You fail to disengage.".to_string()));
            }
        }

        self.tick_statuses_for_player(StatusTiming::TurnEnd);
        self.finish_player_phase(start_len)
    }

    pub fn begin_encounter(&mut self) -> CombatOutcome {
        if self.current_turn() == TurnRef::Player {
            CombatOutcome::Ongoing
        } else {
            self.resolve_enemy_round()
        }
    }

    fn finish_player_phase(&mut self, start_len: usize) -> CombatOutcome {
        self.free_item_used = false;
        self.advance_turn();
        let outcome = self.resolve_enemy_round();
        self.trim_log();
        self.update_new_entries(start_len);
        outcome
    }

    fn resolve_player_weapon(&mut self, rng: &mut impl Rng) {
        let Some(attack) = self.selected_weapon_attack() else {
            self.log.push(CombatLogEvent::Info(
                "No weapon attack is available.".to_string(),
            ));
            return;
        };
        self.resolve_attack(
            true,
            None,
            attack.name,
            attack.accuracy_bonus,
            attack.min_damage,
            attack.max_damage,
            DamageType::Physical,
            None,
            None,
            rng,
        );
    }

    fn resolve_player_ability(&mut self, rng: &mut impl Rng) {
        let Some(ability) = self.selected_ability_def() else {
            self.log
                .push(CombatLogEvent::Info("No ability is selected.".to_string()));
            return;
        };
        if self.cooldown_for_player(ability.id) > 0 {
            self.log.push(CombatLogEvent::Info(format!(
                "{} is still on cooldown.",
                ability.name
            )));
            return;
        }
        if !self.can_pay_cost(ability.resource_kind, ability.cost) {
            self.log.push(CombatLogEvent::Info(format!(
                "Not enough resource for {}.",
                ability.name
            )));
            return;
        }
        self.pay_cost(ability.resource_kind, ability.cost);
        self.set_player_cooldown(ability.id, ability.cooldown);
        self.log.push(CombatLogEvent::AbilityUsed {
            actor: self.player.name.clone(),
            ability: ability.name.to_string(),
            detail: ability.description.to_string(),
        });
        if ability.target == AbilityTarget::SelfTarget {
            if ability.heal_amount > 0 {
                self.player.resources.hp = (self.player.resources.hp + ability.heal_amount)
                    .min(self.player.resources.max_hp);
                self.log.push(CombatLogEvent::ResourceChanged {
                    actor: self.player.name.clone(),
                    label: "HP",
                    amount: ability.heal_amount,
                });
            }
            if let Some((status, duration, potency)) = ability.self_status {
                self.apply_status_to_player(status, duration, potency, ability.name);
            }
            return;
        }

        self.resolve_attack(
            true,
            Some(ability.name),
            ability.name,
            ability.accuracy_bonus,
            ability.damage_min,
            ability.damage_max,
            ability.damage_type,
            ability.apply_status,
            ability.self_status,
            rng,
        );
    }

    fn resolve_player_item(&mut self) {
        if self.free_item_used {
            self.log.push(CombatLogEvent::Info(
                "You already used a free item this turn.".to_string(),
            ));
            return;
        }
        let Some(item) = self.selected_item().cloned() else {
            self.log.push(CombatLogEvent::Info(
                "No usable item is selected.".to_string(),
            ));
            return;
        };
        let Some(def) = item.def() else {
            return;
        };
        if item.quantity <= 0 {
            self.log.push(CombatLogEvent::Info(format!(
                "You are out of {}.",
                def.name
            )));
            return;
        }
        for effect in def.effects {
            self.apply_item_effect(*effect, def.name);
        }
        if let Some(slot) = self.consumables.get_mut(self.selected_item) {
            slot.quantity -= 1;
        }
        self.consumables.retain(|it| it.quantity > 0);
        self.selected_item = self
            .selected_item
            .min(self.consumables.len().saturating_sub(1));
        self.free_item_used = true;
    }

    fn apply_item_effect(&mut self, effect: ItemEffect, item_name: &str) {
        match effect {
            ItemEffect::HealHp(amount) => {
                self.player.resources.hp =
                    (self.player.resources.hp + amount).min(self.player.resources.max_hp);
                self.log.push(CombatLogEvent::ResourceChanged {
                    actor: self.player.name.clone(),
                    label: "HP",
                    amount,
                });
            }
            ItemEffect::RestoreMana(amount) => {
                self.player.resources.mana =
                    (self.player.resources.mana + amount).min(self.player.resources.max_mana);
                self.log.push(CombatLogEvent::ResourceChanged {
                    actor: self.player.name.clone(),
                    label: "Mana",
                    amount,
                });
            }
            ItemEffect::RestoreStamina(amount) => {
                self.player.resources.stamina =
                    (self.player.resources.stamina + amount).min(self.player.resources.max_stamina);
                self.log.push(CombatLogEvent::ResourceChanged {
                    actor: self.player.name.clone(),
                    label: "Stamina",
                    amount,
                });
            }
            ItemEffect::CurePoison => {
                self.player
                    .statuses
                    .retain(|status| status.kind != StatusKind::Poison);
                self.log
                    .push(CombatLogEvent::Info(format!("{item_name} clears poison.")));
            }
            ItemEffect::ApplyGuard(amount) => {
                self.apply_status_to_player(StatusKind::Guard, 1, amount, item_name);
            }
        }
    }

    fn resolve_enemy_round(&mut self) -> CombatOutcome {
        loop {
            if self.enemies.iter().all(|enemy| !enemy.is_alive()) {
                return CombatOutcome::Won(self.collect_rewards());
            }
            if !self.player.is_alive() {
                return CombatOutcome::Lost;
            }
            match self.current_turn() {
                TurnRef::Player => return CombatOutcome::Ongoing,
                TurnRef::Enemy(idx) => {
                    if idx >= self.enemies.len() || !self.enemies[idx].is_alive() {
                        self.advance_turn();
                        continue;
                    }
                    self.tick_statuses_for_enemy(idx, StatusTiming::TurnStart);
                    if !self.enemies[idx].is_alive() {
                        self.advance_turn();
                        continue;
                    }
                    if self.enemies[idx].has_status(StatusKind::Stun) {
                        let name = self.enemies[idx].name.clone();
                        self.log.push(CombatLogEvent::Info(format!(
                            "{name} is stunned and loses the turn."
                        )));
                        self.tick_statuses_for_enemy(idx, StatusTiming::TurnEnd);
                        self.advance_turn();
                        continue;
                    }
                    let mut rng = rand::rng();
                    self.resolve_enemy_action(idx, &mut rng);
                    self.tick_statuses_for_enemy(idx, StatusTiming::TurnEnd);
                    self.advance_turn();
                }
            }
        }
    }

    fn resolve_enemy_action(&mut self, idx: usize, rng: &mut impl Rng) {
        let enemy = self.enemies[idx].clone();
        let chosen_ability = self.choose_enemy_ability(idx);
        if let Some(ability) = chosen_ability {
            self.use_enemy_ability(idx, ability, rng);
        } else {
            let attack = enemy
                .weapon_attacks
                .first()
                .copied()
                .unwrap_or(AttackOption {
                    name: "Strike",
                    accuracy_bonus: enemy.attack_bonus,
                    min_damage: 4,
                    max_damage: 8,
                });
            self.resolve_enemy_attack(idx, attack, rng);
        }
    }

    fn choose_enemy_ability(&self, idx: usize) -> Option<&'static AbilityDef> {
        let enemy = self.enemies.get(idx)?;
        for ability_id in &enemy.ability_ids {
            let ability = ability_def(ability_id)?;
            if self.cooldown_for_enemy(idx, ability.id) > 0 {
                continue;
            }
            match enemy.role.unwrap_or(EnemyRole::Brute) {
                EnemyRole::Support
                    if ability.heal_amount > 0
                        && enemy.resources.hp < enemy.resources.max_hp / 2 =>
                {
                    return Some(ability);
                }
                EnemyRole::Caster if ability.damage_type != DamageType::Physical => {
                    return Some(ability);
                }
                EnemyRole::Skirmisher if ability.apply_status.is_some() => return Some(ability),
                EnemyRole::Brute if ability.damage_min >= 7 => return Some(ability),
                _ => {}
            }
        }
        None
    }

    fn use_enemy_ability(&mut self, idx: usize, ability: &'static AbilityDef, rng: &mut impl Rng) {
        if !self.enemy_can_pay_cost(idx, ability.resource_kind, ability.cost) {
            let attack =
                self.enemies[idx]
                    .weapon_attacks
                    .first()
                    .copied()
                    .unwrap_or(AttackOption {
                        name: "Strike",
                        accuracy_bonus: self.enemies[idx].attack_bonus,
                        min_damage: 4,
                        max_damage: 8,
                    });
            self.resolve_enemy_attack(idx, attack, rng);
            return;
        }
        self.pay_enemy_cost(idx, ability.resource_kind, ability.cost);
        self.set_enemy_cooldown(idx, ability.id, ability.cooldown);

        let name = self.enemies[idx].name.clone();
        self.log.push(CombatLogEvent::AbilityUsed {
            actor: name.clone(),
            ability: ability.name.to_string(),
            detail: ability.description.to_string(),
        });

        if ability.target == AbilityTarget::SelfTarget {
            if ability.heal_amount > 0 {
                self.enemies[idx].resources.hp = (self.enemies[idx].resources.hp
                    + ability.heal_amount)
                    .min(self.enemies[idx].resources.max_hp);
            }
            if let Some((status, duration, potency)) = ability.self_status {
                self.apply_status_to_enemy(idx, status, duration, potency, ability.name);
            }
            return;
        }

        let target_defense = self.player.defense
            - if self.player.has_status(StatusKind::Weakness) {
                1
            } else {
                0
            };
        let roll = rng.random_range(1..=20);
        let total = roll + self.enemies[idx].attack_bonus + ability.accuracy_bonus;
        let hit = roll != 1 && (roll == 20 || total >= target_defense + self.player_guard_bonus());
        let base_damage = rng.random_range(ability.damage_min..=ability.damage_max)
            + self.enemies[idx].spell_power / 3
            - self.player_guard_bonus();
        let damage = base_damage.max(0);
        self.last_roll_summary = Some(format!(
            "{} used {}: d20={} total={} vs DEF {}",
            name,
            ability.name,
            roll,
            total,
            target_defense + self.player_guard_bonus()
        ));
        if hit {
            self.player.resources.hp = (self.player.resources.hp - damage).max(0);
            if let Some((status, duration, potency)) = ability.apply_status {
                self.apply_status_to_player(status, duration, potency, ability.name);
            }
        }
        self.log.push(CombatLogEvent::AttackResolved {
            actor: name,
            target: self.player.name.clone(),
            hit,
            amount: if hit { damage } else { 0 },
            detail: ability.damage_type_label().to_string(),
        });
    }

    fn resolve_enemy_attack(&mut self, idx: usize, attack: AttackOption, rng: &mut impl Rng) {
        let target_defense = self.player.defense + self.player_guard_bonus();
        let roll = rng.random_range(1..=20);
        let total = roll + self.enemies[idx].attack_bonus + attack.accuracy_bonus;
        let hit = roll != 1 && (roll == 20 || total >= target_defense);
        let damage = if hit {
            (rng.random_range(attack.min_damage..=attack.max_damage) - self.player_guard_bonus())
                .max(0)
        } else {
            0
        };
        if hit {
            self.player.resources.hp = (self.player.resources.hp - damage).max(0);
        }
        let name = self.enemies[idx].name.clone();
        self.last_roll_summary = Some(format!(
            "{name} d20={roll} total={total} vs DEF {target_defense}"
        ));
        self.log.push(CombatLogEvent::AttackResolved {
            actor: name,
            target: self.player.name.clone(),
            hit,
            amount: damage,
            detail: attack.name.to_string(),
        });
    }

    fn resolve_attack(
        &mut self,
        player_is_actor: bool,
        ability_name: Option<&str>,
        label: &str,
        accuracy_bonus: i32,
        min_damage: i32,
        max_damage: i32,
        damage_type: DamageType,
        apply_status: Option<(StatusKind, i32, i32)>,
        self_status: Option<(StatusKind, i32, i32)>,
        rng: &mut impl Rng,
    ) {
        let Some(target) = self.enemies.get(self.selected_target) else {
            self.log
                .push(CombatLogEvent::Info("No valid target.".to_string()));
            return;
        };
        if !target.is_alive() {
            self.log.push(CombatLogEvent::Info(
                "That target is already down.".to_string(),
            ));
            return;
        }
        let accuracy = self.player.attack_bonus
            + accuracy_bonus
            + if self.player.has_status(StatusKind::Weakness) {
                -2
            } else {
                0
            };
        let roll = rng.random_range(1..=20);
        let total = roll + accuracy;
        let defense = target.defense;
        let target_name = target.name.clone();
        let hit = roll != 1 && (roll == 20 || total >= defense);
        let crit = hit && rng.random_range(1..=100) <= self.player.crit_chance.max(0);
        let mut damage = if hit {
            rng.random_range(min_damage..=max_damage)
                + if damage_type == DamageType::Physical {
                    self.player.attack_bonus / 2
                } else {
                    self.player.spell_power / 3
                }
        } else {
            0
        };
        if crit {
            damage += damage / 2 + 2;
        }
        self.last_roll_summary = Some(format!(
            "{} d20={} total={} vs DEF {}{}",
            label,
            roll,
            total,
            defense,
            if crit { " CRIT" } else { "" }
        ));
        if hit {
            if let Some(target) = self.enemies.get_mut(self.selected_target) {
                target.resources.hp = (target.resources.hp - damage).max(0);
            }
            if let Some((status, duration, potency)) = apply_status {
                self.apply_status_to_enemy(self.selected_target, status, duration, potency, label);
            }
            if let Some((status, duration, potency)) = self_status {
                self.apply_status_to_player(status, duration, potency, label);
            }
            self.auto_select_alive_target();
        }
        self.log.push(CombatLogEvent::AttackResolved {
            actor: if player_is_actor {
                self.player.name.clone()
            } else {
                "Enemy".to_string()
            },
            target: target_name,
            hit,
            amount: damage,
            detail: ability_name.unwrap_or(label).to_string(),
        });
    }

    fn player_guard_bonus(&self) -> i32 {
        self.player
            .statuses
            .iter()
            .filter(|status| status.kind == StatusKind::Guard)
            .map(|status| status.potency)
            .sum()
    }

    fn tick_statuses_for_player(&mut self, timing: StatusTiming) {
        Self::tick_statuses(&mut self.player, timing, &mut self.log);
    }

    fn tick_statuses_for_enemy(&mut self, idx: usize, timing: StatusTiming) {
        if let Some(enemy) = self.enemies.get_mut(idx) {
            Self::tick_statuses(enemy, timing, &mut self.log);
        }
    }

    fn tick_statuses(
        target: &mut CombatantSnapshot,
        timing: StatusTiming,
        log: &mut Vec<CombatLogEvent>,
    ) {
        let mut expired = vec![];
        for (idx, status) in target.statuses.iter_mut().enumerate() {
            match (status.kind, timing) {
                (StatusKind::Poison, StatusTiming::TurnStart)
                | (StatusKind::Burn, StatusTiming::TurnStart)
                | (StatusKind::Bleed, StatusTiming::TurnStart) => {
                    target.resources.hp = (target.resources.hp - status.potency).max(0);
                    log.push(CombatLogEvent::AttackResolved {
                        actor: status.kind.label().to_string(),
                        target: target.name.clone(),
                        hit: true,
                        amount: status.potency,
                        detail: "damage over time".to_string(),
                    });
                }
                (StatusKind::Regen, StatusTiming::TurnStart) => {
                    target.resources.hp =
                        (target.resources.hp + status.potency).min(target.resources.max_hp);
                    log.push(CombatLogEvent::ResourceChanged {
                        actor: target.name.clone(),
                        label: "HP",
                        amount: status.potency,
                    });
                }
                _ => {}
            }
            if matches!(timing, StatusTiming::TurnEnd) {
                status.duration -= 1;
                if status.duration <= 0 {
                    expired.push(idx);
                }
            }
        }
        for idx in expired.into_iter().rev() {
            let status = target.statuses.remove(idx);
            log.push(CombatLogEvent::StatusExpired {
                actor: target.name.clone(),
                status: status.kind,
            });
        }
    }

    fn apply_status_to_player(
        &mut self,
        status: StatusKind,
        duration: i32,
        potency: i32,
        source_name: &str,
    ) {
        self.player.statuses.push(StatusEffect {
            kind: status,
            duration,
            potency,
            stacks: 1,
            source_name: source_name.to_string(),
        });
        self.log.push(CombatLogEvent::StatusApplied {
            actor: source_name.to_string(),
            target: self.player.name.clone(),
            status,
            duration,
        });
    }

    fn apply_status_to_enemy(
        &mut self,
        idx: usize,
        status: StatusKind,
        duration: i32,
        potency: i32,
        source_name: &str,
    ) {
        if let Some(enemy) = self.enemies.get_mut(idx) {
            enemy.statuses.push(StatusEffect {
                kind: status,
                duration,
                potency,
                stacks: 1,
                source_name: source_name.to_string(),
            });
            self.log.push(CombatLogEvent::StatusApplied {
                actor: source_name.to_string(),
                target: enemy.name.clone(),
                status,
                duration,
            });
        }
    }

    fn can_pay_cost(&self, resource_kind: Option<ResourceKind>, cost: i32) -> bool {
        match resource_kind {
            Some(ResourceKind::Mana) => self.player.resources.mana >= cost,
            Some(ResourceKind::Stamina) => self.player.resources.stamina >= cost,
            None => true,
        }
    }

    fn pay_cost(&mut self, resource_kind: Option<ResourceKind>, cost: i32) {
        match resource_kind {
            Some(ResourceKind::Mana) => {
                self.player.resources.mana = (self.player.resources.mana - cost).max(0)
            }
            Some(ResourceKind::Stamina) => {
                self.player.resources.stamina = (self.player.resources.stamina - cost).max(0)
            }
            None => {}
        }
    }

    fn enemy_can_pay_cost(
        &self,
        idx: usize,
        resource_kind: Option<ResourceKind>,
        cost: i32,
    ) -> bool {
        let Some(enemy) = self.enemies.get(idx) else {
            return false;
        };
        match resource_kind {
            Some(ResourceKind::Mana) => enemy.resources.mana >= cost,
            Some(ResourceKind::Stamina) => enemy.resources.stamina >= cost,
            None => true,
        }
    }

    fn pay_enemy_cost(&mut self, idx: usize, resource_kind: Option<ResourceKind>, cost: i32) {
        if let Some(enemy) = self.enemies.get_mut(idx) {
            match resource_kind {
                Some(ResourceKind::Mana) => {
                    enemy.resources.mana = (enemy.resources.mana - cost).max(0)
                }
                Some(ResourceKind::Stamina) => {
                    enemy.resources.stamina = (enemy.resources.stamina - cost).max(0)
                }
                None => {}
            }
        }
    }

    fn cooldown_for_player(&self, ability_id: &str) -> i32 {
        self.player
            .cooldowns
            .iter()
            .find(|(id, _)| id == ability_id)
            .map(|(_, value)| *value)
            .unwrap_or(0)
    }

    fn set_player_cooldown(&mut self, ability_id: &str, cooldown: i32) {
        if let Some(entry) = self
            .player
            .cooldowns
            .iter_mut()
            .find(|(id, _)| id == ability_id)
        {
            entry.1 = cooldown;
        }
    }

    fn cooldown_for_enemy(&self, idx: usize, ability_id: &str) -> i32 {
        self.enemies
            .get(idx)
            .and_then(|enemy| enemy.cooldowns.iter().find(|(id, _)| id == ability_id))
            .map(|(_, value)| *value)
            .unwrap_or(0)
    }

    fn set_enemy_cooldown(&mut self, idx: usize, ability_id: &str, cooldown: i32) {
        if let Some(entry) = self
            .enemies
            .get_mut(idx)
            .and_then(|enemy| enemy.cooldowns.iter_mut().find(|(id, _)| id == ability_id))
        {
            entry.1 = cooldown;
        }
    }

    fn advance_turn(&mut self) {
        for (_, cooldown) in &mut self.player.cooldowns {
            *cooldown = (*cooldown - 1).max(0);
        }
        for enemy in &mut self.enemies {
            for (_, cooldown) in &mut enemy.cooldowns {
                *cooldown = (*cooldown - 1).max(0);
            }
        }
        self.turn_index = (self.turn_index + 1) % self.initiative.len().max(1);
        while matches!(self.current_turn(), TurnRef::Enemy(idx) if self.enemies.get(idx).map(|enemy| !enemy.is_alive()).unwrap_or(true))
        {
            self.turn_index = (self.turn_index + 1) % self.initiative.len().max(1);
        }
    }

    fn collect_rewards(&mut self) -> CombatReward {
        let mut xp = 0;
        let mut gold = 0;
        let mut defeated_families = vec![];
        let mut drops = vec![];
        let mut rng = rand::rng();
        for enemy in &self.enemies {
            if !defeated_families.contains(&enemy.family) {
                defeated_families.push(enemy.family.clone());
            }
            let def = enemy_def_by_name(&enemy.name);
            xp += def.reward_xp;
            gold += def.reward_gold;
            for entry in def.loot {
                let roll = rng.random_range(1..=entry.weight.max(1));
                if roll == 1 {
                    let qty = if entry.max_qty > entry.min_qty {
                        rng.random_range(entry.min_qty..=entry.max_qty)
                    } else {
                        entry.min_qty
                    };
                    drops.push((entry.item_type.to_string(), qty));
                    if let Some(item_def) = find_def(entry.item_type) {
                        self.log.push(CombatLogEvent::LootGained {
                            item_name: item_def.name.to_string(),
                            qty,
                        });
                    }
                }
            }
        }
        CombatReward {
            xp,
            gold,
            drops,
            defeated_families,
        }
    }

    fn trim_log(&mut self) {
        if self.log.len() > 18 {
            let keep = self.log.len().saturating_sub(18);
            self.log.drain(0..keep);
        }
    }

    fn update_new_entries(&mut self, start_len: usize) {
        self.new_log_entries = self.log.len().saturating_sub(start_len);
    }
}

impl AbilityDef {
    fn damage_type_label(&self) -> &'static str {
        match self.damage_type {
            DamageType::Physical => "physical",
            DamageType::Fire => "fire",
            DamageType::Frost => "frost",
            DamageType::Lightning => "lightning",
            DamageType::Poison => "poison",
            DamageType::Holy => "holy",
            DamageType::Shadow => "shadow",
        }
    }
}

const WOLF_LOOT: &[LootTableEntry] = &[LootTableEntry {
    item_type: "wolf_pelt",
    min_qty: 1,
    max_qty: 2,
    weight: 2,
    min_rarity: ItemRarity::Common,
}];
const BANDIT_LOOT: &[LootTableEntry] = &[
    LootTableEntry {
        item_type: "bandit_seal",
        min_qty: 1,
        max_qty: 1,
        weight: 2,
        min_rarity: ItemRarity::Common,
    },
    LootTableEntry {
        item_type: "old_map",
        min_qty: 1,
        max_qty: 1,
        weight: 4,
        min_rarity: ItemRarity::Uncommon,
    },
];
const UNDEAD_LOOT: &[LootTableEntry] = &[LootTableEntry {
    item_type: "grave_ash",
    min_qty: 1,
    max_qty: 2,
    weight: 2,
    min_rarity: ItemRarity::Rare,
}];

pub const ABILITIES: &[AbilityDef] = &[
    AbilityDef {
        id: "guard_stance",
        name: "Guard Stance",
        description: "Raise guard and harden against the next assault.",
        resource_kind: Some(ResourceKind::Stamina),
        cost: 6,
        cooldown: 2,
        target: AbilityTarget::SelfTarget,
        accuracy_bonus: 0,
        damage_min: 0,
        damage_max: 0,
        damage_type: DamageType::Physical,
        scaling_stat: MajorSkill::Constitution,
        apply_status: None,
        self_status: Some((StatusKind::Guard, 2, 4)),
        heal_amount: 0,
    },
    AbilityDef {
        id: "cleaving_blow",
        name: "Cleaving Blow",
        description: "A punishing martial strike that can leave the foe weakened.",
        resource_kind: Some(ResourceKind::Stamina),
        cost: 8,
        cooldown: 2,
        target: AbilityTarget::Enemy,
        accuracy_bonus: 2,
        damage_min: 8,
        damage_max: 13,
        damage_type: DamageType::Physical,
        scaling_stat: MajorSkill::Strength,
        apply_status: Some((StatusKind::Weakness, 2, 2)),
        self_status: None,
        heal_amount: 0,
    },
    AbilityDef {
        id: "shield_bash",
        name: "Shield Bash",
        description: "Crash into the enemy and force a short stun.",
        resource_kind: Some(ResourceKind::Stamina),
        cost: 10,
        cooldown: 3,
        target: AbilityTarget::Enemy,
        accuracy_bonus: 1,
        damage_min: 6,
        damage_max: 10,
        damage_type: DamageType::Physical,
        scaling_stat: MajorSkill::Strength,
        apply_status: Some((StatusKind::Stun, 1, 1)),
        self_status: None,
        heal_amount: 0,
    },
    AbilityDef {
        id: "hunters_mark",
        name: "Hunter's Mark",
        description: "Mark a target to open them to follow-up damage.",
        resource_kind: Some(ResourceKind::Stamina),
        cost: 7,
        cooldown: 2,
        target: AbilityTarget::Enemy,
        accuracy_bonus: 3,
        damage_min: 5,
        damage_max: 8,
        damage_type: DamageType::Physical,
        scaling_stat: MajorSkill::Dexterity,
        apply_status: Some((StatusKind::Weakness, 2, 2)),
        self_status: None,
        heal_amount: 0,
    },
    AbilityDef {
        id: "volley",
        name: "Volley",
        description: "Loose a punishing shot with high opening pressure.",
        resource_kind: Some(ResourceKind::Stamina),
        cost: 9,
        cooldown: 2,
        target: AbilityTarget::Enemy,
        accuracy_bonus: 2,
        damage_min: 8,
        damage_max: 12,
        damage_type: DamageType::Physical,
        scaling_stat: MajorSkill::Dexterity,
        apply_status: None,
        self_status: None,
        heal_amount: 0,
    },
    AbilityDef {
        id: "crippling_shot",
        name: "Crippling Shot",
        description: "A precision strike that slows the enemy's response.",
        resource_kind: Some(ResourceKind::Stamina),
        cost: 10,
        cooldown: 3,
        target: AbilityTarget::Enemy,
        accuracy_bonus: 2,
        damage_min: 7,
        damage_max: 10,
        damage_type: DamageType::Physical,
        scaling_stat: MajorSkill::Dexterity,
        apply_status: Some((StatusKind::Weakness, 3, 2)),
        self_status: None,
        heal_amount: 0,
    },
    AbilityDef {
        id: "ember_burst",
        name: "Ember Burst",
        description: "A burst of heat that leaves the target burning.",
        resource_kind: Some(ResourceKind::Mana),
        cost: 8,
        cooldown: 1,
        target: AbilityTarget::Enemy,
        accuracy_bonus: 2,
        damage_min: 7,
        damage_max: 11,
        damage_type: DamageType::Fire,
        scaling_stat: MajorSkill::Intelligence,
        apply_status: Some((StatusKind::Burn, 2, 3)),
        self_status: None,
        heal_amount: 0,
    },
    AbilityDef {
        id: "frost_lance",
        name: "Frost Lance",
        description: "A cold spear of force that cuts and slows.",
        resource_kind: Some(ResourceKind::Mana),
        cost: 9,
        cooldown: 2,
        target: AbilityTarget::Enemy,
        accuracy_bonus: 2,
        damage_min: 8,
        damage_max: 12,
        damage_type: DamageType::Frost,
        scaling_stat: MajorSkill::Intelligence,
        apply_status: Some((StatusKind::Weakness, 2, 1)),
        self_status: None,
        heal_amount: 0,
    },
    AbilityDef {
        id: "storm_surge",
        name: "Storm Surge",
        description: "A violent discharge that overwhelms the target's defenses.",
        resource_kind: Some(ResourceKind::Mana),
        cost: 12,
        cooldown: 3,
        target: AbilityTarget::Enemy,
        accuracy_bonus: 3,
        damage_min: 10,
        damage_max: 14,
        damage_type: DamageType::Lightning,
        scaling_stat: MajorSkill::Intelligence,
        apply_status: None,
        self_status: None,
        heal_amount: 0,
    },
    AbilityDef {
        id: "dirty_cut",
        name: "Dirty Cut",
        description: "A jagged slice meant to keep the wound open.",
        resource_kind: Some(ResourceKind::Stamina),
        cost: 7,
        cooldown: 1,
        target: AbilityTarget::Enemy,
        accuracy_bonus: 3,
        damage_min: 5,
        damage_max: 9,
        damage_type: DamageType::Physical,
        scaling_stat: MajorSkill::Dexterity,
        apply_status: Some((StatusKind::Bleed, 2, 3)),
        self_status: None,
        heal_amount: 0,
    },
    AbilityDef {
        id: "evasion",
        name: "Evasion",
        description: "Slip into a guarded stance and recover breath.",
        resource_kind: Some(ResourceKind::Stamina),
        cost: 6,
        cooldown: 2,
        target: AbilityTarget::SelfTarget,
        accuracy_bonus: 0,
        damage_min: 0,
        damage_max: 0,
        damage_type: DamageType::Physical,
        scaling_stat: MajorSkill::Dexterity,
        apply_status: None,
        self_status: Some((StatusKind::Guard, 1, 5)),
        heal_amount: 0,
    },
    AbilityDef {
        id: "shadow_flurry",
        name: "Shadow Flurry",
        description: "A flurry of cuts delivered from broken tempo.",
        resource_kind: Some(ResourceKind::Stamina),
        cost: 11,
        cooldown: 3,
        target: AbilityTarget::Enemy,
        accuracy_bonus: 3,
        damage_min: 9,
        damage_max: 13,
        damage_type: DamageType::Shadow,
        scaling_stat: MajorSkill::Dexterity,
        apply_status: Some((StatusKind::Bleed, 3, 2)),
        self_status: None,
        heal_amount: 0,
    },
    AbilityDef {
        id: "radiant_slam",
        name: "Radiant Slam",
        description: "Drive holy force through steel and shield.",
        resource_kind: Some(ResourceKind::Stamina),
        cost: 8,
        cooldown: 2,
        target: AbilityTarget::Enemy,
        accuracy_bonus: 2,
        damage_min: 7,
        damage_max: 11,
        damage_type: DamageType::Holy,
        scaling_stat: MajorSkill::Strength,
        apply_status: None,
        self_status: Some((StatusKind::Guard, 1, 2)),
        heal_amount: 0,
    },
    AbilityDef {
        id: "vow_guard",
        name: "Vow Guard",
        description: "Swear the line will hold and reinforce your defense.",
        resource_kind: Some(ResourceKind::Mana),
        cost: 6,
        cooldown: 2,
        target: AbilityTarget::SelfTarget,
        accuracy_bonus: 0,
        damage_min: 0,
        damage_max: 0,
        damage_type: DamageType::Holy,
        scaling_stat: MajorSkill::Charisma,
        apply_status: None,
        self_status: Some((StatusKind::Guard, 2, 5)),
        heal_amount: 0,
    },
    AbilityDef {
        id: "lay_on_hands",
        name: "Lay on Hands",
        description: "Restore wounds with a surge of divine conviction.",
        resource_kind: Some(ResourceKind::Mana),
        cost: 10,
        cooldown: 3,
        target: AbilityTarget::SelfTarget,
        accuracy_bonus: 0,
        damage_min: 0,
        damage_max: 0,
        damage_type: DamageType::Holy,
        scaling_stat: MajorSkill::Charisma,
        apply_status: None,
        self_status: Some((StatusKind::Regen, 2, 4)),
        heal_amount: 18,
    },
    AbilityDef {
        id: "healing_prayer",
        name: "Healing Prayer",
        description: "A brief prayer that restores life and steadies the soul.",
        resource_kind: Some(ResourceKind::Mana),
        cost: 8,
        cooldown: 2,
        target: AbilityTarget::SelfTarget,
        accuracy_bonus: 0,
        damage_min: 0,
        damage_max: 0,
        damage_type: DamageType::Holy,
        scaling_stat: MajorSkill::Wisdom,
        apply_status: None,
        self_status: Some((StatusKind::Regen, 2, 5)),
        heal_amount: 16,
    },
    AbilityDef {
        id: "smite_undead",
        name: "Smite Undead",
        description: "Condemn unquiet dead with radiant force.",
        resource_kind: Some(ResourceKind::Mana),
        cost: 9,
        cooldown: 2,
        target: AbilityTarget::Enemy,
        accuracy_bonus: 3,
        damage_min: 8,
        damage_max: 12,
        damage_type: DamageType::Holy,
        scaling_stat: MajorSkill::Wisdom,
        apply_status: None,
        self_status: None,
        heal_amount: 0,
    },
    AbilityDef {
        id: "purge",
        name: "Purge",
        description: "Scour toxins and foul magic from the field.",
        resource_kind: Some(ResourceKind::Mana),
        cost: 10,
        cooldown: 3,
        target: AbilityTarget::Enemy,
        accuracy_bonus: 2,
        damage_min: 6,
        damage_max: 10,
        damage_type: DamageType::Holy,
        scaling_stat: MajorSkill::Wisdom,
        apply_status: Some((StatusKind::Weakness, 2, 2)),
        self_status: Some((StatusKind::Regen, 1, 4)),
        heal_amount: 0,
    },
    AbilityDef {
        id: "rabid_bite",
        name: "Rabid Bite",
        description: "A savage bite that leaves venom in the wound.",
        resource_kind: None,
        cost: 0,
        cooldown: 3,
        target: AbilityTarget::Enemy,
        accuracy_bonus: 1,
        damage_min: 4,
        damage_max: 6,
        damage_type: DamageType::Poison,
        scaling_stat: MajorSkill::Strength,
        apply_status: Some((StatusKind::Poison, 2, 2)),
        self_status: None,
        heal_amount: 0,
    },
    AbilityDef {
        id: "raider_gash",
        name: "Raider Gash",
        description: "A brutal slash meant to leave the target bleeding.",
        resource_kind: None,
        cost: 0,
        cooldown: 3,
        target: AbilityTarget::Enemy,
        accuracy_bonus: 1,
        damage_min: 5,
        damage_max: 8,
        damage_type: DamageType::Physical,
        scaling_stat: MajorSkill::Strength,
        apply_status: Some((StatusKind::Bleed, 2, 2)),
        self_status: None,
        heal_amount: 0,
    },
    AbilityDef {
        id: "grave_bolt",
        name: "Grave Bolt",
        description: "A cold shard of necrotic force from the barrow dead.",
        resource_kind: Some(ResourceKind::Mana),
        cost: 5,
        cooldown: 2,
        target: AbilityTarget::Enemy,
        accuracy_bonus: 1,
        damage_min: 5,
        damage_max: 8,
        damage_type: DamageType::Shadow,
        scaling_stat: MajorSkill::Intelligence,
        apply_status: Some((StatusKind::Weakness, 2, 1)),
        self_status: None,
        heal_amount: 0,
    },
];

pub const ENEMIES: &[EnemyDef] = &[
    EnemyDef {
        id: "wild_wolf",
        name: "Wild Wolf",
        family: "Beast",
        role: EnemyRole::Skirmisher,
        level: 1,
        hp: 24,
        mana: 0,
        stamina: 10,
        attack_bonus: 3,
        defense: 10,
        initiative: 3,
        damage_min: 3,
        damage_max: 5,
        weapon_kind: WeaponKind::Melee,
        ability_ids: &["rabid_bite"],
        loot: WOLF_LOOT,
        reward_xp: 20,
        reward_gold: 8,
    },
    EnemyDef {
        id: "alpha_wolf",
        name: "Alpha Wolf",
        family: "Beast",
        role: EnemyRole::Brute,
        level: 2,
        hp: 32,
        mana: 0,
        stamina: 12,
        attack_bonus: 4,
        defense: 11,
        initiative: 2,
        damage_min: 4,
        damage_max: 7,
        weapon_kind: WeaponKind::Melee,
        ability_ids: &["rabid_bite"],
        loot: WOLF_LOOT,
        reward_xp: 28,
        reward_gold: 12,
    },
    EnemyDef {
        id: "road_raider",
        name: "Road Raider",
        family: "Bandit",
        role: EnemyRole::Brute,
        level: 2,
        hp: 32,
        mana: 0,
        stamina: 14,
        attack_bonus: 4,
        defense: 11,
        initiative: 2,
        damage_min: 4,
        damage_max: 7,
        weapon_kind: WeaponKind::Melee,
        ability_ids: &["raider_gash"],
        loot: BANDIT_LOOT,
        reward_xp: 28,
        reward_gold: 15,
    },
    EnemyDef {
        id: "bandit_bowman",
        name: "Bandit Bowman",
        family: "Bandit",
        role: EnemyRole::Skirmisher,
        level: 2,
        hp: 26,
        mana: 0,
        stamina: 12,
        attack_bonus: 4,
        defense: 10,
        initiative: 4,
        damage_min: 3,
        damage_max: 6,
        weapon_kind: WeaponKind::Ranged,
        ability_ids: &[],
        loot: BANDIT_LOOT,
        reward_xp: 24,
        reward_gold: 12,
    },
    EnemyDef {
        id: "gravebound",
        name: "Gravebound",
        family: "Undead",
        role: EnemyRole::Brute,
        level: 3,
        hp: 38,
        mana: 4,
        stamina: 12,
        attack_bonus: 5,
        defense: 12,
        initiative: 2,
        damage_min: 5,
        damage_max: 8,
        weapon_kind: WeaponKind::Melee,
        ability_ids: &["grave_bolt"],
        loot: UNDEAD_LOOT,
        reward_xp: 34,
        reward_gold: 18,
    },
    EnemyDef {
        id: "grave_channeler",
        name: "Grave Channeler",
        family: "Undead",
        role: EnemyRole::Caster,
        level: 3,
        hp: 28,
        mana: 16,
        stamina: 8,
        attack_bonus: 4,
        defense: 11,
        initiative: 3,
        damage_min: 3,
        damage_max: 6,
        weapon_kind: WeaponKind::Magic,
        ability_ids: &["grave_bolt"],
        loot: UNDEAD_LOOT,
        reward_xp: 38,
        reward_gold: 22,
    },
];

pub const ENCOUNTERS: &[EncounterDef] = &[
    EncounterDef {
        id: "beast_hunt",
        name: "Beast Hunt",
        environment_tags: &["woods", "brush"],
        enemies: &["wild_wolf"],
    },
    EncounterDef {
        id: "beast_alpha",
        name: "Alpha Pack",
        environment_tags: &["woods", "moonlit"],
        enemies: &["alpha_wolf", "wild_wolf"],
    },
    EncounterDef {
        id: "bandit_ambush",
        name: "Bandit Ambush",
        environment_tags: &["road", "ditch"],
        enemies: &["road_raider", "bandit_bowman"],
    },
    EncounterDef {
        id: "bandit_raiders",
        name: "Raider Patrol",
        environment_tags: &["road", "rain"],
        enemies: &["road_raider", "road_raider"],
    },
    EncounterDef {
        id: "undead_patrol",
        name: "Undead Patrol",
        environment_tags: &["barrow", "ash"],
        enemies: &["gravebound", "gravebound"],
    },
    EncounterDef {
        id: "barrow_rites",
        name: "Barrow Rites",
        environment_tags: &["barrow", "ritual"],
        enemies: &["grave_channeler", "gravebound"],
    },
];

pub fn ability_def(id: &str) -> Option<&'static AbilityDef> {
    ABILITIES.iter().find(|ability| ability.id == id)
}

pub fn enemy_def(id: &str) -> &'static EnemyDef {
    ENEMIES
        .iter()
        .find(|enemy| enemy.id == id)
        .unwrap_or(&ENEMIES[0])
}

fn enemy_def_by_name(name: &str) -> &'static EnemyDef {
    ENEMIES
        .iter()
        .find(|enemy| enemy.name == name)
        .unwrap_or(&ENEMIES[0])
}

pub fn encounter_def(id: &str) -> &'static EncounterDef {
    ENCOUNTERS
        .iter()
        .find(|encounter| encounter.id == id)
        .unwrap_or(&ENCOUNTERS[0])
}

#[cfg(test)]
mod tests {
    use super::{
        ActionTab, CombatOutcome, CombatState, PlayerAction, StatusKind, TurnRef, ability_def,
        encounter_def,
    };
    use crate::character::{
        Class, KnownAbility, ProficiencyData, Race, ResourcePool, SavedCharacter, Stats,
    };
    use crate::inventory::{Equipment, InventoryItem, gear_package_items};

    fn test_character(class: Class) -> SavedCharacter {
        SavedCharacter {
            id: 1,
            name: "Test".to_string(),
            race: Race::Human,
            class,
            gear: "Melee Kit".to_string(),
            level: 3,
            xp: 300,
            gold: 20,
            unspent_stat_points: 0,
            stats: Stats {
                strength: 14,
                dexterity: 12,
                constitution: 13,
                intelligence: 12,
                wisdom: 10,
                charisma: 10,
            },
            resources: ResourcePool::full(60, 30, 30),
            proficiencies: vec![ProficiencyData {
                kind: crate::character::MinorSkill::Cooking,
                xp: 0,
            }],
            known_abilities: vec![KnownAbility {
                ability_id: "cleaving_blow".to_string(),
                rank: 1,
                unlocked: true,
                cooldown_remaining: 0,
            }],
        }
    }

    fn test_equipment() -> Equipment {
        let mut equipment = Equipment::default();
        for (slot, item) in gear_package_items("Melee Kit") {
            let slot = crate::inventory::EquipSlot::ALL
                .iter()
                .find(|it| it.db_key() == *slot)
                .copied()
                .unwrap();
            equipment.set_slot(slot, Some((*item).to_string()));
        }
        equipment
    }

    fn force_player_turn(combat: &mut CombatState) {
        combat.turn_index = combat
            .initiative
            .iter()
            .position(|turn| *turn == TurnRef::Player)
            .unwrap_or(0);
    }

    #[test]
    fn encounter_defs_exist() {
        assert_eq!(encounter_def("beast_hunt").enemies.len(), 1);
        assert!(ability_def("cleaving_blow").is_some());
    }

    #[test]
    fn player_can_win_combat() {
        let mut combat = CombatState::from_character_and_encounter(
            &test_character(Class::Warrior),
            &test_equipment(),
            &[],
            "beast_hunt",
        );
        force_player_turn(&mut combat);
        combat.action_tab = ActionTab::Weapon;
        for (idx, enemy) in combat.enemies.iter_mut().enumerate() {
            enemy.resources.hp = if idx == 0 { 1 } else { 0 };
            enemy.defense = 1;
        }
        let mut won = false;
        for _ in 0..32 {
            let outcome = combat.resolve_player_action(PlayerAction::UseWeapon);
            if matches!(outcome, CombatOutcome::Won(_)) {
                won = true;
                break;
            }
            force_player_turn(&mut combat);
        }
        assert!(won);
    }

    #[test]
    fn abilities_apply_status() {
        let mut combat = CombatState::from_character_and_encounter(
            &test_character(Class::Warrior),
            &test_equipment(),
            &[],
            "beast_hunt",
        );
        force_player_turn(&mut combat);
        combat.action_tab = ActionTab::Ability;
        for enemy in &mut combat.enemies {
            enemy.defense = 1;
        }
        let mut affected = false;
        for _ in 0..32 {
            let _ = combat.resolve_player_action(PlayerAction::UseAbility);
            if combat.enemies.iter().any(|enemy| {
                enemy
                    .statuses
                    .iter()
                    .any(|status| status.kind == StatusKind::Weakness)
            }) || combat
                .enemies
                .iter()
                .any(|enemy| enemy.resources.hp < enemy.resources.max_hp)
            {
                affected = true;
                break;
            }
            force_player_turn(&mut combat);
            if let Some(known) = combat
                .player
                .cooldowns
                .iter_mut()
                .find(|(id, _)| id == "cleaving_blow")
            {
                known.1 = 0;
            }
        }
        assert!(affected);
    }

    #[test]
    fn free_item_use_consumes_stack() {
        let mut combat = CombatState::from_character_and_encounter(
            &test_character(Class::Warrior),
            &test_equipment(),
            &[InventoryItem {
                item_type: "health_potion".to_string(),
                quantity: 2,
            }],
            "beast_hunt",
        );
        force_player_turn(&mut combat);
        combat.action_tab = ActionTab::Item;
        combat.player.resources.hp = 10;
        let _ = combat.resolve_player_action(PlayerAction::UseItem);
        assert!(combat.player.resources.hp > 10);
        assert_eq!(combat.consumables[0].quantity, 1);
    }

    #[test]
    fn opening_enemy_turn_resolves_before_player_input() {
        let mut combat = CombatState::from_character_and_encounter(
            &test_character(Class::Warrior),
            &test_equipment(),
            &[],
            "beast_hunt",
        );
        let enemy_first = combat
            .initiative
            .iter()
            .position(|turn| matches!(turn, TurnRef::Enemy(_)))
            .unwrap_or(0);
        combat.turn_index = enemy_first;

        let outcome = combat.begin_encounter();

        assert!(matches!(outcome, CombatOutcome::Ongoing));
        assert_eq!(combat.current_turn(), TurnRef::Player);
    }
}
