#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AreaId {
    WhisperingWoods,
    SunkenRoad,
    AshenBarrow,
}

impl AreaId {
    pub const ALL: [AreaId; 3] = [
        AreaId::WhisperingWoods,
        AreaId::SunkenRoad,
        AreaId::AshenBarrow,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::WhisperingWoods => "whispering_woods",
            Self::SunkenRoad => "sunken_road",
            Self::AshenBarrow => "ashen_barrow",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::WhisperingWoods => "Whispering Woods",
            Self::SunkenRoad => "Sunken Road",
            Self::AshenBarrow => "Ashen Barrow",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VendorId {
    Quartermaster,
    Arcanist,
    Provisioner,
}

impl VendorId {
    pub const ALL: [VendorId; 3] = [
        VendorId::Quartermaster,
        VendorId::Arcanist,
        VendorId::Provisioner,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::Quartermaster => "quartermaster",
            Self::Arcanist => "arcanist",
            Self::Provisioner => "provisioner",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Quartermaster => "Quartermaster",
            Self::Arcanist => "Arcanist",
            Self::Provisioner => "Provisioner",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuestId {
    FirstBlood,
    SupplyLine,
    TombQuietus,
}

impl QuestId {
    pub const ALL: [QuestId; 3] = [QuestId::FirstBlood, QuestId::SupplyLine, QuestId::TombQuietus];

    pub fn id(self) -> &'static str {
        match self {
            Self::FirstBlood => "first_blood",
            Self::SupplyLine => "supply_line",
            Self::TombQuietus => "tomb_quietus",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ObjectiveKind {
    KillFamily { family: &'static str, count: i32 },
    VisitArea { area: AreaId },
    OwnItem { item_type: &'static str, count: i32 },
}

#[derive(Debug, Clone)]
pub struct QuestObjectiveDef {
    pub text: &'static str,
    pub kind: ObjectiveKind,
}

#[derive(Debug, Clone)]
pub struct QuestReward {
    pub xp: i32,
    pub gold: i32,
    pub item_type: Option<&'static str>,
    pub item_qty: i32,
}

#[derive(Debug, Clone)]
pub struct QuestDef {
    pub id: QuestId,
    pub name: &'static str,
    pub summary: &'static str,
    pub giver: &'static str,
    pub objectives: &'static [QuestObjectiveDef],
    pub rewards: QuestReward,
}

#[derive(Debug, Clone)]
pub struct QuestProgress {
    pub quest_id: String,
    pub accepted: bool,
    pub completed: bool,
    pub objective_index: usize,
    pub progress: i32,
}

#[derive(Debug, Clone)]
pub struct VendorItem {
    pub item_type: &'static str,
    pub stock: i32,
}

#[derive(Debug, Clone)]
pub struct VendorDef {
    pub id: VendorId,
    pub name: &'static str,
    pub greeting: &'static str,
    pub inventory: &'static [VendorItem],
}

#[derive(Debug, Clone)]
pub struct AreaDef {
    pub id: AreaId,
    pub name: &'static str,
    pub description: &'static str,
    pub danger: &'static str,
    pub encounters: &'static [&'static str],
    pub event_text: &'static str,
}

#[derive(Debug, Clone)]
pub struct WorldState {
    pub current_area: Option<String>,
    pub unlocked_areas: Vec<String>,
    pub active_quests: Vec<QuestProgress>,
    pub completed_quests: Vec<String>,
    pub world_flags: Vec<String>,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            current_area: None,
            unlocked_areas: vec![AreaId::WhisperingWoods.id().to_string()],
            active_quests: vec![],
            completed_quests: vec![],
            world_flags: vec![],
        }
    }
}

impl WorldState {
    pub fn is_area_unlocked(&self, area: AreaId) -> bool {
        self.unlocked_areas.iter().any(|it| it == area.id())
    }

    pub fn unlock_area(&mut self, area: AreaId) {
        if !self.is_area_unlocked(area) {
            self.unlocked_areas.push(area.id().to_string());
        }
    }

    pub fn has_completed(&self, quest: QuestId) -> bool {
        self.completed_quests.iter().any(|id| id == quest.id())
    }

    pub fn active_quest(&self, quest: QuestId) -> Option<&QuestProgress> {
        self.active_quests.iter().find(|it| it.quest_id == quest.id())
    }

    pub fn active_quest_mut(&mut self, quest: QuestId) -> Option<&mut QuestProgress> {
        self.active_quests.iter_mut().find(|it| it.quest_id == quest.id())
    }

    pub fn accept_quest(&mut self, quest: QuestId) -> bool {
        if self.has_completed(quest) || self.active_quest(quest).is_some() {
            return false;
        }
        self.active_quests.push(QuestProgress {
            quest_id: quest.id().to_string(),
            accepted: true,
            completed: false,
            objective_index: 0,
            progress: 0,
        });
        true
    }
}

const FIRST_BLOOD_OBJECTIVES: &[QuestObjectiveDef] = &[QuestObjectiveDef {
    text: "Cull 3 beast packs in the Whispering Woods.",
    kind: ObjectiveKind::KillFamily {
        family: "Beast",
        count: 3,
    },
}];

const SUPPLY_LINE_OBJECTIVES: &[QuestObjectiveDef] = &[
    QuestObjectiveDef {
        text: "Visit the Sunken Road.",
        kind: ObjectiveKind::VisitArea {
            area: AreaId::SunkenRoad,
        },
    },
    QuestObjectiveDef {
        text: "Bring back 2 old maps from bandit scouts.",
        kind: ObjectiveKind::OwnItem {
            item_type: "old_map",
            count: 2,
        },
    },
];

const TOMB_QUIETUS_OBJECTIVES: &[QuestObjectiveDef] = &[
    QuestObjectiveDef {
        text: "Reach the Ashen Barrow.",
        kind: ObjectiveKind::VisitArea {
            area: AreaId::AshenBarrow,
        },
    },
    QuestObjectiveDef {
        text: "Defeat 2 undead patrols.",
        kind: ObjectiveKind::KillFamily {
            family: "Undead",
            count: 2,
        },
    },
];

pub const QUESTS: &[QuestDef] = &[
    QuestDef {
        id: QuestId::FirstBlood,
        name: "First Blood",
        summary: "Thin the wolf packs circling the town palisade.",
        giver: "Captain Hedd",
        objectives: FIRST_BLOOD_OBJECTIVES,
        rewards: QuestReward {
            xp: 80,
            gold: 35,
            item_type: Some("iron_sword"),
            item_qty: 1,
        },
    },
    QuestDef {
        id: QuestId::SupplyLine,
        name: "Supply Line",
        summary: "Reopen the road and recover the courier's maps.",
        giver: "Quartermaster Vale",
        objectives: SUPPLY_LINE_OBJECTIVES,
        rewards: QuestReward {
            xp: 120,
            gold: 55,
            item_type: Some("stamina_draught"),
            item_qty: 2,
        },
    },
    QuestDef {
        id: QuestId::TombQuietus,
        name: "Tomb Quietus",
        summary: "Break the unrest spreading out from the barrow.",
        giver: "Arcanist Sel",
        objectives: TOMB_QUIETUS_OBJECTIVES,
        rewards: QuestReward {
            xp: 180,
            gold: 80,
            item_type: Some("ember_wand"),
            item_qty: 1,
        },
    },
];

const QUARTERMASTER_STOCK: &[VendorItem] = &[
    VendorItem { item_type: "iron_sword", stock: 1 },
    VendorItem { item_type: "wooden_shield", stock: 1 },
    VendorItem { item_type: "leather_chest", stock: 1 },
    VendorItem { item_type: "health_potion", stock: 5 },
];

const ARCANIST_STOCK: &[VendorItem] = &[
    VendorItem { item_type: "apprentice_staff", stock: 1 },
    VendorItem { item_type: "ember_wand", stock: 1 },
    VendorItem { item_type: "mana_tonic", stock: 4 },
    VendorItem { item_type: "antidote", stock: 3 },
];

const PROVISIONER_STOCK: &[VendorItem] = &[
    VendorItem { item_type: "ration", stock: 6 },
    VendorItem { item_type: "bandage", stock: 5 },
    VendorItem { item_type: "stamina_draught", stock: 4 },
    VendorItem { item_type: "traveler_cloak", stock: 1 },
];

pub const VENDORS: &[VendorDef] = &[
    VendorDef {
        id: VendorId::Quartermaster,
        name: "Quartermaster Vale",
        greeting: "Steel, straps, and hard lessons. Pick what keeps you alive.",
        inventory: QUARTERMASTER_STOCK,
    },
    VendorDef {
        id: VendorId::Arcanist,
        name: "Arcanist Sel",
        greeting: "Power is precise. Buy only what you can actually wield.",
        inventory: ARCANIST_STOCK,
    },
    VendorDef {
        id: VendorId::Provisioner,
        name: "Marta the Provisioner",
        greeting: "Food, wraps, and road goods. Adventurers always come back hungry.",
        inventory: PROVISIONER_STOCK,
    },
];

pub const AREAS: &[AreaDef] = &[
    AreaDef {
        id: AreaId::WhisperingWoods,
        name: "Whispering Woods",
        description: "Dense tree cover and restless beasts just beyond the watchfires.",
        danger: "Low",
        encounters: &["beast_hunt", "beast_alpha"],
        event_text: "You find claw-marked trees and abandoned campfire embers beneath the pines.",
    },
    AreaDef {
        id: AreaId::SunkenRoad,
        name: "Sunken Road",
        description: "A drowned trade route now stalked by raiders and scavengers.",
        danger: "Medium",
        encounters: &["bandit_ambush", "bandit_raiders"],
        event_text: "A broken wagon lies in the mud, its cargo scattered across the ditch.",
    },
    AreaDef {
        id: AreaId::AshenBarrow,
        name: "Ashen Barrow",
        description: "An ancient burial mound where the dead have started to stir.",
        danger: "High",
        encounters: &["undead_patrol", "barrow_rites"],
        event_text: "Cold wind spills from the cracked tomb door, carrying ash and whispers.",
    },
];

pub fn quest_def(id: &str) -> Option<&'static QuestDef> {
    QUESTS.iter().find(|quest| quest.id.id() == id)
}

pub fn vendor_def(id: VendorId) -> &'static VendorDef {
    VENDORS.iter().find(|vendor| vendor.id == id).unwrap_or(&VENDORS[0])
}

pub fn area_def(id: AreaId) -> &'static AreaDef {
    AREAS.iter().find(|area| area.id == id).unwrap_or(&AREAS[0])
}

