// Simple milestone system for snake score thresholds.
// Keep thresholds ascending; labels should fit the snake theme.

pub struct Milestone {
    pub score: u32,
    pub label: &'static str,
}

// Ordered small → large
pub const MILESTONES: &[Milestone] = &[
    Milestone {
        score: 5,
        label: "Snack Streak",
    },
    Milestone {
        score: 10,
        label: "Garden Glutton",
    },
    Milestone {
        score: 15,
        label: "Tunnel Tactician",
    },
    Milestone {
        score: 20,
        label: "Viper Velocity",
    },
    Milestone {
        score: 30,
        label: "Coil Commander",
    },
    Milestone {
        score: 40,
        label: "Shedmaster",
    },
    Milestone {
        score: 50,
        label: "Apex Adder",
    },
    Milestone {
        score: 75,
        label: "Mythscale",
    },
    Milestone {
        score: 100,
        label: "Ouro Ascends",
    },
];

/// Return the index and label of the highest milestone reached by `score`.
pub fn milestone_for_score(score: u32) -> Option<(usize, &'static str)> {
    let mut found: Option<(usize, &'static str)> = None;
    for (i, m) in MILESTONES.iter().enumerate() {
        if score >= m.score {
            found = Some((i, m.label));
        } else {
            break;
        }
    }
    found
}
