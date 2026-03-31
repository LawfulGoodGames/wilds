#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct DialogueChoiceDef {
    pub label: &'static str,
    pub response_lines: &'static [&'static str],
    pub memory_flag: Option<&'static str>,
    pub status_message: Option<&'static str>,
    pub audio_id: Option<&'static str>,
    pub audio_filename: Option<&'static str>,
    pub audio_model: Option<&'static str>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct DialogueSceneDef {
    pub id: &'static str,
    pub title: &'static str,
    pub lines: &'static [&'static str],
    pub choices: &'static [DialogueChoiceDef],
    pub audio_filename: &'static str,
    pub audio_model: &'static str,
}

#[derive(Debug, Clone)]
pub struct VoiceLine {
    pub id: &'static str,
    pub filename: &'static str,
    pub model: &'static str,
    pub text: String,
}

const HEDD_FIRST_MEETING_LINES: &[&str] = &[
    "Rain beads on Captain Hedd's coat while lanterns sway over the square.",
    "\"Good. A steady pair of hands.\" He taps a rough patrol map. \"One of ours is missing in the Whispering Woods. Start there and bring back the truth, not tavern fog.\"",
    "\"Before you go, answer me plain. Why stand the line for Hearthmere?\"",
];

const HEDD_DUTY_RESPONSE_LINES: &[&str] =
    &["Hedd gives a short nod. \"Duty keeps towns alive longer than luck does. Remember that.\""];
const HEDD_COIN_RESPONSE_LINES: &[&str] = &[
    "Hedd snorts. \"Honest enough. Stay alive and there might even be coin left when this is done.\"",
];
const HEDD_TRUTH_RESPONSE_LINES: &[&str] = &[
    "\"Then keep your eyes open wider than the last patrol did,\" Hedd says, pushing the map toward you.",
];
const HEDD_MISSING_LINES: &[&str] = &[
    "\"Work the western edge of the Whispering Woods and trust fresh silence less than fresh tracks,\" Hedd says.",
    "\"If the woods went too quiet for the patrol, they'll try the same trick on you there too.\"",
];
const HEDD_WORD_LINES: &[&str] = &[
    "You lay out the story of the barrow and the marching dead. Hedd goes still.",
    "\"Then this isn't a bad season. It's the front edge of a war.\" He folds the report with soldier's care.",
    "\"Take this to Sel. If she names what raised them, I can start convincing the town before panic does it for me.\"",
];
const HEDD_DEFAULT_LINES: &[&str] = &[
    "\"Keep your pack ready and your head low,\" Hedd says. \"Hearthmere doesn't get quiet by accident anymore.\"",
];

const MIRA_REPORT_LINES: &[&str] = &[
    "Mira studies the mud on your boots before she studies your face.",
    "\"So the patrol crossed the Whispering Woods and never came back from the Sunken Road. That's not wolves.\"",
    "\"Do we stay close and map every step, or press hard before the trail cools?\"",
];
const MIRA_CAREFUL_RESPONSE_LINES: &[&str] =
    &["\"Careful work wins long hunts,\" Mira says. \"Good. We'll read this trail clean.\""];
const MIRA_BOLD_RESPONSE_LINES: &[&str] =
    &["Mira smiles without warmth. \"Fast, then. Just don't mistake speed for silence.\""];
const MIRA_DEFAULT_LINES: &[&str] = &[
    "\"The forest only lies to people who rush it,\" Mira says. \"Listen long enough and it names the thing that frightened it.\"",
];

const VALE_DEFAULT_LINES: &[&str] = &[
    "\"Every cart lost on the Sunken Road becomes someone else's courage problem,\" Vale says, arms full of tally slips.",
    "\"If you find maps, seals, or anything the raiders missed on the road, bring it back before the rain takes the ink.\"",
];

const SEL_ASH_ON_WAX_LINES: &[&str] = &[
    "Sel turns the blackened seal under a lamp until ash glitters in its wax.",
    "\"This mark belonged to one court only, and it should have died with its master.\"",
    "\"If this ash rides with raiders on the Sunken Road, then someone is testing the roads before they test the gates.\"",
];
const SEL_CROWN_IN_CINDERS_LINES: &[&str] = &[
    "Sel reads Hedd's report twice before speaking.",
    "\"The dead in the Ashen Barrow, the ash sigil, the sealed Sunken Road. I know the hand behind that pattern.\"",
    "\"The defamed Mage King did not die in exile. He fled the kingdom, and now he is building an army of the dead to march on the capital.\"",
    "\"Hearthmere is only the first place close enough to hear him testing the drum.\"",
];
const SEL_GRAVEWIND_LINES: &[&str] = &[
    "Sel wraps the seal in cloth as though even its wax should not touch bare skin.",
    "\"If the mark has reached the road, the answer will be waiting where the dead were first taught to stand again,\" she says.",
    "\"Go to the Ashen Barrow. Whatever is waking there is only the beginning.\"",
];
const SEL_DEFAULT_LINES: &[&str] = &[
    "\"Old magic rarely wakes alone,\" Sel says. \"If something has stirred, something else called it.\"",
];

const BRIN_DEFAULT_LINES: &[&str] = &[
    "\"Hearthmere still eats, still sings, and still sweeps blood off the stones by dawn,\" Brin says, polishing a cup.",
    "\"That means we're not beaten yet. It also means you should hear every rumor twice before believing it.\"",
];

const HEDD_FIRST_MEETING_CHOICES: &[DialogueChoiceDef] = &[
    DialogueChoiceDef {
        label: "Because someone has to hold the wall.",
        response_lines: HEDD_DUTY_RESPONSE_LINES,
        memory_flag: Some("hedd_motive_duty"),
        status_message: Some("Captain Hedd remembers your sense of duty."),
        audio_id: Some("hedd.response.duty"),
        audio_filename: Some("hedd-duty-response.wav"),
        audio_model: Some("aura-2-draco-en"),
    },
    DialogueChoiceDef {
        label: "Because danger pays better than hunger.",
        response_lines: HEDD_COIN_RESPONSE_LINES,
        memory_flag: Some("hedd_motive_coin"),
        status_message: Some("Captain Hedd clocks your eye for coin."),
        audio_id: Some("hedd.response.coin"),
        audio_filename: Some("hedd-coin-left.wav"),
        audio_model: Some("aura-2-draco-en"),
    },
    DialogueChoiceDef {
        label: "Because I want to be the one who sees what's coming.",
        response_lines: HEDD_TRUTH_RESPONSE_LINES,
        memory_flag: Some("hedd_motive_truth"),
        status_message: Some("Captain Hedd notes your hunger for the truth."),
        audio_id: Some("hedd.response.truth"),
        audio_filename: Some("hedd-truth-response.wav"),
        audio_model: Some("aura-2-draco-en"),
    },
];

const MIRA_REPORT_CHOICES: &[DialogueChoiceDef] = &[
    DialogueChoiceDef {
        label: "Map every step. We only get one first read.",
        response_lines: MIRA_CAREFUL_RESPONSE_LINES,
        memory_flag: Some("mira_method_careful"),
        status_message: Some("Mira notes that you favor caution."),
        audio_id: Some("mira.response.careful"),
        audio_filename: Some("mira-careful-response.wav"),
        audio_model: Some("aura-2-theia-en"),
    },
    DialogueChoiceDef {
        label: "Press hard. Whoever did this is still ahead of us.",
        response_lines: MIRA_BOLD_RESPONSE_LINES,
        memory_flag: Some("mira_method_bold"),
        status_message: Some("Mira notes that you press the advantage."),
        audio_id: Some("mira.response.bold"),
        audio_filename: Some("mira-bold-response.wav"),
        audio_model: Some("aura-2-theia-en"),
    },
];

pub const HEDD_FIRST_MEETING_SCENE: DialogueSceneDef = DialogueSceneDef {
    id: "hedd.first_meeting",
    title: "Captain Hedd",
    lines: HEDD_FIRST_MEETING_LINES,
    choices: HEDD_FIRST_MEETING_CHOICES,
    audio_filename: "steady-hands.wav",
    audio_model: "aura-2-draco-en",
};

pub const HEDD_MISSING_SCENE: DialogueSceneDef = DialogueSceneDef {
    id: "hedd.missing_on_the_watch",
    title: "Captain Hedd",
    lines: HEDD_MISSING_LINES,
    choices: &[],
    audio_filename: "hedd-missing-on-the-watch.wav",
    audio_model: "aura-2-draco-en",
};

pub const HEDD_WORD_SCENE: DialogueSceneDef = DialogueSceneDef {
    id: "hedd.word_to_the_captain",
    title: "Captain Hedd",
    lines: HEDD_WORD_LINES,
    choices: &[],
    audio_filename: "hedd-word-to-the-captain.wav",
    audio_model: "aura-2-draco-en",
};

pub const HEDD_DEFAULT_SCENE: DialogueSceneDef = DialogueSceneDef {
    id: "hedd.default",
    title: "Captain Hedd",
    lines: HEDD_DEFAULT_LINES,
    choices: &[],
    audio_filename: "hedd-default.wav",
    audio_model: "aura-2-draco-en",
};

pub const MIRA_REPORT_SCENE: DialogueSceneDef = DialogueSceneDef {
    id: "mira.report_to_mira",
    title: "Scout Mira",
    lines: MIRA_REPORT_LINES,
    choices: MIRA_REPORT_CHOICES,
    audio_filename: "mira-report-to-mira.wav",
    audio_model: "aura-2-theia-en",
};

pub const MIRA_DEFAULT_SCENE: DialogueSceneDef = DialogueSceneDef {
    id: "mira.default",
    title: "Scout Mira",
    lines: MIRA_DEFAULT_LINES,
    choices: &[],
    audio_filename: "mira-default.wav",
    audio_model: "aura-2-theia-en",
};

pub const VALE_DEFAULT_SCENE: DialogueSceneDef = DialogueSceneDef {
    id: "vale.default",
    title: "Quartermaster Vale",
    lines: VALE_DEFAULT_LINES,
    choices: &[],
    audio_filename: "vale-default.wav",
    audio_model: "aura-2-neptune-en",
};

pub const SEL_ASH_ON_WAX_SCENE: DialogueSceneDef = DialogueSceneDef {
    id: "sel.ash_on_the_wax",
    title: "Arcanist Sel",
    lines: SEL_ASH_ON_WAX_LINES,
    choices: &[],
    audio_filename: "sel-ash-on-the-wax.wav",
    audio_model: "aura-2-athena-en",
};

pub const SEL_CROWN_IN_CINDERS_SCENE: DialogueSceneDef = DialogueSceneDef {
    id: "sel.crown_in_cinders",
    title: "Arcanist Sel",
    lines: SEL_CROWN_IN_CINDERS_LINES,
    choices: &[],
    audio_filename: "sel-crown-in-cinders.wav",
    audio_model: "aura-2-athena-en",
};

pub const SEL_GRAVEWIND_SCENE: DialogueSceneDef = DialogueSceneDef {
    id: "sel.gravewind",
    title: "Arcanist Sel",
    lines: SEL_GRAVEWIND_LINES,
    choices: &[],
    audio_filename: "sel-gravewind.wav",
    audio_model: "aura-2-athena-en",
};

pub const SEL_DEFAULT_SCENE: DialogueSceneDef = DialogueSceneDef {
    id: "sel.default",
    title: "Arcanist Sel",
    lines: SEL_DEFAULT_LINES,
    choices: &[],
    audio_filename: "sel-default.wav",
    audio_model: "aura-2-athena-en",
};

pub const BRIN_DEFAULT_SCENE: DialogueSceneDef = DialogueSceneDef {
    id: "brin.default",
    title: "Innkeeper Brin",
    lines: BRIN_DEFAULT_LINES,
    choices: &[],
    audio_filename: "brin-default.wav",
    audio_model: "aura-2-cordelia-en",
};

const ALL_SCENES: &[&DialogueSceneDef] = &[
    &HEDD_FIRST_MEETING_SCENE,
    &HEDD_MISSING_SCENE,
    &HEDD_WORD_SCENE,
    &HEDD_DEFAULT_SCENE,
    &MIRA_REPORT_SCENE,
    &MIRA_DEFAULT_SCENE,
    &VALE_DEFAULT_SCENE,
    &SEL_ASH_ON_WAX_SCENE,
    &SEL_CROWN_IN_CINDERS_SCENE,
    &SEL_GRAVEWIND_SCENE,
    &SEL_DEFAULT_SCENE,
    &BRIN_DEFAULT_SCENE,
];

#[allow(dead_code)]
pub fn audio_filename(id: &str) -> Option<&'static str> {
    for scene in ALL_SCENES {
        if scene.id == id {
            return Some(scene.audio_filename);
        }
        for choice in scene.choices {
            if let (Some(audio_id), Some(filename)) = (choice.audio_id, choice.audio_filename) {
                if audio_id == id {
                    return Some(filename);
                }
            }
        }
    }
    None
}

pub fn all_voice_lines() -> Vec<VoiceLine> {
    let mut lines = Vec::new();
    for scene in ALL_SCENES {
        lines.push(VoiceLine {
            id: scene.id,
            filename: scene.audio_filename,
            model: scene.audio_model,
            text: scene.lines.join(" "),
        });
        for choice in scene.choices {
            if let (Some(audio_id), Some(filename), Some(model)) =
                (choice.audio_id, choice.audio_filename, choice.audio_model)
            {
                lines.push(VoiceLine {
                    id: audio_id,
                    filename,
                    model,
                    text: choice.response_lines.join(" "),
                });
            }
        }
    }
    lines
}
