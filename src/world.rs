mod quests;

pub use quests::{QUEST_ITEM_HINTS, QUESTS, quest_completion_story_lines};

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
}

impl QuestId {
    pub const ALL: [QuestId; 10] = [
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
mod tests {
    use super::{QuestId, WorldState, quest_item_drop_is_relevant, quest_item_miss_text};

    #[test]
    fn story_quests_unlock_in_order() {
        let mut state = WorldState::default();
        assert!(state.can_accept_quest(QuestId::LanternsInTheRain));
        assert!(!state.can_accept_quest(QuestId::MissingOnTheWatch));
        state
            .completed_quests
            .push(QuestId::LanternsInTheRain.id().to_string());
        assert!(state.can_accept_quest(QuestId::MissingOnTheWatch));
    }

    #[test]
    fn world_flags_only_store_unique_values() {
        let mut state = WorldState::default();
        state.set_flag("motive_duty");
        state.set_flag("motive_duty");
        assert!(state.has_flag("motive_duty"));
        assert_eq!(state.world_flags.len(), 1);
    }

    #[test]
    fn quest_item_miss_text_is_context_aware() {
        let text = quest_item_miss_text(
            "roadside_ledger",
            "old_map",
            "Old Map",
            "Bandit Ambush",
            &["Bandit".to_string()],
            &["road".to_string()],
        );
        assert_eq!(
            text.as_deref(),
            Some("You search the fallen raiders, but the old map is nowhere on them.")
        );
        let no_text = quest_item_miss_text(
            "roadside_ledger",
            "old_map",
            "Old Map",
            "Beast Hunt",
            &["Beast".to_string()],
            &["woods".to_string()],
        );
        assert!(no_text.is_none());
    }

    #[test]
    fn story_item_drops_stop_after_their_quest_beat_is_done() {
        let mut state = WorldState::default();
        assert!(quest_item_drop_is_relevant(&state, "old_map"));
        assert!(quest_item_drop_is_relevant(&state, "bandit_seal"));

        state
            .completed_quests
            .push(QuestId::RoadsideLedger.id().to_string());
        assert!(!quest_item_drop_is_relevant(&state, "old_map"));
        assert!(quest_item_drop_is_relevant(&state, "bandit_seal"));

        state
            .completed_quests
            .push(QuestId::AshOnTheWax.id().to_string());
        assert!(!quest_item_drop_is_relevant(&state, "bandit_seal"));
    }
}
