//! Redis queue layout shared by the API and the worker.

/// Pending jobs. The API pushes on the left, workers pop from the right.
pub const QUEUE: &str = "acc:queue";
/// Per-worker list of the job currently being processed.
pub fn processing(worker: &str) -> String {
    format!("acc:processing:{worker}")
}

/// A queued unit of work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Job {
    /// Judge a submission.
    Submission(i64),
    /// Check a problem's reference solution against its test data.
    Validate(i32),
}

impl Job {
    pub fn encode(self) -> String {
        match self {
            Job::Submission(id) => format!("s:{id}"),
            Job::Validate(id) => format!("v:{id}"),
        }
    }

    pub fn decode(s: &str) -> Option<Job> {
        let (kind, id) = s.split_once(':')?;
        match kind {
            "s" => id.parse().ok().map(Job::Submission),
            "v" => id.parse().ok().map(Job::Validate),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Job;

    #[test]
    fn roundtrip() {
        for j in [Job::Submission(42), Job::Validate(1000)] {
            assert_eq!(Job::decode(&j.encode()), Some(j));
        }
        assert_eq!(Job::decode("x:1"), None);
        assert_eq!(Job::decode("s:abc"), None);
    }
}
