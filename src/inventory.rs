// ── Item catalog ──────────────────────────────────────────────────────────────

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

pub struct ItemDef {
    pub item_type:   &'static str,
    pub name:        &'static str,
    pub description: &'static str,
    pub kind:        ItemKind,
    /// HP restored on use; 0 means no healing effect.
    pub heal_amount: i32,
}

impl ItemDef {
    pub fn is_usable(&self) -> bool {
        self.kind == ItemKind::Consumable
    }
}

pub const ITEM_CATALOG: &[ItemDef] = &[
    ItemDef {
        item_type:   "health_potion",
        name:        "Health Potion",
        description: "A vial of crimson liquid brewed from mountain herbs. Restores 20 HP.",
        kind:        ItemKind::Consumable,
        heal_amount: 20,
    },
    ItemDef {
        item_type:   "bandage",
        name:        "Bandage",
        description: "Strips of clean linen. Useful in a pinch. Restores 8 HP.",
        kind:        ItemKind::Consumable,
        heal_amount: 8,
    },
    ItemDef {
        item_type:   "antidote",
        name:        "Antidote",
        description: "A bitter green tonic. Neutralises common poisons. No effect yet.",
        kind:        ItemKind::Consumable,
        heal_amount: 0,
    },
    ItemDef {
        item_type:   "ration",
        name:        "Ration",
        description: "Hard bread and dried meat — reliable trail food. Restores 5 HP.",
        kind:        ItemKind::Consumable,
        heal_amount: 5,
    },
    ItemDef {
        item_type:   "wolf_pelt",
        name:        "Wolf Pelt",
        description: "The thick pelt of a wild wolf. Might fetch a fair price from a trader.",
        kind:        ItemKind::Equipment,
        heal_amount: 0,
    },
    ItemDef {
        item_type:   "fox_pelt",
        name:        "Fox Pelt",
        description: "A fine pelt with a russet sheen. Soft and surprisingly valuable.",
        kind:        ItemKind::Equipment,
        heal_amount: 0,
    },
    ItemDef {
        item_type:   "old_map",
        name:        "Old Map",
        description: "A worn parchment covered in faded markings. Someone went to great lengths to hide this.",
        kind:        ItemKind::Quest,
        heal_amount: 0,
    },
];

pub fn find_def(item_type: &str) -> Option<&'static ItemDef> {
    ITEM_CATALOG.iter().find(|d| d.item_type == item_type)
}

// ── Inventory state ───────────────────────────────────────────────────────────

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

    /// Clamp cursor after items list changes (e.g. after using the last of a stack).
    pub fn clamp_cursor(&mut self) {
        if !self.items.is_empty() && self.cursor >= self.items.len() {
            self.cursor = self.items.len() - 1;
        }
    }
}
