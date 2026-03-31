mod catalog;
mod state;

use crate::character::ResistanceProfile;
use crate::world::VendorId;

pub use catalog::{ITEM_CATALOG, find_def, gear_package_items};
pub use state::{Equipment, InventoryItem, InventoryState};

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

    pub fn sort_order(self) -> u8 {
        match self {
            Self::Consumable => 0,
            Self::Equipment => 1,
            Self::Loot => 2,
            Self::Quest => 3,
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
