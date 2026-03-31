mod quests;

pub use quests::{QUEST_ITEM_HINTS, QUESTS, quest_completion_story_lines};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AreaId {
    WhisperingWoods,
    SunkenRoad,
    AshenBarrow,
    BlightedMoor,
}

impl AreaId {
    pub const ALL: [AreaId; 4] = [
        AreaId::WhisperingWoods,
        AreaId::SunkenRoad,
        AreaId::AshenBarrow,
        AreaId::BlightedMoor,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::WhisperingWoods => "whispering_woods",
            Self::SunkenRoad => "sunken_road",
            Self::AshenBarrow => "ashen_barrow",
            Self::BlightedMoor => "blighted_moor",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::WhisperingWoods => "Whispering Woods",
            Self::SunkenRoad => "Sunken Road",
            Self::AshenBarrow => "Ashen Barrow",
            Self::BlightedMoor => "Blighted Moor",
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
pub enum NpcId {
    CaptainHedd,
    ScoutMira,
    QuartermasterVale,
    ArcanistSel,
    InnkeeperBrin,
}

impl NpcId {
    pub const ALL: [NpcId; 5] = [
        NpcId::CaptainHedd,
        NpcId::ScoutMira,
        NpcId::QuartermasterVale,
        NpcId::ArcanistSel,
        NpcId::InnkeeperBrin,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::CaptainHedd => "captain_hedd",
            Self::ScoutMira => "scout_mira",
            Self::QuartermasterVale => "quartermaster_vale",
            Self::ArcanistSel => "arcanist_sel",
            Self::InnkeeperBrin => "innkeeper_brin",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::CaptainHedd => "Captain Hedd",
            Self::ScoutMira => "Scout Mira",
            Self::QuartermasterVale => "Quartermaster Vale",
            Self::ArcanistSel => "Arcanist Sel",
            Self::InnkeeperBrin => "Innkeeper Brin",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuestId {
    LanternsInTheRain,
    MissingOnTheWatch,
    ReportToMira,
    TheBrokenPatrol,
    RoadsideLedger,
    AshOnTheWax,
    Gravewind,
    TheFirstDead,
    WordToTheCaptain,
    CrownInCinders,
    RumorsInTheHearth,
    TheBlightedMoor,
    ServantsOfAsh,
    TheKingsCipher,
    TheExiledThrone,
}

impl QuestId {
    pub const ALL: [QuestId; 15] = [
        QuestId::LanternsInTheRain,
        QuestId::MissingOnTheWatch,
        QuestId::ReportToMira,
        QuestId::TheBrokenPatrol,
        QuestId::RoadsideLedger,
        QuestId::AshOnTheWax,
        QuestId::Gravewind,
        QuestId::TheFirstDead,
        QuestId::WordToTheCaptain,
        QuestId::CrownInCinders,
        QuestId::RumorsInTheHearth,
        QuestId::TheBlightedMoor,
        QuestId::ServantsOfAsh,
        QuestId::TheKingsCipher,
        QuestId::TheExiledThrone,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::LanternsInTheRain => "lanterns_in_the_rain",
            Self::MissingOnTheWatch => "missing_on_the_watch",
            Self::ReportToMira => "report_to_mira",
            Self::TheBrokenPatrol => "the_broken_patrol",
            Self::RoadsideLedger => "roadside_ledger",
            Self::AshOnTheWax => "ash_on_the_wax",
            Self::Gravewind => "gravewind",
            Self::TheFirstDead => "the_first_dead",
            Self::WordToTheCaptain => "word_to_the_captain",
            Self::CrownInCinders => "crown_in_cinders",
            Self::RumorsInTheHearth => "rumors_in_the_hearth",
            Self::TheBlightedMoor => "the_blighted_moor",
            Self::ServantsOfAsh => "servants_of_ash",
            Self::TheKingsCipher => "the_kings_cipher",
            Self::TheExiledThrone => "the_exiled_throne",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ObjectiveKind {
    KillFamily { family: &'static str, count: i32 },
    VisitArea { area: AreaId },
    OwnItem { item_type: &'static str, count: i32 },
    TalkToNpc { npc: NpcId },
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
    pub giver: NpcId,
    pub required_quest: Option<QuestId>,
    pub required_flags: &'static [&'static str],
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
pub struct VendorStockEntry {
    pub vendor_id: String,
    pub item_type: String,
    pub quantity: i32,
}

#[derive(Debug, Clone)]
pub struct VendorDef {
    pub id: VendorId,
    pub name: &'static str,
    pub greeting: &'static str,
    pub inventory: &'static [VendorItem],
}

#[derive(Debug, Clone)]
pub struct NpcDef {
    pub id: NpcId,
    pub name: &'static str,
    pub role: &'static str,
    pub summary: &'static str,
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
pub struct QuestItemHintDef {
    pub quest_id: &'static str,
    pub item_type: &'static str,
    pub relevant_families: &'static [&'static str],
    pub relevant_environment_tags: &'static [&'static str],
    pub miss_text: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct WorldState {
    pub current_area: Option<String>,
    pub unlocked_areas: Vec<String>,
    pub active_quests: Vec<QuestProgress>,
    pub completed_quests: Vec<String>,
    pub vendor_stock: Vec<VendorStockEntry>,
    pub world_flags: Vec<String>,
    pub campaign_day: i32,
    pub hour_of_day: i32,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            current_area: None,
            unlocked_areas: vec![AreaId::WhisperingWoods.id().to_string()],
            active_quests: vec![],
            completed_quests: vec![],
            vendor_stock: vec![],
            world_flags: vec![],
            campaign_day: 1,
            hour_of_day: 8,
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

    pub fn prune_stale_active_quests(&mut self) {
        self.active_quests.retain(|progress| {
            !self
                .completed_quests
                .iter()
                .any(|id| id == &progress.quest_id)
        });
    }

    pub fn active_quest(&self, quest: QuestId) -> Option<&QuestProgress> {
        self.active_quests
            .iter()
            .find(|it| it.quest_id == quest.id())
    }

    pub fn active_quest_mut(&mut self, quest: QuestId) -> Option<&mut QuestProgress> {
        self.active_quests
            .iter_mut()
            .find(|it| it.quest_id == quest.id())
    }

    pub fn accept_quest(&mut self, quest: QuestId) -> bool {
        if self.has_completed(quest)
            || self.active_quest(quest).is_some()
            || !self.can_accept_quest(quest)
        {
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

    pub fn has_flag(&self, flag: &str) -> bool {
        self.world_flags.iter().any(|it| it == flag)
    }

    pub fn set_flag(&mut self, flag: &str) {
        if !self.has_flag(flag) {
            self.world_flags.push(flag.to_string());
        }
    }

    pub fn can_accept_quest(&self, quest: QuestId) -> bool {
        if self.has_completed(quest) || self.active_quest(quest).is_some() {
            return false;
        }
        let Some(def) = quest_def(quest.id()) else {
            return false;
        };
        let prereq_ok = def
            .required_quest
            .map(|required| self.has_completed(required))
            .unwrap_or(true);
        let flags_ok = def.required_flags.iter().all(|flag| self.has_flag(flag));
        prereq_ok && flags_ok
    }

    pub fn current_story_lead(&self) -> Option<QuestId> {
        if let Some(active) = self.active_quests.first() {
            return QuestId::ALL
                .iter()
                .copied()
                .find(|quest| quest.id() == active.quest_id);
        }
        QuestId::ALL
            .iter()
            .copied()
            .find(|quest| self.can_accept_quest(*quest))
    }

    pub fn vendor_stock(&self, vendor: VendorId, item_type: &str) -> i32 {
        self.vendor_stock
            .iter()
            .find(|entry| entry.vendor_id == vendor.id() && entry.item_type == item_type)
            .map(|entry| entry.quantity)
            .unwrap_or_else(|| default_vendor_stock(vendor, item_type))
    }

    pub fn set_vendor_stock(&mut self, vendor: VendorId, item_type: &str, quantity: i32) {
        if let Some(entry) = self
            .vendor_stock
            .iter_mut()
            .find(|entry| entry.vendor_id == vendor.id() && entry.item_type == item_type)
        {
            entry.quantity = quantity.max(0);
            return;
        }
        self.vendor_stock.push(VendorStockEntry {
            vendor_id: vendor.id().to_string(),
            item_type: item_type.to_string(),
            quantity: quantity.max(0),
        });
    }

    pub fn decrement_vendor_stock(&mut self, vendor: VendorId, item_type: &str) -> bool {
        let remaining = self.vendor_stock(vendor, item_type);
        if remaining <= 0 {
            return false;
        }
        self.set_vendor_stock(vendor, item_type, remaining - 1);
        true
    }

    pub fn advance_time(&mut self, hours: i32) {
        if hours <= 0 {
            return;
        }
        let total_hours = self.hour_of_day + hours;
        self.campaign_day += (total_hours - 1) / 24;
        self.hour_of_day = ((total_hours - 1) % 24) + 1;
    }

    pub fn time_label(&self) -> String {
        let period = match self.hour_of_day {
            5..=11 => "Morning",
            12..=16 => "Afternoon",
            17..=20 => "Evening",
            _ => "Night",
        };
        format!(
            "Day {} • {:02}:00 {}",
            self.campaign_day, self.hour_of_day, period
        )
    }
}

const QUARTERMASTER_STOCK: &[VendorItem] = &[
    VendorItem {
        item_type: "iron_sword",
        stock: 1,
    },
    VendorItem {
        item_type: "wooden_shield",
        stock: 1,
    },
    VendorItem {
        item_type: "leather_chest",
        stock: 1,
    },
    VendorItem {
        item_type: "health_potion",
        stock: 5,
    },
    VendorItem {
        item_type: "warden_blade",
        stock: 1,
    },
    VendorItem {
        item_type: "tower_shield",
        stock: 1,
    },
    VendorItem {
        item_type: "brigandine",
        stock: 1,
    },
    VendorItem {
        item_type: "kingsguard_greatsword",
        stock: 1,
    },
    VendorItem {
        item_type: "bastion_shield",
        stock: 1,
    },
];

const ARCANIST_STOCK: &[VendorItem] = &[
    VendorItem {
        item_type: "apprentice_staff",
        stock: 1,
    },
    VendorItem {
        item_type: "ember_wand",
        stock: 1,
    },
    VendorItem {
        item_type: "mana_tonic",
        stock: 4,
    },
    VendorItem {
        item_type: "antidote",
        stock: 3,
    },
    VendorItem {
        item_type: "stormglass_rod",
        stock: 1,
    },
    VendorItem {
        item_type: "moonseal_amulet",
        stock: 1,
    },
    VendorItem {
        item_type: "greater_mana_tonic",
        stock: 2,
    },
    VendorItem {
        item_type: "archmage_focus",
        stock: 1,
    },
    VendorItem {
        item_type: "sunfire_talisman",
        stock: 1,
    },
];

const PROVISIONER_STOCK: &[VendorItem] = &[
    VendorItem {
        item_type: "ration",
        stock: 6,
    },
    VendorItem {
        item_type: "bandage",
        stock: 5,
    },
    VendorItem {
        item_type: "stamina_draught",
        stock: 4,
    },
    VendorItem {
        item_type: "traveler_cloak",
        stock: 1,
    },
    VendorItem {
        item_type: "longshot_bow",
        stock: 1,
    },
    VendorItem {
        item_type: "ranger_boots",
        stock: 1,
    },
    VendorItem {
        item_type: "phoenix_cloak",
        stock: 1,
    },
    VendorItem {
        item_type: "greater_health_potion",
        stock: 3,
    },
    VendorItem {
        item_type: "greater_stamina_draught",
        stock: 2,
    },
    VendorItem {
        item_type: "duskrunner_bow",
        stock: 1,
    },
    VendorItem {
        item_type: "scoutmaster_boots",
        stock: 1,
    },
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

fn default_vendor_stock(vendor: VendorId, item_type: &str) -> i32 {
    vendor_def(vendor)
        .inventory
        .iter()
        .find(|entry| entry.item_type == item_type)
        .map(|entry| entry.stock)
        .unwrap_or(0)
}

pub const NPCS: &[NpcDef] = &[
    NpcDef {
        id: NpcId::CaptainHedd,
        name: "Captain Hedd",
        role: "Watch Captain",
        summary: "Keeps Hearthmere standing with worn discipline and fewer people than the walls deserve.",
    },
    NpcDef {
        id: NpcId::ScoutMira,
        name: "Scout Mira",
        role: "Frontier Scout",
        summary: "Reads the woods like a ledger and notices what fear makes others miss.",
    },
    NpcDef {
        id: NpcId::QuartermasterVale,
        name: "Quartermaster Vale",
        role: "Quartermaster",
        summary: "Tracks steel, rations, and losses with the same steady voice.",
    },
    NpcDef {
        id: NpcId::ArcanistSel,
        name: "Arcanist Sel",
        role: "Court-taught Arcanist",
        summary: "Understands exactly how dangerous old royal magic can become in the wrong hands.",
    },
    NpcDef {
        id: NpcId::InnkeeperBrin,
        name: "Innkeeper Brin",
        role: "Innkeeper",
        summary: "Hears every rumor Hearthmere is too tired to keep quiet.",
    },
];

pub const AREAS: &[AreaDef] = &[
    AreaDef {
        id: AreaId::WhisperingWoods,
        name: "Whispering Woods",
        description: "Dense tree cover and restless beasts just beyond the watchfires.",
        danger: "Low",
        encounters: &["beast_hunt", "beast_hunt", "beast_alpha"],
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
    AreaDef {
        id: AreaId::BlightedMoor,
        name: "Blighted Moor",
        description: "A rotting marshland east of the barrow, fouled by the Mage King's lingering magic.",
        danger: "High",
        encounters: &["moor_shamble", "moor_channeling"],
        event_text: "Black water seeps between dead reeds, and the ground pulses with a faint, wrong warmth.",
    },
];

pub fn quest_def(id: &str) -> Option<&'static QuestDef> {
    QUESTS.iter().find(|quest| quest.id.id() == id)
}

pub fn quest_item_miss_text(
    quest_id: &str,
    item_type: &str,
    item_name: &str,
    encounter_name: &str,
    defeated_families: &[String],
    environment_tags: &[String],
) -> Option<String> {
    let hint = QUEST_ITEM_HINTS
        .iter()
        .find(|hint| hint.quest_id == quest_id && hint.item_type == item_type)?;
    let family_match = defeated_families
        .iter()
        .any(|family| hint.relevant_families.iter().any(|needed| family == needed));
    let tag_match = environment_tags.iter().any(|tag| {
        hint.relevant_environment_tags
            .iter()
            .any(|needed| tag == needed)
    });
    if !family_match && !tag_match {
        return None;
    }
    if let Some(text) = hint.miss_text {
        Some(text.to_string())
    } else {
        Some(format!(
            "You search after {}, but the {} is nowhere to be found.",
            encounter_name,
            item_name.to_lowercase()
        ))
    }
}

pub fn quest_item_drop_is_relevant(state: &WorldState, item_type: &str) -> bool {
    match item_type {
        "old_map" => !state.has_completed(QuestId::RoadsideLedger),
        "bandit_seal" => !state.has_completed(QuestId::AshOnTheWax),
        "cipher_scroll" => !state.has_completed(QuestId::TheKingsCipher),
        _ => true,
    }
}

pub fn npc_def(id: NpcId) -> &'static NpcDef {
    NPCS.iter().find(|npc| npc.id == id).unwrap_or(&NPCS[0])
}

pub fn vendor_def(id: VendorId) -> &'static VendorDef {
    VENDORS
        .iter()
        .find(|vendor| vendor.id == id)
        .unwrap_or(&VENDORS[0])
}

pub fn area_def(id: AreaId) -> &'static AreaDef {
    AREAS.iter().find(|area| area.id == id).unwrap_or(&AREAS[0])
}

#[cfg(test)]
mod tests;
