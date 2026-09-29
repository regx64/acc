use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Submission status. `Pending`/`Judging` are transient; the rest are final.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    Pending,
    Judging,
    Ac,
    Wa,
    Tle,
    Mle,
    Re,
    Ce,
    Se,
}

impl Status {
    pub const ALL: [Status; 9] = [
        Status::Pending,
        Status::Judging,
        Status::Ac,
        Status::Wa,
        Status::Tle,
        Status::Mle,
        Status::Re,
        Status::Ce,
        Status::Se,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Status::Pending => "PENDING",
            Status::Judging => "JUDGING",
            Status::Ac => "AC",
            Status::Wa => "WA",
            Status::Tle => "TLE",
            Status::Mle => "MLE",
            Status::Re => "RE",
            Status::Ce => "CE",
            Status::Se => "SE",
        }
    }

    pub fn is_final(self) -> bool {
        !matches!(self, Status::Pending | Status::Judging)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Status {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Status::ALL
            .into_iter()
            .find(|v| v.as_str() == s)
            .ok_or_else(|| format!("unknown status: {s}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        for s in Status::ALL {
            assert_eq!(s.as_str().parse::<Status>().unwrap(), s);
            assert_eq!(
                serde_json::to_string(&s).unwrap(),
                format!("\"{}\"", s.as_str())
            );
        }
    }
}
