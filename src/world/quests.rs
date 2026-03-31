use super::{
    AreaId, NpcId, ObjectiveKind, QuestDef, QuestId, QuestItemHintDef, QuestObjectiveDef,
    QuestReward,
};

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

const RUMORS_IN_THE_HEARTH_OBJECTIVES: &[QuestObjectiveDef] = &[QuestObjectiveDef {
    text: "Ask Innkeeper Brin what the townsfolk whisper about the Mage King.",
    kind: ObjectiveKind::TalkToNpc {
        npc: NpcId::InnkeeperBrin,
    },
}];

const THE_BLIGHTED_MOOR_OBJECTIVES: &[QuestObjectiveDef] = &[QuestObjectiveDef {
    text: "Reach the Blighted Moor east of the barrow.",
    kind: ObjectiveKind::VisitArea {
        area: AreaId::BlightedMoor,
    },
}];

const SERVANTS_OF_ASH_OBJECTIVES: &[QuestObjectiveDef] = &[QuestObjectiveDef {
    text: "Destroy 3 undead packs roaming the Blighted Moor.",
    kind: ObjectiveKind::KillFamily {
        family: "Undead",
        count: 3,
    },
}];

const THE_KINGS_CIPHER_OBJECTIVES: &[QuestObjectiveDef] = &[QuestObjectiveDef {
    text: "Recover 1 cipher scroll from the Mage King's risen servants.",
    kind: ObjectiveKind::OwnItem {
        item_type: "cipher_scroll",
        count: 1,
    },
}];

const THE_EXILED_THRONE_OBJECTIVES: &[QuestObjectiveDef] = &[QuestObjectiveDef {
    text: "Deliver the cipher scroll to Arcanist Sel.",
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
    QuestDef {
        id: QuestId::RumorsInTheHearth,
        name: "Rumors in the Hearth",
        summary: "Sel needs ears closer to the street. Innkeeper Brin hears every whisper Hearthmere cannot keep quiet.",
        giver: NpcId::ArcanistSel,
        required_quest: Some(QuestId::CrownInCinders),
        required_flags: &[],
        objectives: RUMORS_IN_THE_HEARTH_OBJECTIVES,
        rewards: QuestReward {
            xp: 170,
            gold: 60,
            item_type: Some("greater_health_potion"),
            item_qty: 2,
        },
    },
    QuestDef {
        id: QuestId::TheBlightedMoor,
        name: "The Blighted Moor",
        summary: "Brin's patrons speak of a fouled marshland east of the barrow where the Mage King's loyalists gather.",
        giver: NpcId::InnkeeperBrin,
        required_quest: Some(QuestId::RumorsInTheHearth),
        required_flags: &[],
        objectives: THE_BLIGHTED_MOOR_OBJECTIVES,
        rewards: QuestReward {
            xp: 200,
            gold: 75,
            item_type: Some("antidote"),
            item_qty: 2,
        },
    },
    QuestDef {
        id: QuestId::ServantsOfAsh,
        name: "Servants of Ash",
        summary: "The Mage King's risen servants patrol the moor in growing numbers. Drive them back before they march on Hearthmere.",
        giver: NpcId::ScoutMira,
        required_quest: Some(QuestId::TheBlightedMoor),
        required_flags: &[],
        objectives: SERVANTS_OF_ASH_OBJECTIVES,
        rewards: QuestReward {
            xp: 230,
            gold: 85,
            item_type: Some("greater_mana_tonic"),
            item_qty: 1,
        },
    },
    QuestDef {
        id: QuestId::TheKingsCipher,
        name: "The King's Cipher",
        summary: "Among the risen, one carried sealed orders in the Mage King's own cipher. Vale needs it recovered before it rots in the marsh.",
        giver: NpcId::QuartermasterVale,
        required_quest: Some(QuestId::ServantsOfAsh),
        required_flags: &[],
        objectives: THE_KINGS_CIPHER_OBJECTIVES,
        rewards: QuestReward {
            xp: 220,
            gold: 90,
            item_type: Some("silver_amulet"),
            item_qty: 1,
        },
    },
    QuestDef {
        id: QuestId::TheExiledThrone,
        name: "The Exiled Throne",
        summary: "Bring the cipher to Arcanist Sel and learn where the Mage King plans to reclaim his throne.",
        giver: NpcId::ArcanistSel,
        required_quest: Some(QuestId::TheKingsCipher),
        required_flags: &[],
        objectives: THE_EXILED_THRONE_OBJECTIVES,
        rewards: QuestReward {
            xp: 260,
            gold: 100,
            item_type: Some("phoenix_cloak"),
            item_qty: 1,
        },
    },
];

pub const QUEST_ITEM_HINTS: &[QuestItemHintDef] = &[
    QuestItemHintDef {
        quest_id: "roadside_ledger",
        item_type: "old_map",
        relevant_families: &["Bandit"],
        relevant_environment_tags: &["road"],
        miss_text: Some("You search the fallen raiders, but the old map is nowhere on them."),
    },
    QuestItemHintDef {
        quest_id: "the_kings_cipher",
        item_type: "cipher_scroll",
        relevant_families: &["Undead"],
        relevant_environment_tags: &["moor"],
        miss_text: Some("You search the fallen dead, but the cipher scroll is not among them."),
    },
];

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
        "rumors_in_the_hearth" => {
            &["Brin speaks of travelers who saw lights on a fouled moor east of the barrow."]
        }
        "the_blighted_moor" => &[
            "The marsh reeks of death and old magic. The Mage King's servants are already here.",
        ],
        "servants_of_ash" => {
            &["Among the fallen risen, you find sealed orders bearing the Mage King's cipher."]
        }
        "the_kings_cipher" => &[
            "The cipher scroll is intact. Arcanist Sel is the only one who can read it.",
        ],
        "the_exiled_throne" => &[
            "Sel decodes the scroll and names the place where the Mage King will return.",
            "The second chapter opens with a war the town cannot yet see.",
        ],
        _ => &[],
    }
}
