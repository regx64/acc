//! Handle rules: lowercase ASCII letters, digits and underscore, 3–20 chars.

pub const MIN_LEN: usize = 3;
pub const MAX_LEN: usize = 20;
/// A handle may be changed once per this many days.
pub const CHANGE_INTERVAL_DAYS: i64 = 90;
/// A released handle stays locked (and redirects) for this many days.
pub const RELEASE_LOCK_DAYS: i64 = 180;

pub fn is_valid(handle: &str) -> bool {
    (MIN_LEN..=MAX_LEN).contains(&handle.len())
        && handle
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

#[cfg(test)]
mod tests {
    use super::is_valid;

    #[test]
    fn rules() {
        assert!(is_valid("regx64"));
        assert!(is_valid("a_b"));
        assert!(!is_valid("ab"));
        assert!(!is_valid("Regx"));
        assert!(!is_valid("한글핸들"));
        assert!(!is_valid("a-b-c"));
        assert!(!is_valid(&"a".repeat(21)));
    }
}
