use super::{
    ActionTab, CombatOutcome, CombatState, DamageType, PlayerAction, StatusKind, TurnRef,
    ability_def, encounter_def,
};
use crate::character::{
    Class, KnownAbility, MajorProficiencyData, MajorSkill, MinorSkill, ProficiencyData, Race,
    ResourcePool, SavedCharacter, Stats, proficiency_xp_for_level,
};
use crate::inventory::{Equipment, InventoryItem, gear_package_items};
use crate::settings::Difficulty;

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
        major_proficiencies: MajorSkill::ALL
            .iter()
            .map(|skill| MajorProficiencyData {
                kind: *skill,
                xp: proficiency_xp_for_level(
                    Stats {
                        strength: 14,
                        dexterity: 12,
                        constitution: 13,
                        intelligence: 12,
                        wisdom: 10,
                        charisma: 10,
                    }
                    .by_skill(*skill) as u32,
                ) as i32,
            })
            .collect(),
        resources: ResourcePool::full(60, 30, 30),
        proficiencies: vec![ProficiencyData {
            kind: MinorSkill::Vitality,
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

fn equipment_for(gear_name: &str) -> Equipment {
    let mut equipment = Equipment::default();
    for (slot, item) in gear_package_items(gear_name) {
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
        Difficulty::Normal,
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
        Difficulty::Normal,
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
        Difficulty::Normal,
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
        Difficulty::Normal,
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

#[test]
fn equipment_resistances_reduce_incoming_damage() {
    let character = test_character(Class::Warrior);
    let inventory: Vec<InventoryItem> = vec![];
    let unarmored = CombatState::from_character_and_encounter(
        &character,
        &Equipment::default(),
        &inventory,
        "beast_hunt",
        Difficulty::Normal,
    );
    let armored = CombatState::from_character_and_encounter(
        &character,
        &test_equipment(),
        &inventory,
        "beast_hunt",
        Difficulty::Normal,
    );

    assert!(armored.player.resistances.physical > 0);

    let raw_damage = 10;
    let unarmored_damage = CombatState::apply_resistance_to_target(
        raw_damage,
        unarmored.player.resistances,
        DamageType::Physical,
    );
    let armored_damage = CombatState::apply_resistance_to_target(
        raw_damage,
        armored.player.resistances,
        DamageType::Physical,
    );

    assert!(armored_damage < unarmored_damage);
}

#[test]
fn spell_abilities_spend_mana() {
    let mut character = test_character(Class::Mage);
    character.known_abilities = vec![KnownAbility {
        ability_id: "ember_burst".to_string(),
        rank: 1,
        unlocked: true,
        cooldown_remaining: 0,
    }];
    let mut combat = CombatState::from_character_and_encounter(
        &character,
        &equipment_for("Arcane Kit"),
        &[],
        "beast_hunt",
        Difficulty::Normal,
    );
    force_player_turn(&mut combat);
    combat.action_tab = ActionTab::Ability;
    for enemy in &mut combat.enemies {
        enemy.defense = 1;
    }

    let mana_before = combat.player.resources.mana;
    let _ = combat.resolve_player_action(PlayerAction::UseAbility);

    assert!(combat.player.resources.mana < mana_before);
}

#[test]
fn magic_weapon_attacks_spend_mana() {
    let mut character = test_character(Class::Mage);
    character.known_abilities.clear();
    let mut combat = CombatState::from_character_and_encounter(
        &character,
        &equipment_for("Arcane Kit"),
        &[],
        "beast_hunt",
        Difficulty::Normal,
    );
    force_player_turn(&mut combat);
    combat.action_tab = ActionTab::Weapon;
    for enemy in &mut combat.enemies {
        enemy.defense = 1;
    }

    let mana_before = combat.player.resources.mana;
    let _ = combat.resolve_player_action(PlayerAction::UseWeapon);

    assert!(combat.player.resources.mana < mana_before);
}

#[test]
fn combat_tab_cycle_rotates_weapon_ability_item() {
    let mut combat = CombatState::from_character_and_encounter(
        &test_character(Class::Warrior),
        &test_equipment(),
        &[],
        "beast_hunt",
        Difficulty::Normal,
    );

    assert_eq!(combat.action_tab, ActionTab::Weapon);
    combat.cycle_tab(1);
    assert_eq!(combat.action_tab, ActionTab::Ability);
    combat.cycle_tab(1);
    assert_eq!(combat.action_tab, ActionTab::Item);
    combat.cycle_tab(1);
    assert_eq!(combat.action_tab, ActionTab::Weapon);
}

#[test]
fn difficulty_scales_enemy_snapshots_without_changing_player_snapshot() {
    let character = test_character(Class::Warrior);
    let equipment = test_equipment();
    let inventory = vec![];

    let easy = CombatState::from_character_and_encounter(
        &character,
        &equipment,
        &inventory,
        "beast_hunt",
        Difficulty::Easy,
    );
    let normal = CombatState::from_character_and_encounter(
        &character,
        &equipment,
        &inventory,
        "beast_hunt",
        Difficulty::Normal,
    );
    let hard = CombatState::from_character_and_encounter(
        &character,
        &equipment,
        &inventory,
        "beast_hunt",
        Difficulty::Hard,
    );

    assert_eq!(easy.player.resources, normal.player.resources);
    assert_eq!(normal.player.resources, hard.player.resources);
    assert_eq!(easy.player.attack_bonus, normal.player.attack_bonus);
    assert_eq!(normal.player.attack_bonus, hard.player.attack_bonus);
    assert_eq!(easy.player.defense, normal.player.defense);
    assert_eq!(normal.player.defense, hard.player.defense);

    let easy_enemy = &easy.enemies[0];
    let normal_enemy = &normal.enemies[0];
    let hard_enemy = &hard.enemies[0];

    assert!(easy_enemy.resources.max_hp < normal_enemy.resources.max_hp);
    assert!(normal_enemy.resources.max_hp < hard_enemy.resources.max_hp);
    assert!(easy_enemy.attack_bonus < normal_enemy.attack_bonus);
    assert!(normal_enemy.attack_bonus < hard_enemy.attack_bonus);
    assert!(easy_enemy.defense <= normal_enemy.defense);
    assert!(normal_enemy.defense <= hard_enemy.defense);
    assert!(easy_enemy.dodge <= normal_enemy.dodge);
    assert!(normal_enemy.dodge <= hard_enemy.dodge);
}
