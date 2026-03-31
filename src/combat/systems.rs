use super::*;

impl CombatState {
    pub(super) fn resolve_attack(
        &mut self,
        player_is_actor: bool,
        ability_name: Option<&str>,
        label: &str,
        base_accuracy: i32,
        accuracy_bonus: i32,
        min_damage: i32,
        max_damage: i32,
        damage_bonus: i32,
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
        let accuracy = base_accuracy
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
            rng.random_range(min_damage..=max_damage) + damage_bonus.max(0)
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
        let mut dealt_damage = damage;
        if hit {
            if let Some(target) = self.enemies.get_mut(self.selected_target) {
                dealt_damage =
                    Self::apply_resistance_to_target(damage, target.resistances, damage_type);
                target.resources.hp = (target.resources.hp - dealt_damage).max(0);
                if player_is_actor {
                    self.player_damage_dealt += dealt_damage;
                }
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
            amount: if hit { dealt_damage } else { damage },
            detail: ability_name.unwrap_or(label).to_string(),
        });
    }

    pub(super) fn apply_resistance(
        &self,
        amount: i32,
        resistances: ResistanceProfile,
        damage_type: DamageType,
    ) -> i32 {
        Self::apply_resistance_to_target(amount, resistances, damage_type)
    }

    pub(super) fn apply_resistance_to_target(
        amount: i32,
        resistances: ResistanceProfile,
        damage_type: DamageType,
    ) -> i32 {
        let resistance = match damage_type {
            DamageType::Physical => resistances.physical,
            DamageType::Fire => resistances.fire,
            DamageType::Frost => resistances.frost,
            DamageType::Lightning => resistances.lightning,
            DamageType::Poison => resistances.poison,
            DamageType::Holy => resistances.holy,
            DamageType::Shadow => resistances.shadow,
        };
        (amount - resistance).max(0)
    }

    pub(super) fn player_guard_bonus(&self) -> i32 {
        self.player
            .statuses
            .iter()
            .filter(|status| status.kind == StatusKind::Guard)
            .map(|status| status.potency)
            .sum()
    }

    pub(super) fn tick_statuses_for_player(&mut self, timing: StatusTiming) {
        Self::tick_statuses(&mut self.player, timing, &mut self.log);
    }

    pub(super) fn tick_statuses_for_enemy(&mut self, idx: usize, timing: StatusTiming) {
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

    pub(super) fn apply_status_to_player(
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

    pub(super) fn apply_status_to_enemy(
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

    pub(super) fn can_pay_cost(&self, resource_kind: Option<ResourceKind>, cost: i32) -> bool {
        match resource_kind {
            Some(ResourceKind::Mana) => self.player.resources.mana >= cost,
            Some(ResourceKind::Stamina) => self.player.resources.stamina >= cost,
            None => true,
        }
    }

    pub(super) fn pay_cost(&mut self, resource_kind: Option<ResourceKind>, cost: i32) {
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

    pub(super) fn enemy_can_pay_cost(
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

    pub(super) fn pay_enemy_cost(
        &mut self,
        idx: usize,
        resource_kind: Option<ResourceKind>,
        cost: i32,
    ) {
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

    pub(super) fn cooldown_for_player(&self, ability_id: &str) -> i32 {
        self.player
            .cooldowns
            .iter()
            .find(|(id, _)| id == ability_id)
            .map(|(_, value)| *value)
            .unwrap_or(0)
    }

    pub(super) fn set_player_cooldown(&mut self, ability_id: &str, cooldown: i32) {
        if let Some(entry) = self
            .player
            .cooldowns
            .iter_mut()
            .find(|(id, _)| id == ability_id)
        {
            entry.1 = cooldown;
        }
    }

    pub(super) fn cooldown_for_enemy(&self, idx: usize, ability_id: &str) -> i32 {
        self.enemies
            .get(idx)
            .and_then(|enemy| enemy.cooldowns.iter().find(|(id, _)| id == ability_id))
            .map(|(_, value)| *value)
            .unwrap_or(0)
    }

    pub(super) fn set_enemy_cooldown(&mut self, idx: usize, ability_id: &str, cooldown: i32) {
        if let Some(entry) = self
            .enemies
            .get_mut(idx)
            .and_then(|enemy| enemy.cooldowns.iter_mut().find(|(id, _)| id == ability_id))
        {
            entry.1 = cooldown;
        }
    }

    pub(super) fn advance_turn(&mut self) {
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

    pub(super) fn collect_rewards(&mut self) -> CombatReward {
        let mut xp = 0;
        let mut gold = 0;
        let mut defeated_families = vec![];
        let mut beast_kills = 0;
        let mut bandit_kills = 0;
        let mut undead_kills = 0;
        let mut drops = vec![];
        let mut rng = rand::rng();
        for enemy in &self.enemies {
            if !defeated_families.contains(&enemy.family) {
                defeated_families.push(enemy.family.clone());
            }
            match enemy.family.as_str() {
                "Beast" => beast_kills += 1,
                "Bandit" => bandit_kills += 1,
                "Undead" => undead_kills += 1,
                _ => {}
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
            enemies_defeated: self.enemies.len() as i32,
            beast_kills,
            bandit_kills,
            undead_kills,
            damage_dealt: self.player_damage_dealt,
            ability_uses: self.player_ability_uses,
            weapon_attacks: self.player_weapon_attacks,
            item_uses: self.player_item_uses,
        }
    }

    pub(super) fn trim_log(&mut self) {
        if self.log.len() > 18 {
            let keep = self.log.len().saturating_sub(18);
            self.log.drain(0..keep);
        }
    }

    pub(super) fn update_new_entries(&mut self, start_len: usize) {
        self.new_log_entries = self.log.len().saturating_sub(start_len);
    }
}

impl AbilityDef {
    pub(super) fn damage_type_label(&self) -> &'static str {
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
