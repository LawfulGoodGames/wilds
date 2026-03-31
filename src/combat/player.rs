use super::*;

impl CombatState {
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
        let (base_accuracy, damage_bonus, damage_type, resource_kind, cost) =
            self.player_weapon_profile(attack);
        if !self.can_pay_cost(resource_kind, cost) {
            let resource_name = match resource_kind {
                Some(ResourceKind::Mana) => "mana",
                Some(ResourceKind::Stamina) => "stamina",
                None => "resource",
            };
            self.log.push(CombatLogEvent::Info(format!(
                "Not enough {resource_name} for {}.",
                attack.name
            )));
            return;
        }
        self.pay_cost(resource_kind, cost);
        self.player_weapon_attacks += 1;
        self.resolve_attack(
            true,
            None,
            attack.name,
            base_accuracy,
            attack.accuracy_bonus,
            attack.min_damage,
            attack.max_damage,
            damage_bonus,
            damage_type,
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
        self.player_ability_uses += 1;
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
            self.player_accuracy_for_skill(ability.scaling_stat),
            ability.accuracy_bonus,
            ability.damage_min,
            ability.damage_max,
            self.player_damage_bonus_for_ability(ability),
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
        self.player_item_uses += 1;
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

    fn player_weapon_profile(
        &self,
        attack: AttackOption,
    ) -> (i32, i32, DamageType, Option<ResourceKind>, i32) {
        match self.player.weapon_kind.unwrap_or(WeaponKind::Melee) {
            WeaponKind::Melee => (
                self.player.attack_bonus,
                self.player.strength_bonus.max(0),
                DamageType::Physical,
                None,
                0,
            ),
            WeaponKind::Ranged => (
                self.player.ranged_attack_bonus,
                (self.player.ranged_attack_bonus / 2).max(0),
                DamageType::Physical,
                None,
                0,
            ),
            WeaponKind::Magic => (
                self.player.magic_attack_bonus,
                (self.player.spell_power / 3).max(1),
                Self::weapon_damage_type(attack.name),
                Some(ResourceKind::Mana),
                Self::weapon_mana_cost(attack),
            ),
        }
    }

    fn player_accuracy_for_skill(&self, skill: MajorSkill) -> i32 {
        match skill {
            MajorSkill::Strength | MajorSkill::Charisma => self.player.attack_bonus,
            MajorSkill::Dexterity => self.player.ranged_attack_bonus,
            MajorSkill::Intelligence => self.player.magic_attack_bonus,
            MajorSkill::Wisdom => self.player.prayer_attack_bonus,
            MajorSkill::Constitution => self.player.attack_bonus,
        }
    }

    fn player_damage_bonus_for_ability(&self, ability: &AbilityDef) -> i32 {
        match ability.damage_type {
            DamageType::Physical => self.player.strength_bonus.max(0),
            DamageType::Holy => (self.player.healing_power / 3).max(1),
            DamageType::Fire
            | DamageType::Frost
            | DamageType::Lightning
            | DamageType::Poison
            | DamageType::Shadow => (self.player.spell_power / 3).max(1),
        }
    }

    fn weapon_damage_type(name: &str) -> DamageType {
        let lower = name.to_ascii_lowercase();
        if lower.contains("frost") {
            DamageType::Frost
        } else if lower.contains("ember") || lower.contains("fire") {
            DamageType::Fire
        } else if lower.contains("storm") || lower.contains("spark") || lower.contains("arcane") {
            DamageType::Lightning
        } else {
            DamageType::Fire
        }
    }

    fn weapon_mana_cost(attack: AttackOption) -> i32 {
        let average = (attack.min_damage + attack.max_damage) / 2;
        (average / 3).clamp(2, 5)
    }
}
