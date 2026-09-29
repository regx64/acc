//! Output comparison: trailing whitespace on each line and trailing blank
//! lines at the end of the file are ignored; everything else must match.

/// Returns true when `actual` is accepted against `expected`.
pub fn outputs_match(expected: &[u8], actual: &[u8]) -> bool {
    let mut e = normalized_lines(expected);
    let mut a = normalized_lines(actual);
    loop {
        match (e.next(), a.next()) {
            (None, None) => return true,
            (Some(x), Some(y)) if x == y => continue,
            _ => return false,
        }
    }
}

/// Lines with trailing whitespace removed, stopping before trailing blank lines.
fn normalized_lines(data: &[u8]) -> impl Iterator<Item = &[u8]> {
    let mut lines: Vec<&[u8]> = data.split(|&b| b == b'\n').map(trim_end).collect();
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    lines.into_iter()
}

fn trim_end(line: &[u8]) -> &[u8] {
    let mut end = line.len();
    while end > 0 && matches!(line[end - 1], b' ' | b'\t' | b'\r') {
        end -= 1;
    }
    &line[..end]
}

#[cfg(test)]
mod tests {
    use super::outputs_match as m;

    #[test]
    fn exact() {
        assert!(m(b"1 2\n3\n", b"1 2\n3\n"));
    }

    #[test]
    fn trailing_space_and_newlines() {
        assert!(m(b"1 2\n3\n", b"1 2   \n3"));
        assert!(m(b"1 2\n3", b"1 2\r\n3\r\n\n\n"));
        assert!(m(b"", b"\n\n"));
    }

    #[test]
    fn inner_differences() {
        assert!(!m(b"1 2\n", b"1  2\n"));
        assert!(!m(b"1\n2\n", b"1\n\n2\n"));
        assert!(!m(b"1\n", b" 1\n"));
        assert!(!m(b"1\n2\n", b"1\n"));
        assert!(!m(b"1\n", b"1\n2\n"));
    }
}
