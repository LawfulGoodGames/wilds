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

const LANTERNS_IN_THE_RAIN_OBJECTIVES: &[QuestObjectiveDef] = &[QuestObjectiveDef {
    text: "Speak with Captain Hedd in the square.",
    kind: ObjectiveKind::TalkToNpc {
        npc: NpcId::CaptainHedd,
    },
}];

const MISSING_ON_THE_WATCH_OBJECTIVES: &[QuestObjectiveDef] = &[QuestObjectiveDef {
    text: "Cull 2 beast packs in the Whispering Woods.",
    kind: ObjectiveKind::KillFamily {
        family: "Beast",
        count: 2,
    },
}];

const REPORT_TO_MIRA_OBJECTIVES: &[QuestObjectiveDef] = &[QuestObjectiveDef {
    text: "Find Scout Mira in Hearthmere and share what you saw.",
    kind: ObjectiveKind::TalkToNpc {
        npc: NpcId::ScoutMira,
    },
}];

const THE_BROKEN_PATROL_OBJECTIVES: &[QuestObjectiveDef] = &[QuestObjectiveDef {
    text: "Follow the missing patrol's trail onto the Sunken Road.",
    kind: ObjectiveKind::VisitArea {
        area: AreaId::SunkenRoad,
    },
}];

const ROADSIDE_LEDGER_OBJECTIVES: &[QuestObjectiveDef] = &[QuestObjectiveDef {
    text: "Recover 1 old map from raiders on the Sunken Road.",
    kind: ObjectiveKind::OwnItem {
        item_type: "old_map",
        count: 1,
    },
}];

const ASH_ON_THE_WAX_OBJECTIVES: &[QuestObjectiveDef] = &[QuestObjectiveDef {
    text: "Bring the blackened seal recovered on the Sunken Road to Arcanist Sel.",
    kind: ObjectiveKind::TalkToNpc {
        npc: NpcId::ArcanistSel,
    },
}];

const GRAVEWIND_OBJECTIVES: &[QuestObjectiveDef] = &[QuestObjectiveDef {
    text: "Push on to the Ashen Barrow.",
    kind: ObjectiveKind::VisitArea {
        area: AreaId::AshenBarrow,
    },
}];

const THE_FIRST_DEAD_OBJECTIVES: &[QuestObjectiveDef] = &[QuestObjectiveDef {
    text: "Cut down 2 undead patrols in the Ashen Barrow.",
    kind: ObjectiveKind::KillFamily {
        family: "Undead",
        count: 2,
    },
}];

const WORD_TO_THE_CAPTAIN_OBJECTIVES: &[QuestObjectiveDef] = &[QuestObjectiveDef {
    text: "Return to Captain Hedd with proof of the dead walking.",
    kind: ObjectiveKind::TalkToNpc {
        npc: NpcId::CaptainHedd,
    },
}];

const CROWN_IN_CINDERS_OBJECTIVES: &[QuestObjectiveDef] = &[QuestObjectiveDef {
    text: "Bring the captain's report to Arcanist Sel for a true reading.",
    kind: ObjectiveKind::TalkToNpc {
        npc: NpcId::ArcanistSel,
    },
}];

pub const QUESTS: &[QuestDef] = &[
    QuestDef {
        id: QuestId::LanternsInTheRain,
        name: "Lanterns in the Rain",
        summary: "Captain Hedd has a quiet job for anyone willing to search the Whispering Woods.",
        giver: NpcId::CaptainHedd,
        required_quest: None,
        required_flags: &[],
        objectives: LANTERNS_IN_THE_RAIN_OBJECTIVES,
        rewards: QuestReward {
            xp: 50,
            gold: 20,
            item_type: Some("ration"),
            item_qty: 2,
        },
    },
    QuestDef {
        id: QuestId::MissingOnTheWatch,
        name: "Missing on the Watch",
        summary: "Search the Whispering Woods for signs of the vanished patrol and clear out what found them first.",
        giver: NpcId::CaptainHedd,
        required_quest: Some(QuestId::LanternsInTheRain),
        required_flags: &[],
        objectives: MISSING_ON_THE_WATCH_OBJECTIVES,
        rewards: QuestReward {
            xp: 120,
            gold: 35,
            item_type: Some("bandage"),
            item_qty: 2,
        },
    },
    QuestDef {
        id: QuestId::ReportToMira,
        name: "Report to Mira",
        summary: "Bring the patrol's trail to the scout who knows the forest better than anyone left.",
        giver: NpcId::ScoutMira,
        required_quest: Some(QuestId::MissingOnTheWatch),
        required_flags: &[],
        objectives: REPORT_TO_MIRA_OBJECTIVES,
        rewards: QuestReward {
            xp: 95,
            gold: 25,
            item_type: Some("stamina_draught"),
            item_qty: 1,
        },
    },
    QuestDef {
        id: QuestId::TheBrokenPatrol,
        name: "The Broken Patrol",
        summary: "Follow the patrol's last path from the Whispering Woods onto the Sunken Road.",
        giver: NpcId::ScoutMira,
        required_quest: Some(QuestId::ReportToMira),
        required_flags: &[],
        objectives: THE_BROKEN_PATROL_OBJECTIVES,
        rewards: QuestReward {
            xp: 110,
            gold: 40,
            item_type: Some("stamina_draught"),
            item_qty: 2,
        },
    },
    QuestDef {
        id: QuestId::RoadsideLedger,
        name: "Roadside Ledger",
        summary: "Recover a courier's map ledger from the Sunken Road before the raiders burn every trace of the patrol's route.",
        giver: NpcId::QuartermasterVale,
        required_quest: Some(QuestId::TheBrokenPatrol),
        required_flags: &[],
        objectives: ROADSIDE_LEDGER_OBJECTIVES,
        rewards: QuestReward {
            xp: 140,
            gold: 45,
            item_type: Some("bandit_seal"),
            item_qty: 1,
        },
    },
    QuestDef {
        id: QuestId::AshOnTheWax,
        name: "Ash on the Wax",
        summary: "The blackened seal recovered on the Sunken Road bears a mark Sel does not want to name aloud.",
        giver: NpcId::ArcanistSel,
        required_quest: Some(QuestId::RoadsideLedger),
        required_flags: &[],
        objectives: ASH_ON_THE_WAX_OBJECTIVES,
        rewards: QuestReward {
            xp: 130,
            gold: 35,
            item_type: Some("mana_tonic"),
            item_qty: 2,
        },
    },
    QuestDef {
        id: QuestId::Gravewind,
        name: "Gravewind",
        summary: "Follow Sel's warning into the Ashen Barrow before whatever stirs there can spread.",
        giver: NpcId::ArcanistSel,
        required_quest: Some(QuestId::AshOnTheWax),
        required_flags: &[],
        objectives: GRAVEWIND_OBJECTIVES,
        rewards: QuestReward {
            xp: 155,
            gold: 55,
            item_type: Some("antidote"),
            item_qty: 1,
        },
    },
    QuestDef {
        id: QuestId::TheFirstDead,
        name: "The First Dead",
        summary: "Put down the first of the risen before Hearthmere learns what kind of war is coming.",
        giver: NpcId::ArcanistSel,
        required_quest: Some(QuestId::Gravewind),
        required_flags: &[],
        objectives: THE_FIRST_DEAD_OBJECTIVES,
        rewards: QuestReward {
            xp: 185,
            gold: 70,
            item_type: Some("ember_wand"),
            item_qty: 1,
        },
    },
    QuestDef {
        id: QuestId::WordToTheCaptain,
        name: "Word to the Captain",
        summary: "Carry the truth back to Captain Hedd before rumor reaches the walls ahead of you.",
        giver: NpcId::CaptainHedd,
        required_quest: Some(QuestId::TheFirstDead),
        required_flags: &[],
        objectives: WORD_TO_THE_CAPTAIN_OBJECTIVES,
        rewards: QuestReward {
            xp: 140,
            gold: 50,
            item_type: Some("wooden_shield"),
            item_qty: 1,
        },
    },
    QuestDef {
        id: QuestId::CrownInCinders,
        name: "Crown in Cinders",
        summary: "Bring Hedd's report to Sel and hear the first name tied to the dead: the exiled Mage King.",
        giver: NpcId::ArcanistSel,
        required_quest: Some(QuestId::WordToTheCaptain),
        required_flags: &[],
        objectives: CROWN_IN_CINDERS_OBJECTIVES,
        rewards: QuestReward {
            xp: 200,
            gold: 80,
            item_type: Some("apprentice_staff"),
            item_qty: 1,
        },
    },
];

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

pub const QUEST_ITEM_HINTS: &[QuestItemHintDef] = &[QuestItemHintDef {
    quest_id: "roadside_ledger",
    item_type: "old_map",
    relevant_families: &["Bandit"],
    relevant_environment_tags: &["road"],
    miss_text: Some("You search the fallen raiders, but the old map is nowhere on them."),
}];

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

pub fn quest_completion_story_lines(quest_id: &str) -> &'static [&'static str] {
    match quest_id {
        "lanterns_in_the_rain" => {
            &["Captain Hedd sends you into the Whispering Woods to trace the missing patrol."]
        }
        "missing_on_the_watch" => {
            &["The patrol's broken trail points back to Scout Mira in Hearthmere."]
        }
        "report_to_mira" => {
            &["Mira confirms the trail leaves the Whispering Woods and runs onto the Sunken Road."]
        }
        "the_broken_patrol" => &[
            "The patrol was driven onto the Sunken Road, and Vale wants the courier's ledger recovered before the raiders destroy it.",
        ],
        "roadside_ledger" => &[
            "Alongside the old map, you recover a blackened seal marked with ash.",
            "Arcanist Sel should see the seal immediately.",
        ],
        "ash_on_the_wax" => {
            &["Sel identifies the seal as a warning sign and sends you toward the Ashen Barrow."]
        }
        "gravewind" => &[
            "Whatever woke in the Ashen Barrow is no rumor now. Put the risen down before they spread.",
        ],
        "the_first_dead" => {
            &["You have proof now. Captain Hedd needs to hear what walked in the barrow."]
        }
        "word_to_the_captain" => {
            &["Hedd wants Sel's full reading before he warns the town of what is coming."]
        }
        "crown_in_cinders" => {
            &["The first chapter closes with the Mage King's name finally spoken aloud."]
        }
        _ => &[],
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
    use super::{QuestId, WorldState, quest_item_miss_text};

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
}
