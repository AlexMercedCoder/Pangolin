//! Shared search semantics for the four backends.
//!
//! B28: search behaved four different ways.
//!
//! * **Wildcards.** Postgres and SQLite built their patterns with
//!   `format!("%{}%", query)` and no escaping, so a query containing `%` or `_`
//!   was interpreted as a LIKE wildcard: searching for `100%` matched
//!   everything, and `a_b` matched `axb`. Mongo escaped correctly with
//!   `regex::escape`, and the memory backend used a literal `contains`. Four
//!   backends, four answers to the same query.
//! * **Tag filters.** Memory and SQLite matched *any* requested tag; Postgres
//!   (`@>`) and Mongo (`$all`) required *all* of them. And an empty tag list
//!   returned zero results on memory but everything on the others.
//!
//! This module is the single definition of both, so a backend can only diverge
//! by not calling it.

/// The escape character used with `LIKE ... ESCAPE`.
pub const LIKE_ESCAPE_CHAR: char = '\\';

/// Escape LIKE metacharacters in a user-supplied search term.
///
/// Must be paired with `ESCAPE '\'` in the SQL, which the helpers below embed.
pub fn escape_like(query: &str) -> String {
    let mut escaped = String::with_capacity(query.len());
    for ch in query.chars() {
        if ch == '%' || ch == '_' || ch == LIKE_ESCAPE_CHAR {
            escaped.push(LIKE_ESCAPE_CHAR);
        }
        escaped.push(ch);
    }
    escaped
}

/// Build a `%term%` LIKE pattern with metacharacters escaped.
pub fn contains_pattern(query: &str) -> String {
    format!("%{}%", escape_like(query))
}

/// The `ESCAPE` clause every `LIKE`/`ILIKE` using [`contains_pattern`] needs.
pub const ESCAPE_CLAUSE: &str = " ESCAPE '\\'";

/// Does `tags` satisfy a tag filter?
///
/// **The chosen semantic is ALL-match**: a result qualifies only if it carries
/// every requested tag. That is what Postgres's `@>` and Mongo's `$all` already
/// did, so aligning on it keeps the two SQL/document backends unchanged and
/// moves memory and SQLite - and it is the semantic faceted filtering wants,
/// where each added tag narrows the result set.
///
/// An **empty or absent** filter means "no tag constraint", matching everything.
/// Previously an empty list returned nothing on the memory backend and
/// everything elsewhere.
pub fn tags_match(tags: &[String], required: Option<&[String]>) -> bool {
    match required {
        None => true,
        Some(required) => required.iter().all(|want| tags.contains(want)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn like_metacharacters_are_escaped() {
        assert_eq!(escape_like("100%"), "100\\%");
        assert_eq!(escape_like("a_b"), "a\\_b");
        assert_eq!(escape_like("back\\slash"), "back\\\\slash");
        assert_eq!(escape_like("plain"), "plain");
    }

    #[test]
    fn contains_pattern_wraps_the_escaped_term() {
        assert_eq!(contains_pattern("50%"), "%50\\%%");
    }

    #[test]
    fn tag_filter_is_all_match() {
        let tags = vec!["pii".to_string(), "finance".to_string()];

        assert!(tags_match(&tags, None), "no filter matches everything");
        assert!(tags_match(&tags, Some(&[])), "an empty filter is no filter");
        assert!(tags_match(&tags, Some(&["pii".to_string()])));
        assert!(tags_match(
            &tags,
            Some(&["pii".to_string(), "finance".to_string()])
        ));
        assert!(
            !tags_match(&tags, Some(&["pii".to_string(), "hr".to_string()])),
            "every requested tag must be present"
        );
    }
}
