//! Levels 0..=30, named by number-system groups ℕ ℤ ℚ ℝ ℂ ℍ × 5 steps.

use serde::Serialize;

pub const MAX_LEVEL: u8 = 30;
/// `tier(L) ⇔ rating ≥ TIER_COEFF · L²`. Kept as a constant so it can be
/// tuned once real distribution data exists.
pub const TIER_COEFF: i64 = 90;
/// Rating counts the scores of this many best solved problems.
pub const RATED_PROBLEMS: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Group {
    pub symbol: &'static str,
    /// English name, used in URLs and the API instead of the symbol.
    pub name: &'static str,
    pub color: &'static str,
}

pub const UNRATED: Group = Group {
    symbol: "?",
    name: "Unrated",
    color: "#868e96",
};
pub const GROUPS: [Group; 6] = [
    Group {
        symbol: "ℕ",
        name: "Natural",
        color: "#5c940d",
    },
    Group {
        symbol: "ℤ",
        name: "Integer",
        color: "#0c8599",
    },
    Group {
        symbol: "ℚ",
        name: "Rational",
        color: "#1c7ed6",
    },
    Group {
        symbol: "ℝ",
        name: "Real",
        color: "#6741d9",
    },
    Group {
        symbol: "ℂ",
        name: "Complex",
        color: "#ae3ec9",
    },
    Group {
        symbol: "ℍ",
        name: "Quaternion",
        color: "#212529",
    },
];

/// Group of a level (Unrated for 0).
pub fn group(level: u8) -> Group {
    if level == 0 || level > MAX_LEVEL {
        UNRATED
    } else {
        GROUPS[usize::from((level - 1) / 5)]
    }
}

/// Step 1..=5 inside the group (0 for Unrated).
pub fn step(level: u8) -> u8 {
    if level == 0 || level > MAX_LEVEL {
        0
    } else {
        (level - 1) % 5 + 1
    }
}

/// Human label such as "ℚ3" or "Unrated".
pub fn label(level: u8) -> String {
    match level {
        0 => "Unrated".into(),
        l => format!("{}{}", group(l).symbol, step(l)),
    }
}

/// Score of solving a problem of this level.
pub fn score(level: u8) -> i64 {
    i64::from(level).pow(2)
}

/// Rating from the levels of all solved problems.
pub fn rating<I: IntoIterator<Item = u8>>(levels: I) -> i64 {
    let mut scores: Vec<i64> = levels.into_iter().map(score).filter(|&s| s > 0).collect();
    scores.sort_unstable_by(|a, b| b.cmp(a));
    scores.into_iter().take(RATED_PROBLEMS).sum()
}

/// Tier for a rating: the largest L with rating ≥ 90·L², and tier 1 for any
/// positive rating. 0 means no tier yet.
pub fn tier(rating: i64) -> u8 {
    if rating < 1 {
        return 0;
    }
    (1..=MAX_LEVEL)
        .rev()
        .find(|&l| rating >= TIER_COEFF * score(l))
        .unwrap_or(1)
}

/// Rating needed for a tier.
pub fn tier_threshold(tier: u8) -> i64 {
    match tier {
        0 => 0,
        1 => 1,
        l => TIER_COEFF * score(l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_and_steps() {
        assert_eq!(label(0), "Unrated");
        assert_eq!(label(1), "ℕ1");
        assert_eq!(label(5), "ℕ5");
        assert_eq!(label(6), "ℤ1");
        assert_eq!(label(13), "ℚ3");
        assert_eq!(label(30), "ℍ5");
        assert_eq!(group(26).name, "Quaternion");
    }

    #[test]
    fn rating_takes_top_100() {
        assert_eq!(rating([]), 0);
        assert_eq!(rating([0, 0]), 0);
        assert_eq!(rating([3, 2]), 13);
        assert_eq!(rating(std::iter::repeat_n(30, 150)), 90_000);
        let mut v = vec![1u8; 100];
        v.push(10);
        assert_eq!(rating(v), 99 + 100);
    }

    #[test]
    fn tiers() {
        assert_eq!(tier(0), 0);
        assert_eq!(tier(1), 1);
        assert_eq!(tier(359), 1);
        assert_eq!(tier(360), 2);
        assert_eq!(tier(90 * 225), 15);
        assert_eq!(tier(90 * 225 - 1), 14);
        assert_eq!(tier(90_000), 30);
        // 90 problems of level L reach tier L.
        for l in 1..=30u8 {
            assert_eq!(tier(rating(std::iter::repeat_n(l, 90))), l);
        }
        assert_eq!(tier_threshold(1), 1);
        assert_eq!(tier_threshold(30), 81_000);
    }
}
