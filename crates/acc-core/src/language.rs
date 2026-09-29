use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Supported submission languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    C,
    Cpp17,
    Python3,
    Java,
    Rust,
}

impl Language {
    pub const ALL: [Language; 5] = [
        Language::C,
        Language::Cpp17,
        Language::Python3,
        Language::Java,
        Language::Rust,
    ];

    /// Stable id used in the DB, URLs and the API.
    pub fn id(self) -> &'static str {
        match self {
            Language::C => "c",
            Language::Cpp17 => "cpp17",
            Language::Python3 => "python3",
            Language::Java => "java",
            Language::Rust => "rust",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Language::C => "C",
            Language::Cpp17 => "C++17",
            Language::Python3 => "Python 3",
            Language::Java => "Java",
            Language::Rust => "Rust",
        }
    }

    /// Source file name inside the sandbox.
    pub fn source_file(self) -> &'static str {
        match self {
            Language::C => "main.c",
            Language::Cpp17 => "main.cpp",
            Language::Python3 => "main.py",
            Language::Java => "Main.java",
            Language::Rust => "main.rs",
        }
    }

    /// Time limit adjustment: `limit * mul + add_ms`.
    ///
    /// Provisional until measured on the production server (plan M6).
    pub fn time_factor(self) -> (u64, u64) {
        match self {
            Language::C | Language::Cpp17 | Language::Rust => (1, 0),
            Language::Java => (2, 1000),
            Language::Python3 => (3, 2000),
        }
    }

    /// Effective time limit in ms for this language.
    pub fn adjusted_time_ms(self, limit_ms: u64) -> u64 {
        let (mul, add) = self.time_factor();
        limit_ms * mul + add
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

impl FromStr for Language {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Language::ALL
            .into_iter()
            .find(|l| l.id() == s)
            .ok_or_else(|| format!("unknown language: {s}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_roundtrip() {
        for l in Language::ALL {
            assert_eq!(l.id().parse::<Language>().unwrap(), l);
            assert_eq!(
                serde_json::to_string(&l).unwrap(),
                format!("\"{}\"", l.id())
            );
        }
    }

    #[test]
    fn adjusted() {
        assert_eq!(Language::Cpp17.adjusted_time_ms(1000), 1000);
        assert_eq!(Language::Python3.adjusted_time_ms(1000), 5000);
    }
}
