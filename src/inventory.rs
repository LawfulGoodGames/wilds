// ── Attack option (shared with combat) ───────────────────────────────────────

/// A single attack move granted by an equipped weapon.
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

// ── Weapon / equipment types ─────────────────────────────────────────────────

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
            Self::Head   => "Head",
            Self::Neck   => "Neck",
            Self::Chest  => "Chest",
            Self::Cape   => "Cape",
            Self::Hands  => "Hands",
            Self::Ring   => "Ring",
            Self::Legs   => "Legs",
            Self::Feet   => "Feet",
        }
    }

    pub fn db_key(self) -> &'static str {
        match self {
            Self::Weapon => "weapon",
            Self::Shield => "shield",
            Self::Head   => "head",
            Self::Neck   => "neck",
            Self::Chest  => "chest",
            Self::Cape   => "cape",
            Self::Hands  => "hands",
            Self::Ring   => "ring",
            Self::Legs   => "legs",
            Self::Feet   => "feet",
        }
    }
}

// ── Item kind ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Consumable,
    Equipment,
    Quest,
}

impl ItemKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Consumable => "Consumable",
            Self::Equipment  => "Equipment",
            Self::Quest      => "Quest Item",
        }
    }
}

// ── Item catalog definition ───────────────────────────────────────────────────

pub struct ItemDef {
    pub item_type:     &'static str,
    pub name:          &'static str,
    pub description:   &'static str,
    pub kind:          ItemKind,
    /// HP restored when used (0 = no direct heal).
    pub heal_amount:   i32,
    /// Which equipment slot this occupies, if any.
    pub equip_slot:    Option<EquipSlot>,
    /// For weapons: whether attacks are melee / ranged / magic.
    pub weapon_kind:   Option<WeaponKind>,
    /// Attacks unlocked by this weapon when equipped.
    pub attacks:       &'static [AttackOption],
    /// Passive defense bonus while equipped.
    pub defense_bonus: i32,
}

impl ItemDef {
    pub fn is_usable(&self) -> bool {
        self.kind == ItemKind::Consumable
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

// ── Weapon attack tables ──────────────────────────────────────────────────────

const IRON_SWORD_ATTACKS: &[AttackOption] = &[
    AttackOption { name: "Slash",  accuracy_bonus: 6, min_damage: 4, max_damage: 9 },
    AttackOption { name: "Lunge",  accuracy_bonus: 4, min_damage: 6, max_damage: 11 },
];
const TWIN_DAGGERS_ATTACKS: &[AttackOption] = &[
    AttackOption { name: "Quick Slash", accuracy_bonus: 10, min_damage: 3, max_damage: 7 },
    AttackOption { name: "Backstab",    accuracy_bonus: 5,  min_damage: 6, max_damage: 10 },
];
const BATTLE_AXE_ATTACKS: &[AttackOption] = &[
    AttackOption { name: "Heavy Blow", accuracy_bonus: 3, min_damage: 8, max_damage: 15 },
    AttackOption { name: "Cleave",     accuracy_bonus: 5, min_damage: 6, max_damage: 12 },
];
const WAR_HAMMER_ATTACKS: &[AttackOption] = &[
    AttackOption { name: "Crush",          accuracy_bonus: 4, min_damage: 7,  max_damage: 14 },
    AttackOption { name: "Overhead Smash", accuracy_bonus: 2, min_damage: 10, max_damage: 16 },
];
const HUNTING_BOW_ATTACKS: &[AttackOption] = &[
    AttackOption { name: "Quick Shot",  accuracy_bonus: 7, min_damage: 4, max_damage: 8 },
    AttackOption { name: "Power Draw",  accuracy_bonus: 3, min_damage: 6, max_damage: 11 },
];
const THROWING_KNIVES_ATTACKS: &[AttackOption] = &[
    AttackOption { name: "Flurry",        accuracy_bonus: 11, min_damage: 2, max_damage: 6 },
    AttackOption { name: "Precise Throw", accuracy_bonus: 6,  min_damage: 4, max_damage: 9 },
];
const CROSSBOW_ATTACKS: &[AttackOption] = &[
    AttackOption { name: "Bolt Shot",  accuracy_bonus: 9, min_damage: 5, max_damage: 10 },
    AttackOption { name: "Rapid Fire", accuracy_bonus: 6, min_damage: 3, max_damage: 8 },
];
const APPRENTICE_STAFF_ATTACKS: &[AttackOption] = &[
    AttackOption { name: "Arcane Bolt", accuracy_bonus: 7, min_damage: 5, max_damage: 10 },
    AttackOption { name: "Minor Frost", accuracy_bonus: 6, min_damage: 4, max_damage: 8 },
];
const EMBER_WAND_ATTACKS: &[AttackOption] = &[
    AttackOption { name: "Ember Lance", accuracy_bonus: 5, min_damage: 6, max_damage: 12 },
    AttackOption { name: "Fire Spark",  accuracy_bonus: 8, min_damage: 4, max_damage: 9 },
];
const STORM_ORB_ATTACKS: &[AttackOption] = &[
    AttackOption { name: "Lightning Bolt", accuracy_bonus: 6, min_damage: 7,  max_damage: 13 },
    AttackOption { name: "Chain Strike",   accuracy_bonus: 4, min_damage: 8,  max_damage: 14 },
];

// ── Item catalog ──────────────────────────────────────────────────────────────

pub const ITEM_CATALOG: &[ItemDef] = &[
    // ── Consumables ──────────────────────────────────────────────────────────
    ItemDef {
        item_type: "health_potion", name: "Health Potion",
        description: "A vial of crimson liquid brewed from mountain herbs. Restores 20 HP.",
        kind: ItemKind::Consumable, heal_amount: 20,
        equip_slot: None, weapon_kind: None, attacks: &[], defense_bonus: 0,
    },
    ItemDef {
        item_type: "bandage", name: "Bandage",
        description: "Strips of clean linen. Useful in a pinch. Restores 8 HP.",
        kind: ItemKind::Consumable, heal_amount: 8,
        equip_slot: None, weapon_kind: None, attacks: &[], defense_bonus: 0,
    },
    ItemDef {
        item_type: "antidote", name: "Antidote",
        description: "A bitter green tonic. Neutralises common poisons.",
        kind: ItemKind::Consumable, heal_amount: 0,
        equip_slot: None, weapon_kind: None, attacks: &[], defense_bonus: 0,
    },
    ItemDef {
        item_type: "ration", name: "Ration",
        description: "Hard bread and dried meat — reliable trail food. Restores 5 HP.",
        kind: ItemKind::Consumable, heal_amount: 5,
        equip_slot: None, weapon_kind: None, attacks: &[], defense_bonus: 0,
    },
    // ── Melee weapons ────────────────────────────────────────────────────────
    ItemDef {
        item_type: "iron_sword", name: "Iron Sword",
        description: "A reliable blade of tempered iron. Unlocks Slash and Lunge.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Weapon), weapon_kind: Some(WeaponKind::Melee),
        attacks: IRON_SWORD_ATTACKS, defense_bonus: 0,
    },
    ItemDef {
        item_type: "twin_daggers", name: "Twin Daggers",
        description: "Nimble paired blades favoured by rogues. Unlocks Quick Slash and Backstab.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Weapon), weapon_kind: Some(WeaponKind::Melee),
        attacks: TWIN_DAGGERS_ATTACKS, defense_bonus: 0,
    },
    ItemDef {
        item_type: "battle_axe", name: "Battle Axe",
        description: "A heavy two-handed axe. Unlocks Heavy Blow and Cleave.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Weapon), weapon_kind: Some(WeaponKind::Melee),
        attacks: BATTLE_AXE_ATTACKS, defense_bonus: 0,
    },
    ItemDef {
        item_type: "war_hammer", name: "War Hammer",
        description: "A crushing weapon of iron and oak. Unlocks Crush and Overhead Smash.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Weapon), weapon_kind: Some(WeaponKind::Melee),
        attacks: WAR_HAMMER_ATTACKS, defense_bonus: 0,
    },
    // ── Ranged weapons ───────────────────────────────────────────────────────
    ItemDef {
        item_type: "hunting_bow", name: "Hunting Bow",
        description: "A carved longbow of yew. Unlocks Quick Shot and Power Draw.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Weapon), weapon_kind: Some(WeaponKind::Ranged),
        attacks: HUNTING_BOW_ATTACKS, defense_bonus: 0,
    },
    ItemDef {
        item_type: "throwing_knives", name: "Throwing Knives",
        description: "A set of weighted throwing blades. Unlocks Flurry and Precise Throw.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Weapon), weapon_kind: Some(WeaponKind::Ranged),
        attacks: THROWING_KNIVES_ATTACKS, defense_bonus: 0,
    },
    ItemDef {
        item_type: "crossbow", name: "Crossbow",
        description: "A mechanical ranged weapon of great precision. Unlocks Bolt Shot and Rapid Fire.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Weapon), weapon_kind: Some(WeaponKind::Ranged),
        attacks: CROSSBOW_ATTACKS, defense_bonus: 0,
    },
    // ── Magic weapons ────────────────────────────────────────────────────────
    ItemDef {
        item_type: "apprentice_staff", name: "Apprentice Staff",
        description: "A basic mage's staff etched with runes. Unlocks Arcane Bolt and Minor Frost.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Weapon), weapon_kind: Some(WeaponKind::Magic),
        attacks: APPRENTICE_STAFF_ATTACKS, defense_bonus: 0,
    },
    ItemDef {
        item_type: "ember_wand", name: "Ember Wand",
        description: "A wand imbued with fire essence. Unlocks Ember Lance and Fire Spark.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Weapon), weapon_kind: Some(WeaponKind::Magic),
        attacks: EMBER_WAND_ATTACKS, defense_bonus: 0,
    },
    ItemDef {
        item_type: "storm_orb", name: "Storm Orb",
        description: "A crackling orb of captured lightning. Unlocks Lightning Bolt and Chain Strike.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Weapon), weapon_kind: Some(WeaponKind::Magic),
        attacks: STORM_ORB_ATTACKS, defense_bonus: 0,
    },
    // ── Armor ────────────────────────────────────────────────────────────────
    ItemDef {
        item_type: "leather_helm", name: "Leather Helm",
        description: "A sturdy leather cap offering modest head protection.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Head), weapon_kind: None, attacks: &[], defense_bonus: 2,
    },
    ItemDef {
        item_type: "leather_chest", name: "Leather Chest",
        description: "A fitted leather chest piece. Standard adventurer's fare.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Chest), weapon_kind: None, attacks: &[], defense_bonus: 4,
    },
    ItemDef {
        item_type: "leather_legs", name: "Leather Legs",
        description: "Leather-reinforced leg guards. Light and flexible.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Legs), weapon_kind: None, attacks: &[], defense_bonus: 3,
    },
    ItemDef {
        item_type: "leather_boots", name: "Leather Boots",
        description: "Worn but reliable boots with a thick sole.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Feet), weapon_kind: None, attacks: &[], defense_bonus: 1,
    },
    ItemDef {
        item_type: "leather_gloves", name: "Leather Gloves",
        description: "Fingerless leather gloves for grip and protection.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Hands), weapon_kind: None, attacks: &[], defense_bonus: 1,
    },
    ItemDef {
        item_type: "iron_shield", name: "Iron Shield",
        description: "A round shield of hammered iron. Solid defense.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Shield), weapon_kind: None, attacks: &[], defense_bonus: 6,
    },
    ItemDef {
        item_type: "wooden_shield", name: "Wooden Shield",
        description: "A reinforced wooden buckler. Light but effective.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Shield), weapon_kind: None, attacks: &[], defense_bonus: 3,
    },
    ItemDef {
        item_type: "traveler_cloak", name: "Traveler's Cloak",
        description: "A thick woolen cloak that wards off the chill.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Cape), weapon_kind: None, attacks: &[], defense_bonus: 1,
    },
    ItemDef {
        item_type: "gold_ring", name: "Gold Ring",
        description: "A simple band of gold. Worn smooth with age.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Ring), weapon_kind: None, attacks: &[], defense_bonus: 0,
    },
    ItemDef {
        item_type: "silver_amulet", name: "Silver Amulet",
        description: "An amulet etched with faint protective runes.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: Some(EquipSlot::Neck), weapon_kind: None, attacks: &[], defense_bonus: 2,
    },
    // ── Loot ─────────────────────────────────────────────────────────────────
    ItemDef {
        item_type: "wolf_pelt", name: "Wolf Pelt",
        description: "The thick pelt of a wild wolf. Might fetch a fair price.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: None, weapon_kind: None, attacks: &[], defense_bonus: 0,
    },
    ItemDef {
        item_type: "fox_pelt", name: "Fox Pelt",
        description: "A fine pelt with a russet sheen. Soft and surprisingly valuable.",
        kind: ItemKind::Equipment, heal_amount: 0,
        equip_slot: None, weapon_kind: None, attacks: &[], defense_bonus: 0,
    },
    ItemDef {
        item_type: "old_map", name: "Old Map",
        description: "A worn parchment covered in faded markings. Someone went to great lengths to hide this.",
        kind: ItemKind::Quest, heal_amount: 0,
        equip_slot: None, weapon_kind: None, attacks: &[], defense_bonus: 0,
    },
];

pub fn find_def(item_type: &str) -> Option<&'static ItemDef> {
    ITEM_CATALOG.iter().find(|d| d.item_type == item_type)
}

/// Returns the (slot_key, item_type) pairs that a gear package starts with equipped.
/// This is the single source of truth used by both the DB seeder and the creation UI.
pub fn gear_package_items(gear_name: &str) -> &'static [(&'static str, &'static str)] {
    match gear_name {
        "Melee Kit" => &[
            ("weapon", "iron_sword"),
            ("shield", "wooden_shield"),
            ("chest",  "leather_chest"),
            ("legs",   "leather_legs"),
        ],
        "Ranged Kit" => &[
            ("weapon", "hunting_bow"),
            ("cape",   "traveler_cloak"),
            ("chest",  "leather_chest"),
            ("legs",   "leather_legs"),
        ],
        "Arcane Kit" => &[
            ("weapon", "apprentice_staff"),
            ("chest",  "leather_chest"),
            ("legs",   "leather_legs"),
        ],
        "Stealth Kit" => &[
            ("weapon", "twin_daggers"),
            ("cape",   "traveler_cloak"),
            ("chest",  "leather_chest"),
            ("legs",   "leather_legs"),
        ],
        _ => &[],
    }
}

// ── Equipment state ───────────────────────────────────────────────────────────

/// Currently equipped items, keyed by slot.
#[derive(Debug, Clone, Default)]
pub struct Equipment {
    pub weapon: Option<String>,
    pub shield: Option<String>,
    pub head:   Option<String>,
    pub neck:   Option<String>,
    pub chest:  Option<String>,
    pub cape:   Option<String>,
    pub hands:  Option<String>,
    pub ring:   Option<String>,
    pub legs:   Option<String>,
    pub feet:   Option<String>,
}

impl Equipment {
    pub fn get_slot(&self, slot: EquipSlot) -> Option<&str> {
        match slot {
            EquipSlot::Weapon => self.weapon.as_deref(),
            EquipSlot::Shield => self.shield.as_deref(),
            EquipSlot::Head   => self.head.as_deref(),
            EquipSlot::Neck   => self.neck.as_deref(),
            EquipSlot::Chest  => self.chest.as_deref(),
            EquipSlot::Cape   => self.cape.as_deref(),
            EquipSlot::Hands  => self.hands.as_deref(),
            EquipSlot::Ring   => self.ring.as_deref(),
            EquipSlot::Legs   => self.legs.as_deref(),
            EquipSlot::Feet   => self.feet.as_deref(),
        }
    }

    pub fn set_slot(&mut self, slot: EquipSlot, item_type: Option<String>) {
        match slot {
            EquipSlot::Weapon => self.weapon = item_type,
            EquipSlot::Shield => self.shield = item_type,
            EquipSlot::Head   => self.head   = item_type,
            EquipSlot::Neck   => self.neck   = item_type,
            EquipSlot::Chest  => self.chest  = item_type,
            EquipSlot::Cape   => self.cape   = item_type,
            EquipSlot::Hands  => self.hands  = item_type,
            EquipSlot::Ring   => self.ring   = item_type,
            EquipSlot::Legs   => self.legs   = item_type,
            EquipSlot::Feet   => self.feet   = item_type,
        }
    }

    fn weapon_attacks_for_kind(&self, kind: WeaponKind) -> Vec<AttackOption> {
        let weapon_type = match self.weapon.as_deref() {
            Some(t) => t,
            None => return vec![],
        };
        let def = match find_def(weapon_type) {
            Some(d) => d,
            None => return vec![],
        };
        if def.weapon_kind != Some(kind) {
            return vec![];
        }
        def.attacks.to_vec()
    }

    pub fn melee_attacks(&self) -> Vec<AttackOption> {
        self.weapon_attacks_for_kind(WeaponKind::Melee)
    }

    pub fn ranged_attacks(&self) -> Vec<AttackOption> {
        self.weapon_attacks_for_kind(WeaponKind::Ranged)
    }

    pub fn magic_attacks(&self) -> Vec<AttackOption> {
        self.weapon_attacks_for_kind(WeaponKind::Magic)
    }

    /// Sum of all equipped items' armor bonuses.
    pub fn total_armor_bonus(&self) -> i32 {
        EquipSlot::ALL
            .iter()
            .filter_map(|&slot| {
                let item_type = self.get_slot(slot)?;
                Some(find_def(item_type)?.defense_bonus)
            })
            .sum()
    }

    pub fn total_defense_bonus(&self) -> i32 {
        self.total_armor_bonus()
    }
}

// ── Inventory bag state ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub item_type: String,
    pub quantity:  i32,
}

impl InventoryItem {
    pub fn def(&self) -> Option<&'static ItemDef> {
        find_def(&self.item_type)
    }
}

#[derive(Debug, Clone, Default)]
pub struct InventoryState {
    pub items:            Vec<InventoryItem>,
    pub cursor:           usize,
    pub last_use_message: Option<String>,
}

impl InventoryState {
    pub fn selected(&self) -> Option<&InventoryItem> {
        self.items.get(self.cursor)
    }

    pub fn selected_def(&self) -> Option<&'static ItemDef> {
        self.selected().and_then(|i| i.def())
    }

    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn cursor_down(&mut self) {
        if !self.items.is_empty() && self.cursor < self.items.len() - 1 {
            self.cursor += 1;
        }
    }

    pub fn clamp_cursor(&mut self) {
        if !self.items.is_empty() && self.cursor >= self.items.len() {
            self.cursor = self.items.len() - 1;
        }
    }
}
