use super::{AttackOption, EquipSlot, EquipmentStats, ItemDef, find_def};

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
