use super::*;

impl CombatState {
    pub(super) fn resolve_enemy_round(&mut self) -> CombatOutcome {
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
        let damage = self.apply_resistance(
            base_damage.max(0),
            self.player.resistances,
            ability.damage_type,
        );
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
            self.apply_resistance(
                (rng.random_range(attack.min_damage..=attack.max_damage)
                    - self.player_guard_bonus())
                .max(0),
                self.player.resistances,
                DamageType::Physical,
            )
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
}
