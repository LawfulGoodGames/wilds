use crate::character::{MajorSkill, MinorSkill, SavedCharacter};
use rand::RngExt;

#[derive(Debug, Clone)]
pub struct Enemy {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub armor_class: i32,
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

#[derive(Debug, Clone)]
pub struct AttackOption {
    pub name: &'static str,
    pub accuracy_bonus: i32,
    pub min_damage: i32,
    pub max_damage: i32,
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
    pub enemy: Enemy,
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
    pub fn from_character(character: &SavedCharacter) -> Self {
        let level = character.level.max(1);
        let enemy_hp = 20 + level * 3;
        let enemy_attack = 4 + level / 2;
        Self {
            player_hp: character.hp.max(1),
            player_max_hp: character.max_hp.max(1),
            enemy: Enemy {
                name: "Wild Wolf".to_string(),
                hp: enemy_hp,
                max_hp: enemy_hp,
                attack: enemy_attack,
                armor_class: 48 + level * 2,
                reward_xp: 30 + level * 10,
                reward_gold: 8 + level * 3,
            },
            turn: Turn::Player,
            defending: false,
            active_attack_kind: AttackKind::Melee,
            selected_melee: 0,
            selected_ranged: 0,
            selected_spell: 0,
            melee_options: vec![
                AttackOption {
                    name: "Iron Sword",
                    accuracy_bonus: 6,
                    min_damage: 4,
                    max_damage: 9,
                },
                AttackOption {
                    name: "Twin Daggers",
                    accuracy_bonus: 8,
                    min_damage: 3,
                    max_damage: 8,
                },
            ],
            ranged_options: vec![
                AttackOption {
                    name: "Hunting Bow",
                    accuracy_bonus: 7,
                    min_damage: 4,
                    max_damage: 8,
                },
                AttackOption {
                    name: "Throwing Knife",
                    accuracy_bonus: 9,
                    min_damage: 2,
                    max_damage: 6,
                },
            ],
            spell_options: vec![
                AttackOption {
                    name: "Arcane Bolt",
                    accuracy_bonus: 7,
                    min_damage: 5,
                    max_damage: 10,
                },
                AttackOption {
                    name: "Ember Lance",
                    accuracy_bonus: 5,
                    min_damage: 6,
                    max_damage: 12,
                },
            ],
            last_roll_summary: None,
            log: vec!["A Wild Wolf leaps from the brush.".to_string()],
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
        self.active_attack_kind = kind;
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
        let attack_roll = rng.random_range(1..=100);
        let damage_roll = rng.random_range(1..=100);
        self.resolve_player_action_with_rolls(action, character, attack_roll, damage_roll)
    }

    fn resolve_player_action_with_rolls(
        &mut self,
        action: PlayerAction,
        character: &SavedCharacter,
        attack_roll: i32,
        damage_roll: i32,
    ) -> CombatOutcome {
        if self.turn != Turn::Player {
            return CombatOutcome::Ongoing;
        }
        let start_log_len = self.log.len();

        match action {
            PlayerAction::UseSelectedAttack => {
                let (attack_kind, option) = self.current_attack();
                let option_name = option.name;
                let option_accuracy_bonus = option.accuracy_bonus;
                let option_min_damage = option.min_damage;
                let option_max_damage = option.max_damage;
                let major_bonus = self.major_hit_bonus(character, attack_kind);
                let minor_bonus = self.minor_hit_bonus(character, attack_kind);
                let total_roll = attack_roll + option_accuracy_bonus + major_bonus + minor_bonus;
                let hit = total_roll >= self.enemy.armor_class;

                if hit {
                    let damage = self.roll_damage(option_min_damage, option_max_damage, damage_roll)
                        + self.major_damage_bonus(character, attack_kind)
                        + self.minor_damage_bonus(character, attack_kind);
                    let damage = damage.max(1);
                    self.enemy.hp = (self.enemy.hp - damage).max(0);
                    self.last_roll_summary = Some(format!(
                        "{} [{}] d100={} total={} vs AC {} -> HIT, dmgRoll={} dmg={}",
                        attack_kind.label(),
                        option_name,
                        attack_roll,
                        total_roll,
                        self.enemy.armor_class,
                        damage_roll,
                        damage
                    ));
                    self.log.push(format!(
                        "{} [{}] roll {} + bonuses = {} vs AC {} -> HIT for {} damage.",
                        attack_kind.label(),
                        option_name,
                        attack_roll,
                        total_roll,
                        self.enemy.armor_class,
                        damage
                    ));
                    if self.enemy.hp == 0 {
                        self.log.push(format!("{} is defeated.", self.enemy.name));
                        self.update_new_log_entries(start_log_len);
                        return CombatOutcome::Won {
                            xp: self.enemy.reward_xp,
                            gold: self.enemy.reward_gold,
                        };
                    }
                } else {
                    self.last_roll_summary = Some(format!(
                        "{} [{}] d100={} total={} vs AC {} -> MISS",
                        attack_kind.label(),
                        option_name,
                        attack_roll,
                        total_roll,
                        self.enemy.armor_class
                    ));
                    self.log.push(format!(
                        "{} [{}] roll {} + bonuses = {} vs AC {} -> MISS.",
                        attack_kind.label(),
                        option_name,
                        attack_roll,
                        total_roll,
                        self.enemy.armor_class
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
                let dexterity = character.major_skill(MajorSkill::Dexterity);
                let wisdom = character.major_skill(MajorSkill::Wisdom);
                if dexterity + wisdom >= self.enemy.attack * 2 {
                    self.log.push("You slip away from battle.".to_string());
                    self.update_new_log_entries(start_log_len);
                    return CombatOutcome::Fled;
                }
                self.log.push("You fail to escape.".to_string());
            }
        }

        self.turn = Turn::Enemy;
        let outcome = self.resolve_enemy_turn(character);
        if self.log.len() > 12 {
            let keep_from = self.log.len().saturating_sub(12);
            self.log.drain(0..keep_from);
        }
        self.update_new_log_entries(start_log_len);
        outcome
    }

    fn resolve_enemy_turn(&mut self, character: &SavedCharacter) -> CombatOutcome {
        let constitution = character.major_skill(MajorSkill::Constitution);
        let mitigation = constitution / 5 + if self.defending { 2 } else { 0 };
        let damage = (self.enemy.attack - mitigation).max(1);
        self.player_hp = (self.player_hp - damage).max(0);
        self.log
            .push(format!("{} claws you for {} damage.", self.enemy.name, damage));
        self.defending = false;
        self.turn = Turn::Player;
        if self.player_hp == 0 {
            self.log.push("You collapse from your wounds.".to_string());
            return CombatOutcome::Lost;
        }
        CombatOutcome::Ongoing
    }

    fn current_attack(&self) -> (AttackKind, &AttackOption) {
        match self.active_attack_kind {
            AttackKind::Melee => (
                AttackKind::Melee,
                &self.melee_options[self.selected_melee.min(self.melee_options.len().saturating_sub(1))],
            ),
            AttackKind::Ranged => (
                AttackKind::Ranged,
                &self.ranged_options[self.selected_ranged.min(self.ranged_options.len().saturating_sub(1))],
            ),
            AttackKind::Spell => (
                AttackKind::Spell,
                &self.spell_options[self.selected_spell.min(self.spell_options.len().saturating_sub(1))],
            ),
        }
    }

    fn major_hit_bonus(&self, character: &SavedCharacter, kind: AttackKind) -> i32 {
        match kind {
            AttackKind::Melee => {
                character.major_skill(MajorSkill::Strength) / 2
                    + character.major_skill(MajorSkill::Dexterity) / 4
            }
            AttackKind::Ranged => {
                character.major_skill(MajorSkill::Dexterity) / 2
                    + character.major_skill(MajorSkill::Wisdom) / 4
            }
            AttackKind::Spell => {
                character.major_skill(MajorSkill::Intelligence) / 2
                    + character.major_skill(MajorSkill::Wisdom) / 4
            }
        }
    }

    fn major_damage_bonus(&self, character: &SavedCharacter, kind: AttackKind) -> i32 {
        match kind {
            AttackKind::Melee => character.major_skill(MajorSkill::Strength) / 3,
            AttackKind::Ranged => character.major_skill(MajorSkill::Dexterity) / 3,
            AttackKind::Spell => character.major_skill(MajorSkill::Intelligence) / 3,
        }
    }

    fn minor_hit_bonus(&self, character: &SavedCharacter, kind: AttackKind) -> i32 {
        match kind {
            AttackKind::Melee => {
                self.minor_level(character, MinorSkill::Blacksmithing) / 8
                    + self.minor_level(character, MinorSkill::Mining) / 12
            }
            AttackKind::Ranged => {
                self.minor_level(character, MinorSkill::Thieving) / 8
                    + self.minor_level(character, MinorSkill::Woodcutting) / 12
            }
            AttackKind::Spell => {
                self.minor_level(character, MinorSkill::Enchanting) / 8
                    + self.minor_level(character, MinorSkill::Runecrafting) / 8
            }
        }
    }

    fn minor_damage_bonus(&self, character: &SavedCharacter, kind: AttackKind) -> i32 {
        match kind {
            AttackKind::Melee => self.minor_level(character, MinorSkill::Blacksmithing) / 15,
            AttackKind::Ranged => self.minor_level(character, MinorSkill::Thieving) / 15,
            AttackKind::Spell => {
                (self.minor_level(character, MinorSkill::Enchanting)
                    + self.minor_level(character, MinorSkill::Runecrafting))
                    / 20
            }
        }
    }

    fn minor_level(&self, character: &SavedCharacter, skill: MinorSkill) -> i32 {
        character
            .minor_skills
            .iter()
            .find(|s| s.kind == skill)
            .map(|s| s.level() as i32)
            .unwrap_or(1)
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
    use super::{AttackKind, AttackOption, CombatOutcome, CombatState, Enemy, PlayerAction, Turn};
    use crate::character::{MajorSkill, MajorSkillData, MinorSkill, MinorSkillData, SavedCharacter};

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

    fn test_combat(enemy_hp: i32, enemy_attack: i32, armor_class: i32) -> CombatState {
        CombatState {
            player_hp: 20,
            player_max_hp: 20,
            enemy: Enemy {
                name: "Dummy".to_string(),
                hp: enemy_hp,
                max_hp: enemy_hp,
                attack: enemy_attack,
                armor_class,
                reward_xp: 25,
                reward_gold: 8,
            },
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
        );

        assert!(matches!(outcome, CombatOutcome::Ongoing));
        assert!(combat.enemy.hp < 30);
        assert!(combat.player_hp < 20);
    }

    #[test]
    fn selected_attack_misses_when_roll_below_ac() {
        let ch = test_character(8, 6, 5, 5);
        let mut combat = test_combat(30, 5, 95);

        let outcome = combat.resolve_player_action_with_rolls(
            PlayerAction::UseSelectedAttack,
            &ch,
            20,
            50,
        );

        assert!(matches!(outcome, CombatOutcome::Ongoing));
        assert_eq!(combat.enemy.hp, 30);
    }

    #[test]
    fn selected_attack_can_win_and_grant_rewards() {
        let ch = test_character(10, 10, 5, 5);
        let mut combat = test_combat(2, 5, 20);

        let outcome = combat.resolve_player_action_with_rolls(
            PlayerAction::UseSelectedAttack,
            &ch,
            90,
            100,
        );

        assert!(matches!(
            outcome,
            CombatOutcome::Won { xp: 25, gold: 8 }
        ));
        assert_eq!(combat.enemy.hp, 0);
    }

    #[test]
    fn defend_reduces_incoming_damage() {
        let ch = test_character(5, 5, 5, 5);
        let mut normal = test_combat(30, 8, 50);
        let mut defended = test_combat(30, 8, 50);

        let _ = normal.resolve_player_action_with_rolls(PlayerAction::UseSelectedAttack, &ch, 1, 1);
        let _ = defended.resolve_player_action(PlayerAction::Defend, &ch);

        let normal_damage = 20 - normal.player_hp;
        let defended_damage = 20 - defended.player_hp;
        assert!(defended_damage < normal_damage);
    }

    #[test]
    fn flee_can_succeed_with_high_stats() {
        let ch = test_character(5, 10, 5, 10);
        let mut combat = test_combat(30, 5, 50);

        let outcome = combat.resolve_player_action(PlayerAction::Flee, &ch);

        assert!(matches!(outcome, CombatOutcome::Fled));
    }

    #[test]
    fn defeat_returns_lost_outcome() {
        let ch = test_character(5, 5, 5, 5);
        let mut combat = test_combat(30, 50, 50);

        let outcome = combat.resolve_player_action_with_rolls(
            PlayerAction::UseSelectedAttack,
            &ch,
            1,
            1,
        );

        assert!(matches!(outcome, CombatOutcome::Lost));
        assert_eq!(combat.player_hp, 0);
    }
}
