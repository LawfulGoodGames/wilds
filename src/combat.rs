use crate::character::{MajorSkill, SavedCharacter};
use crate::inventory::{AttackOption, Equipment};
use rand::RngExt;

#[derive(Debug, Clone)]
pub struct Enemy {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub attack_bonus: i32,
    pub defense: i32,
    pub min_damage: i32,
    pub max_damage: i32,
    pub reward_xp: i32,
    pub reward_gold: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Turn {
    Player,
    Enemy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerAction {
    UseSelectedAttack,
    Defend,
    Flee,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackKind {
    Melee,
    Ranged,
    Spell,
}

impl AttackKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Melee => "Melee",
            Self::Ranged => "Ranged",
            Self::Spell => "Spell",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatOutcome {
    Ongoing,
    Won { xp: i32, gold: i32 },
    Lost,
    Fled,
}

#[derive(Debug, Clone)]
pub struct CombatState {
    pub player_hp: i32,
    pub player_max_hp: i32,
    pub player_defense: i32,
    pub enemies: Vec<Enemy>,
    pub selected_enemy: usize,
    pub turn: Turn,
    pub defending: bool,
    pub active_attack_kind: AttackKind,
    pub selected_melee: usize,
    pub selected_ranged: usize,
    pub selected_spell: usize,
    pub melee_options: Vec<AttackOption>,
    pub ranged_options: Vec<AttackOption>,
    pub spell_options: Vec<AttackOption>,
    pub last_roll_summary: Option<String>,
    pub log: Vec<String>,
    pub new_log_entries: usize,
}

impl CombatState {
    pub fn from_character_with_equipment(
        character: &SavedCharacter,
        equipment: &Equipment,
    ) -> Self {
        let level = character.level.max(1);

        let melee_options = equipment.melee_attacks();
        let ranged_options = equipment.ranged_attacks();
        let spell_options = equipment.magic_attacks();

        // Default to the first attack type that has options available.
        let active_attack_kind = if !melee_options.is_empty() {
            AttackKind::Melee
        } else if !ranged_options.is_empty() {
            AttackKind::Ranged
        } else if !spell_options.is_empty() {
            AttackKind::Spell
        } else {
            AttackKind::Melee // no weapons — all options empty, player can only flee
        };

        Self {
            player_hp: character.hp.max(1),
            player_max_hp: character.max_hp.max(1),
            player_defense: Self::player_defense(character, equipment),
            enemies: vec![
                Enemy {
                    name: "Wild Wolf".to_string(),
                    hp: 20 + level * 3,
                    max_hp: 20 + level * 3,
                    attack_bonus: 3 + level / 2,
                    defense: 11 + level,
                    min_damage: 3 + level / 3,
                    max_damage: 6 + level / 2,
                    reward_xp: 30 + level * 10,
                    reward_gold: 8 + level * 3,
                },
                Enemy {
                    name: "Feral Fox".to_string(),
                    hp: 14 + level * 2,
                    max_hp: 14 + level * 2,
                    attack_bonus: 4 + level / 2,
                    defense: 12 + level,
                    min_damage: 2 + level / 3,
                    max_damage: 5 + level / 2,
                    reward_xp: 20 + level * 7,
                    reward_gold: 5 + level * 2,
                },
            ],
            selected_enemy: 0,
            turn: Turn::Player,
            defending: false,
            active_attack_kind,
            selected_melee: 0,
            selected_ranged: 0,
            selected_spell: 0,
            melee_options,
            ranged_options,
            spell_options,
            last_roll_summary: None,
            log: vec!["A Wild Wolf and a Feral Fox spring from the shadows.".to_string()],
            new_log_entries: 1,
        }
    }

    pub fn selected_options(&self) -> (&[AttackOption], usize) {
        match self.active_attack_kind {
            AttackKind::Melee => (&self.melee_options, self.selected_melee),
            AttackKind::Ranged => (&self.ranged_options, self.selected_ranged),
            AttackKind::Spell => (&self.spell_options, self.selected_spell),
        }
    }

    pub fn selected_option_name(&self) -> &'static str {
        let (options, selected_idx) = self.selected_options();
        options
            .get(selected_idx)
            .map(|o| o.name)
            .unwrap_or("Unknown")
    }

    pub fn set_attack_kind(&mut self, kind: AttackKind) {
        let available = match kind {
            AttackKind::Melee  => !self.melee_options.is_empty(),
            AttackKind::Ranged => !self.ranged_options.is_empty(),
            AttackKind::Spell  => !self.spell_options.is_empty(),
        };
        if available {
            self.active_attack_kind = kind;
        }
    }

    pub fn cycle_target(&mut self, dir: i32) {
        let n = self.enemies.len();
        if n == 0 {
            return;
        }
        let step = if dir >= 0 { 1i32 } else { -1i32 };
        let start = self.selected_enemy;
        let mut next = ((self.selected_enemy as i32 + step).rem_euclid(n as i32)) as usize;
        let mut count = 0;
        while self.enemies[next].hp == 0 && next != start && count < n {
            next = ((next as i32 + step).rem_euclid(n as i32)) as usize;
            count += 1;
        }
        self.selected_enemy = next;
    }

    fn auto_select_alive_enemy(&mut self) {
        let n = self.enemies.len();
        for i in 1..=n {
            let idx = (self.selected_enemy + i) % n;
            if self.enemies[idx].hp > 0 {
                self.selected_enemy = idx;
                return;
            }
        }
    }

    pub fn cycle_selected_option(&mut self, dir: i32) {
        let len = match self.active_attack_kind {
            AttackKind::Melee => self.melee_options.len(),
            AttackKind::Ranged => self.ranged_options.len(),
            AttackKind::Spell => self.spell_options.len(),
        };
        if len == 0 {
            return;
        }
        let cursor = match self.active_attack_kind {
            AttackKind::Melee => &mut self.selected_melee,
            AttackKind::Ranged => &mut self.selected_ranged,
            AttackKind::Spell => &mut self.selected_spell,
        };
        if dir > 0 {
            *cursor = (*cursor + 1) % len;
        } else if *cursor == 0 {
            *cursor = len - 1;
        } else {
            *cursor -= 1;
        }
    }

    pub fn resolve_player_action(
        &mut self,
        action: PlayerAction,
        character: &SavedCharacter,
    ) -> CombatOutcome {
        let mut rng = rand::rng();
        let attack_roll = rng.random_range(1..=20);
        let damage_roll = rng.random_range(1..=100);
        let enemy_attack_roll = rng.random_range(1..=20);
        let enemy_damage_roll = rng.random_range(1..=100);
        self.resolve_player_action_with_rolls(
            action,
            character,
            attack_roll,
            damage_roll,
            enemy_attack_roll,
            enemy_damage_roll,
        )
    }

    fn resolve_player_action_with_rolls(
        &mut self,
        action: PlayerAction,
        character: &SavedCharacter,
        attack_roll: i32,
        damage_roll: i32,
        enemy_attack_roll: i32,
        enemy_damage_roll: i32,
    ) -> CombatOutcome {
        if self.turn != Turn::Player {
            return CombatOutcome::Ongoing;
        }
        let start_log_len = self.log.len();

        match action {
            PlayerAction::UseSelectedAttack => {
                let (attack_kind, option_ref) = self.current_attack();
                let Some(option) = option_ref else {
                    self.log.push("No weapon equipped for this attack style.".to_string());
                    self.update_new_log_entries(start_log_len);
                    return CombatOutcome::Ongoing;
                };
                let option_name = option.name;
                let option_accuracy_bonus = option.accuracy_bonus;
                let option_min_damage = option.min_damage;
                let option_max_damage = option.max_damage;
                let attack_bonus = option_accuracy_bonus
                    + self.major_hit_bonus(character, attack_kind)
                    + self.class_attack_bonus(character, attack_kind)
                    + self.level_bonus(character);
                let target_defense = self.enemies[self.selected_enemy].defense;
                let target_name = self.enemies[self.selected_enemy].name.clone();
                let total_roll = attack_roll + attack_bonus;
                let hit = attack_roll == 20 || (attack_roll != 1 && total_roll >= target_defense);

                if hit {
                    let damage = self.roll_damage(option_min_damage, option_max_damage, damage_roll)
                        + self.major_damage_bonus(character, attack_kind)
                        + self.class_damage_bonus(character, attack_kind);
                    let damage = damage.max(1);
                    self.enemies[self.selected_enemy].hp =
                        (self.enemies[self.selected_enemy].hp - damage).max(0);
                    self.last_roll_summary = Some(format!(
                        "{} [{}] d20={} total={} vs DEF {} -> HIT, dmg={} ({})",
                        attack_kind.label(),
                        option_name,
                        attack_roll,
                        total_roll,
                        target_defense,
                        damage,
                        self.roll_damage(option_min_damage, option_max_damage, damage_roll),
                    ));
                    self.log.push(format!(
                        "{} [{}] roll {} + attack bonus {} = {} vs DEF {} -> HIT for {} damage.",
                        attack_kind.label(),
                        option_name,
                        attack_roll,
                        attack_bonus,
                        total_roll,
                        target_defense,
                        damage
                    ));
                    if self.enemies[self.selected_enemy].hp == 0 {
                        self.log.push(format!("{} is defeated.", target_name));
                        if self.enemies.iter().all(|e| e.hp == 0) {
                            let total_xp: i32 = self.enemies.iter().map(|e| e.reward_xp).sum();
                            let total_gold: i32 = self.enemies.iter().map(|e| e.reward_gold).sum();
                            self.update_new_log_entries(start_log_len);
                            return CombatOutcome::Won {
                                xp: total_xp,
                                gold: total_gold,
                            };
                        }
                        self.auto_select_alive_enemy();
                    }
                } else {
                    self.last_roll_summary = Some(format!(
                        "{} [{}] d20={} total={} vs DEF {} -> MISS",
                        attack_kind.label(),
                        option_name,
                        attack_roll,
                        total_roll,
                        target_defense
                    ));
                    self.log.push(format!(
                        "{} [{}] roll {} + attack bonus {} = {} vs DEF {} -> MISS.",
                        attack_kind.label(),
                        option_name,
                        attack_roll,
                        attack_bonus,
                        total_roll,
                        target_defense
                    ));
                }
            }
            PlayerAction::Defend => {
                self.defending = true;
                self.last_roll_summary = None;
                self.log.push("You brace for the next blow.".to_string());
            }
            PlayerAction::Flee => {
                self.last_roll_summary = None;
                let escape_bonus = self.ability_modifier(character.major_skill(MajorSkill::Dexterity))
                    + self.ability_modifier(character.major_skill(MajorSkill::Wisdom))
                    + self.level_bonus(character);
                let escape_dc = 10
                    + self.enemies.iter().filter(|e| e.hp > 0).count() as i32
                    + self
                        .enemies
                        .iter()
                        .filter(|e| e.hp > 0)
                        .map(|e| e.attack_bonus)
                        .max()
                        .unwrap_or(0);
                let total_roll = attack_roll + escape_bonus;
                if attack_roll == 20 || (attack_roll != 1 && total_roll >= escape_dc) {
                    self.log.push("You slip away from battle.".to_string());
                    self.last_roll_summary = Some(format!(
                        "Flee d20={} total={} vs DC {} -> SUCCESS",
                        attack_roll, total_roll, escape_dc
                    ));
                    self.update_new_log_entries(start_log_len);
                    return CombatOutcome::Fled;
                }
                self.last_roll_summary = Some(format!(
                    "Flee d20={} total={} vs DC {} -> FAIL",
                    attack_roll, total_roll, escape_dc
                ));
                self.log.push("You fail to escape.".to_string());
            }
        }

        self.turn = Turn::Enemy;
        let outcome = self.resolve_enemy_turn_with_rolls(character, enemy_attack_roll, enemy_damage_roll);
        if self.log.len() > 12 {
            let keep_from = self.log.len().saturating_sub(12);
            self.log.drain(0..keep_from);
        }
        self.update_new_log_entries(start_log_len);
        outcome
    }

    fn resolve_enemy_turn_with_rolls(
        &mut self,
        character: &SavedCharacter,
        attack_roll: i32,
        damage_roll: i32,
    ) -> CombatOutcome {
        let effective_defense = self.player_defense + if self.defending { 4 } else { 0 };
        let attacks: Vec<(String, i32, i32, i32)> = self
            .enemies
            .iter()
            .filter(|e| e.hp > 0)
            .map(|e| (e.name.clone(), e.attack_bonus, e.min_damage, e.max_damage))
            .collect();
        for (name, attack_bonus, min_damage, max_damage) in attacks {
            let total_roll = attack_roll + attack_bonus;
            let hit = attack_roll == 20 || (attack_roll != 1 && total_roll >= effective_defense);
            if hit {
                let mut damage = self.roll_damage(min_damage, max_damage, damage_roll);
                damage += self.enemy_damage_bonus(character);
                if self.defending {
                    damage = (damage + 1) / 2;
                }
                damage = damage.max(1);
                self.player_hp = (self.player_hp - damage).max(0);
                self.log.push(format!(
                    "{} attacks with roll {} + {} = {} vs DEF {} and hits for {} damage.",
                    name, attack_roll, attack_bonus, total_roll, effective_defense, damage
                ));
            } else {
                self.log.push(format!(
                    "{} attacks with roll {} + {} = {} vs DEF {} and misses.",
                    name, attack_roll, attack_bonus, total_roll, effective_defense
                ));
            }
            if self.player_hp == 0 {
                break;
            }
        }
        self.defending = false;
        self.turn = Turn::Player;
        if self.player_hp == 0 {
            self.log.push("You collapse from your wounds.".to_string());
            return CombatOutcome::Lost;
        }
        CombatOutcome::Ongoing
    }

    fn current_attack(&self) -> (AttackKind, Option<&AttackOption>) {
        let (options, idx) = self.selected_options();
        (self.active_attack_kind, options.get(idx))
    }

    fn major_hit_bonus(&self, character: &SavedCharacter, kind: AttackKind) -> i32 {
        match kind {
            AttackKind::Melee => self.ability_modifier(character.major_skill(MajorSkill::Strength)),
            AttackKind::Ranged => self.ability_modifier(character.major_skill(MajorSkill::Dexterity)),
            AttackKind::Spell => self.ability_modifier(character.major_skill(MajorSkill::Intelligence)),
        }
    }

    fn major_damage_bonus(&self, character: &SavedCharacter, kind: AttackKind) -> i32 {
        match kind {
            AttackKind::Melee => self.ability_modifier(character.major_skill(MajorSkill::Strength)),
            AttackKind::Ranged => self.ability_modifier(character.major_skill(MajorSkill::Dexterity)),
            AttackKind::Spell => self.ability_modifier(character.major_skill(MajorSkill::Intelligence)),
        }
    }

    fn class_attack_bonus(&self, character: &SavedCharacter, kind: AttackKind) -> i32 {
        match (character.class.as_str(), kind) {
            ("Warrior", AttackKind::Melee) | ("Paladin", AttackKind::Melee) => 2,
            ("Ranger", AttackKind::Ranged) | ("Rogue", AttackKind::Ranged) => 2,
            ("Mage", AttackKind::Spell) | ("Cleric", AttackKind::Spell) => 2,
            ("Rogue", AttackKind::Melee) => 1,
            ("Cleric", AttackKind::Melee) => 1,
            _ => 0,
        }
    }

    fn class_damage_bonus(&self, character: &SavedCharacter, kind: AttackKind) -> i32 {
        match (character.class.as_str(), kind) {
            ("Warrior", AttackKind::Melee) | ("Paladin", AttackKind::Melee) => 1,
            ("Ranger", AttackKind::Ranged) | ("Rogue", AttackKind::Ranged) => 1,
            ("Mage", AttackKind::Spell) | ("Cleric", AttackKind::Spell) => 1,
            _ => 0,
        }
    }

    fn level_bonus(&self, character: &SavedCharacter) -> i32 {
        ((character.level.max(1) - 1) / 4).max(0)
    }

    fn ability_modifier(&self, score: i32) -> i32 {
        (score - 8).div_euclid(2)
    }

    fn enemy_damage_bonus(&self, character: &SavedCharacter) -> i32 {
        let constitution = self.ability_modifier(character.major_skill(MajorSkill::Constitution));
        constitution.min(0).abs()
    }

    fn player_defense(character: &SavedCharacter, equipment: &Equipment) -> i32 {
        let dexterity = (character.major_skill(MajorSkill::Dexterity) - 8).div_euclid(2);
        let wisdom = (character.major_skill(MajorSkill::Wisdom) - 8).div_euclid(2);
        let armor = equipment.total_armor_bonus() / 2;
        10 + dexterity + wisdom.max(0) / 2 + armor
    }

    fn roll_damage(&self, min_damage: i32, max_damage: i32, damage_roll: i32) -> i32 {
        let span = (max_damage - min_damage + 1).max(1);
        min_damage + ((damage_roll - 1).max(0) % span)
    }

    fn update_new_log_entries(&mut self, start_log_len: usize) {
        self.new_log_entries = self.log.len().saturating_sub(start_log_len);
    }
}

#[cfg(test)]
mod tests {
    use super::{AttackKind, CombatOutcome, CombatState, Enemy, PlayerAction, Turn};
    use crate::character::{MajorSkill, MajorSkillData, MinorSkill, MinorSkillData, SavedCharacter};
    use crate::inventory::AttackOption;

    fn test_character(strength: i32, dexterity: i32, constitution: i32, wisdom: i32) -> SavedCharacter {
        SavedCharacter {
            id: 1,
            name: "Tester".to_string(),
            race: "Human".to_string(),
            class: "Warrior".to_string(),
            gear: "Melee Kit".to_string(),
            level: 1,
            xp: 0,
            hp: 20,
            max_hp: 20,
            gold: 0,
            major_skills: vec![
                MajorSkillData { kind: MajorSkill::Strength, points: strength },
                MajorSkillData { kind: MajorSkill::Dexterity, points: dexterity },
                MajorSkillData { kind: MajorSkill::Constitution, points: constitution },
                MajorSkillData { kind: MajorSkill::Intelligence, points: 5 },
                MajorSkillData { kind: MajorSkill::Wisdom, points: wisdom },
                MajorSkillData { kind: MajorSkill::Charisma, points: 5 },
            ],
            minor_skills: MinorSkill::ALL
                .iter()
                .map(|kind| MinorSkillData {
                    kind: *kind,
                    xp: 10_000,
                })
                .collect(),
        }
    }

    fn test_combat(enemy_hp: i32, enemy_attack_bonus: i32, defense: i32) -> CombatState {
        CombatState {
            player_hp: 20,
            player_max_hp: 20,
            player_defense: 12,
            enemies: vec![Enemy {
                name: "Dummy".to_string(),
                hp: enemy_hp,
                max_hp: enemy_hp,
                attack_bonus: enemy_attack_bonus,
                defense,
                min_damage: 4,
                max_damage: 6,
                reward_xp: 25,
                reward_gold: 8,
            }],
            selected_enemy: 0,
            turn: Turn::Player,
            defending: false,
            active_attack_kind: AttackKind::Melee,
            selected_melee: 0,
            selected_ranged: 0,
            selected_spell: 0,
            melee_options: vec![AttackOption {
                name: "Test Blade",
                accuracy_bonus: 0,
                min_damage: 4,
                max_damage: 6,
            }],
            ranged_options: vec![AttackOption {
                name: "Test Bow",
                accuracy_bonus: 0,
                min_damage: 3,
                max_damage: 5,
            }],
            spell_options: vec![AttackOption {
                name: "Test Bolt",
                accuracy_bonus: 0,
                min_damage: 5,
                max_damage: 7,
            }],
            last_roll_summary: None,
            log: Vec::new(),
            new_log_entries: 0,
        }
    }

    #[test]
    fn selected_attack_hits_when_roll_meets_ac() {
        let ch = test_character(8, 6, 5, 5);
        let mut combat = test_combat(30, 5, 45);

        let outcome = combat.resolve_player_action_with_rolls(
            PlayerAction::UseSelectedAttack,
            &ch,
            80,
            50,
            12,
            50,
        );

        assert!(matches!(outcome, CombatOutcome::Ongoing));
        assert!(combat.enemies[0].hp < 30);
        assert!(combat.player_hp < 20);
    }

    #[test]
    fn selected_attack_misses_when_roll_below_ac() {
        let ch = test_character(8, 6, 5, 5);
        let mut combat = test_combat(30, 5, 18);

        let outcome = combat.resolve_player_action_with_rolls(
            PlayerAction::UseSelectedAttack,
            &ch,
            2,
            50,
            10,
            50,
        );

        assert!(matches!(outcome, CombatOutcome::Ongoing));
        assert_eq!(combat.enemies[0].hp, 30);
    }

    #[test]
    fn selected_attack_can_win_and_grant_rewards() {
        let ch = test_character(10, 10, 5, 5);
        let mut combat = test_combat(2, 5, 20);

        let outcome = combat.resolve_player_action_with_rolls(
            PlayerAction::UseSelectedAttack,
            &ch,
            19,
            100,
            10,
            100,
        );

        assert!(matches!(
            outcome,
            CombatOutcome::Won { xp: 25, gold: 8 }
        ));
        assert_eq!(combat.enemies[0].hp, 0);
    }

    #[test]
    fn defend_reduces_incoming_damage() {
        let ch = test_character(5, 5, 5, 5);
        let mut normal = test_combat(30, 8, 50);
        let mut defended = test_combat(30, 8, 50);

        let _ = normal.resolve_player_action_with_rolls(PlayerAction::UseSelectedAttack, &ch, 1, 1, 15, 60);
        let _ = defended.resolve_player_action_with_rolls(PlayerAction::Defend, &ch, 1, 1, 15, 60);

        let normal_damage = 20 - normal.player_hp;
        let defended_damage = 20 - defended.player_hp;
        assert!(defended_damage < normal_damage);
    }

    #[test]
    fn flee_can_succeed_with_high_stats() {
        let ch = test_character(5, 10, 5, 10);
        let mut combat = test_combat(30, 5, 50);

        let outcome = combat.resolve_player_action_with_rolls(
            PlayerAction::Flee,
            &ch,
            18,
            1,
            1,
            1,
        );

        assert!(matches!(outcome, CombatOutcome::Fled));
    }

    #[test]
    fn defeat_returns_lost_outcome() {
        let ch = test_character(5, 5, 5, 5);
        let mut combat = test_combat(30, 12, 18);
        combat.enemies[0].min_damage = 25;
        combat.enemies[0].max_damage = 25;

        let outcome = combat.resolve_player_action_with_rolls(
            PlayerAction::UseSelectedAttack,
            &ch,
            1,
            1,
            20,
            100,
        );

        assert!(matches!(outcome, CombatOutcome::Lost));
        assert_eq!(combat.player_hp, 0);
    }
}
