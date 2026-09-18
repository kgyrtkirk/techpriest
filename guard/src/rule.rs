//! The shape every heresy takes, whichever catalogue registers it.

/// A heresy: what the act is, what it costs, the directive it breaks, the
/// correct incantation, and the test that convicts it.
///
/// `J` is what the rule judges — a shell command, a doc comment.
pub struct Rule<J> {
    /// Stable key for tallies — never reword it, or the session's count resets.
    pub id: &'static str,
    pub what: &'static str,
    pub why: &'static str,
    pub directive: &'static str,
    pub fix: &'static str,
    pub(crate) convicts: fn(&J) -> bool,
}

impl<J> Rule<J> {
    pub fn convicts(&self, subject: &J) -> bool {
        (self.convicts)(subject)
    }
}

#[cfg(test)]
pub fn assert_well_formed<J>(catalogue: &[Rule<J>]) {
    let mut seen = Vec::new();
    for rule in catalogue {
        assert!(!seen.contains(&rule.id), "duplicate rule id {}", rule.id);
        seen.push(rule.id);
        assert!(
            rule.id.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
            "{} is not a kebab-case id",
            rule.id);
        for (field, value) in [("what", rule.what), ("why", rule.why), ("directive", rule.directive), ("fix", rule.fix)] {
            assert!(!value.is_empty(), "{} lacks a {field}", rule.id);
        }
    }
}
