pub mod alarms;
pub mod audit;
pub mod devices;
pub mod measurements;
pub mod overview;
pub mod rooms;
pub mod service_calls;
pub mod sessions;
pub mod stays;
pub mod thresholds;
pub mod users;

/// Prepares user input for the SQL `ILIKE` pattern used in `list`.
///
/// In SQL pattern matching, `%` means "any characters" and `_` means "any single
/// character". If a user searches for "50%" or "j_hansen", those characters must be
/// treated literally, so they are escaped with a backslash before being wrapped in
/// `%…%` for the contains-match.
pub(crate) fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[cfg(test)]
mod tests {
    use super::escape_like;

    #[test]
    fn like_wildcards_are_escaped() {
        assert_eq!(escape_like("50%"), "50\\%");
        assert_eq!(escape_like("j_hansen"), "j\\_hansen");
        assert_eq!(escape_like("a\\b"), "a\\\\b");
        assert_eq!(escape_like("plain"), "plain");
    }
}
