use crate::character::ResistanceProfile;
use crate::world::VendorId;

#[derive(Debug, Clone, Copy)]
pub struct AttackOption {
    pub name: &'static str,
    pub accuracy_bonus: i32,
    pub min_damage: i32,
    pub max_damage: i32,
}

impl AttackOption {
    pub fn damage_range_label(self) -> String {
        format!("{}-{}", self.min_damage, self.max_damage)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponKind {
    Melee,
    Ranged,
    Magic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipSlot {
    Weapon,
    Shield,
    Head,
    Neck,
    Chest,
    Cape,
    Hands,
    Ring,
    Legs,
    Feet,
}

impl EquipSlot {
    pub const ALL: [EquipSlot; 10] = [
        EquipSlot::Weapon,
        EquipSlot::Shield,
        EquipSlot::Head,
        EquipSlot::Neck,
        EquipSlot::Chest,
        EquipSlot::Cape,
        EquipSlot::Hands,
        EquipSlot::Ring,
        EquipSlot::Legs,
        EquipSlot::Feet,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Weapon => "Weapon",
            Self::Shield => "Shield",
            Self::Head => "Head",
            Self::Neck => "Neck",
            Self::Chest => "Chest",
            Self::Cape => "Cape",
            Self::Hands => "Hands",
            Self::Ring => "Ring",
            Self::Legs => "Legs",
            Self::Feet => "Feet",
        }
    }

    pub fn db_key(self) -> &'static str {
        match self {
            Self::Weapon => "weapon",
            Self::Shield => "shield",
            Self::Head => "head",
            Self::Neck => "neck",
            Self::Chest => "chest",
            Self::Cape => "cape",
            Self::Hands => "hands",
            Self::Ring => "ring",
            Self::Legs => "legs",
            Self::Feet => "feet",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Consumable,
    Equipment,
    Loot,
    Quest,
}

impl ItemKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Consumable => "Consumable",
            Self::Equipment => "Equipment",
            Self::Loot => "Loot",
            Self::Quest => "Quest Item",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
}

impl ItemRarity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Common => "Common",
            Self::Uncommon => "Uncommon",
            Self::Rare => "Rare",
            Self::Epic => "Epic",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemEffect {
    HealHp(i32),
    RestoreMana(i32),
    RestoreStamina(i32),
    CurePoison,
    ApplyGuard(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EquipmentStats {
    pub armor: i32,
    pub attack_bonus: i32,
    pub spell_power: i32,
    pub crit_bonus: i32,
    pub initiative_bonus: i32,
    pub resistances: ResistanceProfile,
}

const ZERO_RESISTANCES: ResistanceProfile = ResistanceProfile {
    physical: 0,
    fire: 0,
    frost: 0,
    lightning: 0,
    poison: 0,
    holy: 0,
    shadow: 0,
};

const ZERO_EQUIPMENT_STATS: EquipmentStats = EquipmentStats {
    armor: 0,
    attack_bonus: 0,
    spell_power: 0,
    crit_bonus: 0,
    initiative_bonus: 0,
    resistances: ZERO_RESISTANCES,
};

#[derive(Debug, Clone, Copy)]
pub struct LootTableEntry {
    pub item_type: &'static str,
    pub min_qty: i32,
    pub max_qty: i32,
    pub weight: i32,
    pub min_rarity: ItemRarity,
}

#[derive(Debug, Clone)]
pub struct VendorInventory {
    pub vendor_id: VendorId,
    pub mode_buy: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ItemDef {
    pub item_type: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub kind: ItemKind,
    pub rarity: ItemRarity,
    pub base_value: i32,
    pub stackable: bool,
    pub effects: &'static [ItemEffect],
    pub equip_slot: Option<EquipSlot>,
    pub weapon_kind: Option<WeaponKind>,
    pub attacks: &'static [AttackOption],
    pub equipment_stats: EquipmentStats,
}

impl ItemDef {
    pub fn is_usable(&self) -> bool {
        !self.effects.is_empty()
    }

    pub fn is_equippable(&self) -> bool {
        self.equip_slot.is_some()
    }

    pub fn combat_role_label(&self) -> &'static str {
        match self.weapon_kind {
            Some(WeaponKind::Melee) => "Melee Weapon",
            Some(WeaponKind::Ranged) => "Ranged Weapon",
            Some(WeaponKind::Magic) => "Spell Focus",
            None => self.kind.label(),
        }
    }
}

const IRON_SWORD_ATTACKS: &[AttackOption] = &[
    AttackOption {
        name: "Slash",
        accuracy_bonus: 2,
        min_damage: 5,
        max_damage: 9,
    },
    AttackOption {
        name: "Lunge",
        accuracy_bonus: 1,
        min_damage: 7,
        max_damage: 11,
    },
];

const TWIN_DAGGER_ATTACKS: &[AttackOption] = &[
    AttackOption {
        name: "Quick Slash",
        accuracy_bonus: 3,
        min_damage: 4,
        max_damage: 7,
    },
    AttackOption {
        name: "Backstab",
        accuracy_bonus: 1,
        min_damage: 6,
        max_damage: 10,
    },
];

const BOW_ATTACKS: &[AttackOption] = &[
    AttackOption {
        name: "Quick Shot",
        accuracy_bonus: 3,
        min_damage: 4,
        max_damage: 8,
    },
    AttackOption {
        name: "Power Draw",
        accuracy_bonus: 1,
        min_damage: 6,
        max_damage: 10,
    },
];

const STAFF_ATTACKS: &[AttackOption] = &[
    AttackOption {
        name: "Arcane Bolt",
        accuracy_bonus: 2,
        min_damage: 5,
        max_damage: 9,
    },
    AttackOption {
        name: "Frost Tap",
        accuracy_bonus: 2,
        min_damage: 4,
        max_damage: 8,
    },
];

const WAND_ATTACKS: &[AttackOption] = &[
    AttackOption {
        name: "Fire Spark",
        accuracy_bonus: 2,
        min_damage: 5,
        max_damage: 10,
    },
    AttackOption {
        name: "Ember Lance",
        accuracy_bonus: 1,
        min_damage: 7,
        max_damage: 11,
    },
];

const HEALTH_EFFECT: &[ItemEffect] = &[ItemEffect::HealHp(24)];
const BANDAGE_EFFECT: &[ItemEffect] = &[ItemEffect::HealHp(12)];
const RATION_EFFECT: &[ItemEffect] = &[ItemEffect::HealHp(8), ItemEffect::RestoreStamina(6)];
const MANA_EFFECT: &[ItemEffect] = &[ItemEffect::RestoreMana(18)];
const STAMINA_EFFECT: &[ItemEffect] = &[ItemEffect::RestoreStamina(18)];
const ANTIDOTE_EFFECT: &[ItemEffect] = &[ItemEffect::CurePoison];

pub const ITEM_CATALOG: &[ItemDef] = &[
    ItemDef {
        item_type: "health_potion",
        name: "Health Potion",
        description: "A bright red tonic that closes cuts and steadies breathing.",
        kind: ItemKind::Consumable,
        rarity: ItemRarity::Common,
        base_value: 18,
        stackable: true,
        effects: HEALTH_EFFECT,
        equip_slot: None,
        weapon_kind: None,
        attacks: &[],
        equipment_stats: ZERO_EQUIPMENT_STATS,
    },
    ItemDef {
        item_type: "bandage",
        name: "Bandage",
        description: "Quick field wrap for patching lighter injuries.",
        kind: ItemKind::Consumable,
        rarity: ItemRarity::Common,
        base_value: 8,
        stackable: true,
        effects: BANDAGE_EFFECT,
        equip_slot: None,
        weapon_kind: None,
        attacks: &[],
        equipment_stats: ZERO_EQUIPMENT_STATS,
    },
    ItemDef {
        item_type: "ration",
        name: "Ration",
        description: "Salt meat, hard bread, and enough calories to keep moving.",
        kind: ItemKind::Consumable,
        rarity: ItemRarity::Common,
        base_value: 6,
        stackable: true,
        effects: RATION_EFFECT,
        equip_slot: None,
        weapon_kind: None,
        attacks: &[],
        equipment_stats: ZERO_EQUIPMENT_STATS,
    },
    ItemDef {
        item_type: "mana_tonic",
        name: "Mana Tonic",
        description: "A bitter draught that restores spell reserve.",
        kind: ItemKind::Consumable,
        rarity: ItemRarity::Uncommon,
        base_value: 22,
        stackable: true,
        effects: MANA_EFFECT,
        equip_slot: None,
        weapon_kind: None,
        attacks: &[],
        equipment_stats: ZERO_EQUIPMENT_STATS,
    },
    ItemDef {
        item_type: "stamina_draught",
        name: "Stamina Draught",
        description: "Restores drive and clears the ache from tired limbs.",
        kind: ItemKind::Consumable,
        rarity: ItemRarity::Uncommon,
        base_value: 20,
        stackable: true,
        effects: STAMINA_EFFECT,
        equip_slot: None,
        weapon_kind: None,
        attacks: &[],
        equipment_stats: ZERO_EQUIPMENT_STATS,
    },
    ItemDef {
        item_type: "antidote",
        name: "Antidote",
        description: "Neutralizes common toxins and clears poison.",
        kind: ItemKind::Consumable,
        rarity: ItemRarity::Common,
        base_value: 14,
        stackable: true,
        effects: ANTIDOTE_EFFECT,
        equip_slot: None,
        weapon_kind: None,
        attacks: &[],
        equipment_stats: ZERO_EQUIPMENT_STATS,
    },
    ItemDef {
        item_type: "iron_sword",
        name: "Iron Sword",
        description: "Reliable steel favored by militia and caravan guards.",
        kind: ItemKind::Equipment,
        rarity: ItemRarity::Common,
        base_value: 42,
        stackable: false,
        effects: &[],
        equip_slot: Some(EquipSlot::Weapon),
        weapon_kind: Some(WeaponKind::Melee),
        attacks: IRON_SWORD_ATTACKS,
        equipment_stats: EquipmentStats {
            armor: 0,
            attack_bonus: 2,
            spell_power: 0,
            crit_bonus: 0,
            initiative_bonus: 0,
            resistances: ResistanceProfile {
                physical: 0,
                fire: 0,
                frost: 0,
                lightning: 0,
                poison: 0,
                holy: 0,
                shadow: 0,
            },
        },
    },
    ItemDef {
        item_type: "twin_daggers",
        name: "Twin Daggers",
        description: "Light blades built for precise openings and dirty work.",
        kind: ItemKind::Equipment,
        rarity: ItemRarity::Uncommon,
        base_value: 55,
        stackable: false,
        effects: &[],
        equip_slot: Some(EquipSlot::Weapon),
        weapon_kind: Some(WeaponKind::Melee),
        attacks: TWIN_DAGGER_ATTACKS,
        equipment_stats: EquipmentStats {
            armor: 0,
            attack_bonus: 1,
            spell_power: 0,
            crit_bonus: 3,
            initiative_bonus: 2,
            resistances: ResistanceProfile {
                physical: 0,
                fire: 0,
                frost: 0,
                lightning: 0,
                poison: 0,
                holy: 0,
                shadow: 0,
            },
        },
    },
    ItemDef {
        item_type: "hunting_bow",
        name: "Hunting Bow",
        description: "Yew bow suited to measured pressure at range.",
        kind: ItemKind::Equipment,
        rarity: ItemRarity::Common,
        base_value: 48,
        stackable: false,
        effects: &[],
        equip_slot: Some(EquipSlot::Weapon),
        weapon_kind: Some(WeaponKind::Ranged),
        attacks: BOW_ATTACKS,
        equipment_stats: EquipmentStats {
            armor: 0,
            attack_bonus: 2,
            spell_power: 0,
            crit_bonus: 1,
            initiative_bonus: 1,
            resistances: ZERO_RESISTANCES,
        },
    },
    ItemDef {
        item_type: "apprentice_staff",
        name: "Apprentice Staff",
        description: "A runed staff that steadies novice channeling.",
        kind: ItemKind::Equipment,
        rarity: ItemRarity::Common,
        base_value: 50,
        stackable: false,
        effects: &[],
        equip_slot: Some(EquipSlot::Weapon),
        weapon_kind: Some(WeaponKind::Magic),
        attacks: STAFF_ATTACKS,
        equipment_stats: EquipmentStats {
            armor: 0,
            attack_bonus: 1,
            spell_power: 3,
            crit_bonus: 0,
            initiative_bonus: 0,
            resistances: ZERO_RESISTANCES,
        },
    },
    ItemDef {
        item_type: "ember_wand",
        name: "Ember Wand",
        description: "A refined focus for aggressive elemental casting.",
        kind: ItemKind::Equipment,
        rarity: ItemRarity::Rare,
        base_value: 88,
        stackable: false,
        effects: &[],
        equip_slot: Some(EquipSlot::Weapon),
        weapon_kind: Some(WeaponKind::Magic),
        attacks: WAND_ATTACKS,
        equipment_stats: EquipmentStats {
            armor: 0,
            attack_bonus: 1,
            spell_power: 5,
            crit_bonus: 1,
            initiative_bonus: 1,
            resistances: ZERO_RESISTANCES,
        },
    },
    ItemDef {
        item_type: "wooden_shield",
        name: "Wooden Shield",
        description: "Simple protection that keeps the worst of a blow off your ribs.",
        kind: ItemKind::Equipment,
        rarity: ItemRarity::Common,
        base_value: 28,
        stackable: false,
        effects: &[],
        equip_slot: Some(EquipSlot::Shield),
        weapon_kind: None,
        attacks: &[],
        equipment_stats: EquipmentStats {
            armor: 3,
            attack_bonus: 0,
            spell_power: 0,
            crit_bonus: 0,
            initiative_bonus: 0,
            resistances: ResistanceProfile {
                physical: 1,
                ..ZERO_RESISTANCES
            },
        },
    },
    ItemDef {
        item_type: "leather_chest",
        name: "Leather Jerkin",
        description: "Sturdy boiled leather with enough give for active fighting.",
        kind: ItemKind::Equipment,
        rarity: ItemRarity::Common,
        base_value: 32,
        stackable: false,
        effects: &[],
        equip_slot: Some(EquipSlot::Chest),
        weapon_kind: None,
        attacks: &[],
        equipment_stats: EquipmentStats {
            armor: 4,
            attack_bonus: 0,
            spell_power: 0,
            crit_bonus: 0,
            initiative_bonus: 0,
            resistances: ResistanceProfile {
                physical: 1,
                ..ZERO_RESISTANCES
            },
        },
    },
    ItemDef {
        item_type: "leather_legs",
        name: "Leather Greaves",
        description: "Flexible leg guards for long marches and quick turns.",
        kind: ItemKind::Equipment,
        rarity: ItemRarity::Common,
        base_value: 24,
        stackable: false,
        effects: &[],
        equip_slot: Some(EquipSlot::Legs),
        weapon_kind: None,
        attacks: &[],
        equipment_stats: EquipmentStats {
            armor: 2,
            attack_bonus: 0,
            spell_power: 0,
            crit_bonus: 0,
            initiative_bonus: 1,
            resistances: ZERO_RESISTANCES,
        },
    },
    ItemDef {
        item_type: "traveler_cloak",
        name: "Traveler's Cloak",
        description: "Wool and weather-proofing stitched for hard roads.",
        kind: ItemKind::Equipment,
        rarity: ItemRarity::Common,
        base_value: 20,
        stackable: false,
        effects: &[],
        equip_slot: Some(EquipSlot::Cape),
        weapon_kind: None,
        attacks: &[],
        equipment_stats: EquipmentStats {
            armor: 1,
            attack_bonus: 0,
            spell_power: 0,
            crit_bonus: 0,
            initiative_bonus: 1,
            resistances: ResistanceProfile {
                frost: 1,
                ..ZERO_RESISTANCES
            },
        },
    },
    ItemDef {
        item_type: "silver_amulet",
        name: "Silver Amulet",
        description: "An old protective charm used to ward restless spirits.",
        kind: ItemKind::Equipment,
        rarity: ItemRarity::Uncommon,
        base_value: 60,
        stackable: false,
        effects: &[],
        equip_slot: Some(EquipSlot::Neck),
        weapon_kind: None,
        attacks: &[],
        equipment_stats: EquipmentStats {
            armor: 0,
            attack_bonus: 0,
            spell_power: 2,
            crit_bonus: 0,
            initiative_bonus: 0,
            resistances: ResistanceProfile {
                holy: 2,
                shadow: 1,
                ..ZERO_RESISTANCES
            },
        },
    },
    ItemDef {
        item_type: "wolf_pelt",
        name: "Wolf Pelt",
        description: "A thick hide that sells well to fur traders.",
        kind: ItemKind::Loot,
        rarity: ItemRarity::Common,
        base_value: 12,
        stackable: true,
        effects: &[],
        equip_slot: None,
        weapon_kind: None,
        attacks: &[],
        equipment_stats: ZERO_EQUIPMENT_STATS,
    },
    ItemDef {
        item_type: "bandit_seal",
        name: "Bandit Seal",
        description: "Stamped insignia stolen from raider captains.",
        kind: ItemKind::Loot,
        rarity: ItemRarity::Uncommon,
        base_value: 20,
        stackable: true,
        effects: &[],
        equip_slot: None,
        weapon_kind: None,
        attacks: &[],
        equipment_stats: ZERO_EQUIPMENT_STATS,
    },
    ItemDef {
        item_type: "grave_ash",
        name: "Grave Ash",
        description: "Powder scraped from an active barrow's inner crypt.",
        kind: ItemKind::Loot,
        rarity: ItemRarity::Rare,
        base_value: 26,
        stackable: true,
        effects: &[],
        equip_slot: None,
        weapon_kind: None,
        attacks: &[],
        equipment_stats: ZERO_EQUIPMENT_STATS,
    },
    ItemDef {
        item_type: "old_map",
        name: "Old Map",
        description: "A courier chart detailing paths across the Sunken Road.",
        kind: ItemKind::Quest,
        rarity: ItemRarity::Uncommon,
        base_value: 0,
        stackable: true,
        effects: &[],
        equip_slot: None,
        weapon_kind: None,
        attacks: &[],
        equipment_stats: ZERO_EQUIPMENT_STATS,
    },
];

pub fn find_def(item_type: &str) -> Option<&'static ItemDef> {
    ITEM_CATALOG.iter().find(|item| item.item_type == item_type)
}

pub fn gear_package_items(gear_name: &str) -> &'static [(&'static str, &'static str)] {
    match gear_name {
        "Melee Kit" => &[
            ("weapon", "iron_sword"),
            ("shield", "wooden_shield"),
            ("chest", "leather_chest"),
            ("legs", "leather_legs"),
        ],
        "Ranged Kit" => &[
            ("weapon", "hunting_bow"),
            ("cape", "traveler_cloak"),
            ("chest", "leather_chest"),
            ("legs", "leather_legs"),
        ],
        "Arcane Kit" => &[
            ("weapon", "apprentice_staff"),
            ("neck", "silver_amulet"),
            ("cape", "traveler_cloak"),
        ],
        "Stealth Kit" => &[
            ("weapon", "twin_daggers"),
            ("cape", "traveler_cloak"),
            ("chest", "leather_chest"),
            ("legs", "leather_legs"),
        ],
        _ => &[],
    }
}

#[derive(Debug, Clone, Default)]
pub struct Equipment {
    pub weapon: Option<String>,
    pub shield: Option<String>,
    pub head: Option<String>,
    pub neck: Option<String>,
    pub chest: Option<String>,
    pub cape: Option<String>,
    pub hands: Option<String>,
    pub ring: Option<String>,
    pub legs: Option<String>,
    pub feet: Option<String>,
}

impl Equipment {
    pub fn get_slot(&self, slot: EquipSlot) -> Option<&str> {
        match slot {
            EquipSlot::Weapon => self.weapon.as_deref(),
            EquipSlot::Shield => self.shield.as_deref(),
            EquipSlot::Head => self.head.as_deref(),
            EquipSlot::Neck => self.neck.as_deref(),
            EquipSlot::Chest => self.chest.as_deref(),
            EquipSlot::Cape => self.cape.as_deref(),
            EquipSlot::Hands => self.hands.as_deref(),
            EquipSlot::Ring => self.ring.as_deref(),
            EquipSlot::Legs => self.legs.as_deref(),
            EquipSlot::Feet => self.feet.as_deref(),
        }
    }

    pub fn set_slot(&mut self, slot: EquipSlot, item: Option<String>) {
        match slot {
            EquipSlot::Weapon => self.weapon = item,
            EquipSlot::Shield => self.shield = item,
            EquipSlot::Head => self.head = item,
            EquipSlot::Neck => self.neck = item,
            EquipSlot::Chest => self.chest = item,
            EquipSlot::Cape => self.cape = item,
            EquipSlot::Hands => self.hands = item,
            EquipSlot::Ring => self.ring = item,
            EquipSlot::Legs => self.legs = item,
            EquipSlot::Feet => self.feet = item,
        }
    }

    pub fn equipped_defs(&self) -> Vec<&'static ItemDef> {
        EquipSlot::ALL
            .iter()
            .filter_map(|slot| self.get_slot(*slot))
            .filter_map(find_def)
            .collect()
    }

    pub fn total_equipment_stats(&self) -> EquipmentStats {
        let mut total = EquipmentStats::default();
        for def in self.equipped_defs() {
            total.armor += def.equipment_stats.armor;
            total.attack_bonus += def.equipment_stats.attack_bonus;
            total.spell_power += def.equipment_stats.spell_power;
            total.crit_bonus += def.equipment_stats.crit_bonus;
            total.initiative_bonus += def.equipment_stats.initiative_bonus;
            total.resistances.physical += def.equipment_stats.resistances.physical;
            total.resistances.fire += def.equipment_stats.resistances.fire;
            total.resistances.frost += def.equipment_stats.resistances.frost;
            total.resistances.lightning += def.equipment_stats.resistances.lightning;
            total.resistances.poison += def.equipment_stats.resistances.poison;
            total.resistances.holy += def.equipment_stats.resistances.holy;
            total.resistances.shadow += def.equipment_stats.resistances.shadow;
        }
        total
    }

    pub fn total_armor_bonus(&self) -> i32 {
        self.total_equipment_stats().armor
    }

    pub fn attack_options(&self) -> Vec<AttackOption> {
        self.weapon
            .as_deref()
            .and_then(find_def)
            .map(|def| def.attacks.to_vec())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub item_type: String,
    pub quantity: i32,
}

impl InventoryItem {
    pub fn def(&self) -> Option<&'static ItemDef> {
        find_def(&self.item_type)
    }

    pub fn value_each(&self) -> i32 {
        self.def().map(|def| def.base_value).unwrap_or(0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct InventoryState {
    pub items: Vec<InventoryItem>,
    pub cursor: usize,
    pub last_use_message: Option<String>,
}

impl InventoryState {
    pub fn selected(&self) -> Option<&InventoryItem> {
        self.items.get(self.cursor)
    }

    pub fn selected_def(&self) -> Option<&'static ItemDef> {
        self.selected().and_then(|item| item.def())
    }

    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn cursor_down(&mut self) {
        if !self.items.is_empty() && self.cursor + 1 < self.items.len() {
            self.cursor += 1;
        }
    }

    pub fn clamp_cursor(&mut self) {
        if self.items.is_empty() {
            self.cursor = 0;
        } else if self.cursor >= self.items.len() {
            self.cursor = self.items.len() - 1;
        }
    }
}
