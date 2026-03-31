use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AchievementDef {
    pub id: String,
    pub category: &'static str,
    pub name: String,
    pub description: String,
    pub metric: &'static str,
    pub target: i32,
}

#[derive(Debug, Clone, Default)]
pub struct AchievementState {
    pub metrics: HashMap<String, i32>,
    pub unlocked_ids: Vec<String>,
}

impl AchievementState {
    pub fn progress_for(&self, metric: &str) -> i32 {
        *self.metrics.get(metric).unwrap_or(&0)
    }

    pub fn progress_toward(&self, achievement_id: &str) -> i32 {
        find_achievement(achievement_id)
            .map(|def| self.progress_for(def.metric).min(def.target))
            .unwrap_or(0)
    }

    pub fn is_unlocked(&self, achievement_id: &str) -> bool {
        self.unlocked_ids.iter().any(|id| id == achievement_id)
    }

    pub fn unlocked_count(&self) -> usize {
        self.unlocked_ids.len()
    }

    pub fn record_increment(&mut self, metric: &str, amount: i32) -> Vec<AchievementDef> {
        if amount <= 0 {
            return vec![];
        }
        let next = self.progress_for(metric) + amount;
        self.metrics.insert(metric.to_string(), next);
        self.refresh_unlocks()
    }

    pub fn record_max(&mut self, metric: &str, value: i32) -> Vec<AchievementDef> {
        if value <= self.progress_for(metric) {
            return vec![];
        }
        self.metrics.insert(metric.to_string(), value);
        self.refresh_unlocks()
    }

    pub fn recompute_unlocked(&mut self) {
        self.unlocked_ids = achievement_defs()
            .into_iter()
            .filter(|def| self.progress_for(def.metric) >= def.target)
            .map(|def| def.id)
            .collect();
        self.unlocked_ids.sort();
        self.unlocked_ids.dedup();
    }

    fn refresh_unlocks(&mut self) -> Vec<AchievementDef> {
        let mut newly_unlocked = vec![];
        for def in achievement_defs() {
            if self.progress_for(def.metric) < def.target || self.is_unlocked(&def.id) {
                continue;
            }
            self.unlocked_ids.push(def.id.clone());
            newly_unlocked.push(def);
        }
        newly_unlocked
    }
}

struct MetricSeries {
    metric: &'static str,
    category: &'static str,
    series: &'static str,
    verb: &'static str,
    noun: &'static str,
    thresholds: [i32; 5],
}

const SERIES: [MetricSeries; 20] = [
    MetricSeries {
        metric: "combat_victories",
        category: "Combat",
        series: "Victor of the Wilds",
        verb: "Win",
        noun: "combat victories",
        thresholds: [1, 3, 5, 10, 25],
    },
    MetricSeries {
        metric: "enemy_kills",
        category: "Combat",
        series: "Slayer's Ledger",
        verb: "Defeat",
        noun: "enemies",
        thresholds: [1, 5, 15, 40, 100],
    },
    MetricSeries {
        metric: "beast_kills",
        category: "Combat",
        series: "Beast Hunter",
        verb: "Defeat",
        noun: "beasts",
        thresholds: [1, 3, 10, 25, 50],
    },
    MetricSeries {
        metric: "bandit_kills",
        category: "Combat",
        series: "Road Warden",
        verb: "Defeat",
        noun: "bandits",
        thresholds: [1, 3, 10, 25, 50],
    },
    MetricSeries {
        metric: "undead_kills",
        category: "Combat",
        series: "Gravebreaker",
        verb: "Defeat",
        noun: "undead",
        thresholds: [1, 3, 10, 25, 50],
    },
    MetricSeries {
        metric: "damage_dealt",
        category: "Combat",
        series: "Relentless Edge",
        verb: "Deal",
        noun: "total damage",
        thresholds: [25, 100, 250, 600, 1200],
    },
    MetricSeries {
        metric: "ability_uses",
        category: "Combat",
        series: "Arc of Technique",
        verb: "Use",
        noun: "abilities",
        thresholds: [1, 10, 25, 60, 120],
    },
    MetricSeries {
        metric: "weapon_attacks",
        category: "Combat",
        series: "Steel Rhythm",
        verb: "Land",
        noun: "weapon attacks",
        thresholds: [1, 10, 25, 60, 120],
    },
    MetricSeries {
        metric: "item_uses",
        category: "Combat",
        series: "Battlefield Ready",
        verb: "Use",
        noun: "combat items",
        thresholds: [1, 5, 10, 20, 40],
    },
    MetricSeries {
        metric: "study_sessions",
        category: "Skills",
        series: "Dedicated Student",
        verb: "Complete",
        noun: "study sessions",
        thresholds: [1, 5, 10, 20, 40],
    },
    MetricSeries {
        metric: "study_hours",
        category: "Skills",
        series: "Long Vigil",
        verb: "Spend",
        noun: "hours studying",
        thresholds: [4, 20, 60, 150, 300],
    },
    MetricSeries {
        metric: "study_successes",
        category: "Skills",
        series: "Earned Insight",
        verb: "Succeed at",
        noun: "study sessions",
        thresholds: [1, 5, 10, 25, 50],
    },
    MetricSeries {
        metric: "best_proficiency_rank",
        category: "Skills",
        series: "Master Craft",
        verb: "Reach",
        noun: "a proficiency rank of",
        thresholds: [5, 10, 20, 50, 100],
    },
    MetricSeries {
        metric: "level_reached",
        category: "Progression",
        series: "Rising Legend",
        verb: "Reach",
        noun: "character level",
        thresholds: [2, 5, 10, 15, 20],
    },
    MetricSeries {
        metric: "abilities_unlocked",
        category: "Progression",
        series: "Arsenal of Tricks",
        verb: "Unlock",
        noun: "abilities",
        thresholds: [1, 3, 5, 7, 9],
    },
    MetricSeries {
        metric: "equipment_slots_filled",
        category: "Equipment",
        series: "Fully Armed",
        verb: "Fill",
        noun: "equipment slots",
        thresholds: [1, 3, 5, 8, 10],
    },
    MetricSeries {
        metric: "items_equipped",
        category: "Equipment",
        series: "Quartermaster's Pride",
        verb: "Equip",
        noun: "items",
        thresholds: [1, 5, 10, 20, 40],
    },
    MetricSeries {
        metric: "gold_earned",
        category: "Adventure",
        series: "Treasure Seeker",
        verb: "Earn",
        noun: "gold",
        thresholds: [25, 100, 250, 600, 1500],
    },
    MetricSeries {
        metric: "gold_spent",
        category: "Adventure",
        series: "Patron of Hearthmere",
        verb: "Spend",
        noun: "gold",
        thresholds: [25, 100, 250, 600, 1500],
    },
    MetricSeries {
        metric: "rests_taken",
        category: "World",
        series: "Well Rested",
        verb: "Take",
        noun: "inn rests",
        thresholds: [1, 3, 7, 15, 30],
    },
];

pub fn achievement_defs() -> Vec<AchievementDef> {
    let numerals = ["I", "II", "III", "IV", "V"];
    let mut defs = Vec::with_capacity(100);
    for series in SERIES {
        for (idx, target) in series.thresholds.into_iter().enumerate() {
            let name = format!("{} {}", series.series, numerals[idx]);
            let description = if matches!(series.metric, "best_proficiency_rank" | "level_reached")
            {
                format!("{} {} {}.", series.verb, series.noun, target)
            } else {
                format!("{} {} {}.", series.verb, target, series.noun)
            };
            defs.push(AchievementDef {
                id: format!("{}_{}", series.metric, idx + 1),
                category: series.category,
                name,
                description,
                metric: series.metric,
                target,
            });
        }
    }
    defs
}

pub fn find_achievement(id: &str) -> Option<AchievementDef> {
    achievement_defs().into_iter().find(|def| def.id == id)
}

#[cfg(test)]
mod tests;
