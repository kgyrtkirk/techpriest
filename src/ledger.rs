//! Session-scoped tallies of heresy, persisted between hook invocations.

use std::fs;
use std::path::{Path, PathBuf};

/// What the tallies say about this session after the current command.
#[derive(Debug, PartialEq, Eq)]
pub struct Tallies {
    /// Heresies committed this session, all kinds together.
    pub total: u32,
    /// Times each convicted heresy has now been committed, in conviction order.
    pub repeats: Vec<u32>,
}

/// The tallies of one session, kept in a directory of counter files.
///
/// Counting is best-effort: a lost or corrupt counter reads as zero, because a
/// broken tally must never suppress the verdict it decorates.
pub struct Ledger {
    dir: PathBuf,
}

impl Ledger {
    pub fn for_session(session: &str) -> Self {
        Ledger::in_dir(std::env::temp_dir().join(format!("claude-heresy-{session}")))
    }

    pub fn in_dir(dir: impl Into<PathBuf>) -> Self {
        Ledger { dir: dir.into() }
    }

    /// Adds one conviction per key and reports the new state of the tallies.
    pub fn record<'k>(&self, keys: impl IntoIterator<Item = &'k str>) -> Tallies {
        let _ = fs::create_dir_all(&self.dir);
        let repeats: Vec<u32> =
            keys.into_iter().map(|key| self.bump(&self.dir.join(format!("{key}.count")), 1)).collect();
        Tallies { total: self.bump(&self.dir.join("total"), repeats.len() as u32), repeats }
    }

    fn bump(&self, counter: &Path, by: u32) -> u32 {
        let next = read(counter).saturating_add(by);
        let _ = fs::write(counter, next.to_string());
        next
    }
}

fn read(counter: &Path) -> u32 {
    fs::read_to_string(counter).ok().and_then(|body| body.trim().parse().ok()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn totals_accumulate_across_invocations() {
        let dir = tempfile::tempdir().unwrap();
        let ledger = Ledger::in_dir(dir.path());

        assert_eq!(ledger.record(["cd-self", "useless-cat"]).total, 2);
        assert_eq!(ledger.record(["truncation"]).total, 3);
    }

    #[test]
    fn repeats_count_each_kind_separately() {
        let dir = tempfile::tempdir().unwrap();
        let ledger = Ledger::in_dir(dir.path());

        assert_eq!(ledger.record(["cd-self", "useless-cat"]).repeats, [1, 1]);
        assert_eq!(ledger.record(["useless-cat"]).repeats, [2]);
        assert_eq!(ledger.record(["cd-self", "useless-cat"]).repeats, [2, 3]);
    }

    #[test]
    fn a_rule_named_total_would_not_clobber_the_total() {
        let dir = tempfile::tempdir().unwrap();
        let ledger = Ledger::in_dir(dir.path());

        ledger.record(["total"]);
        assert_eq!(ledger.record(["total"]), Tallies { total: 2, repeats: vec![2] });
    }

    #[test]
    fn sessions_do_not_contaminate_each_other() {
        let dir = tempfile::tempdir().unwrap();
        Ledger::in_dir(dir.path().join("a")).record(["cd-self"]);

        assert_eq!(
            Ledger::in_dir(dir.path().join("b")).record(["cd-self"]),
            Tallies { total: 1, repeats: vec![1] });
    }

    #[test]
    fn a_corrupt_counter_reads_as_zero() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("total"), "heretical nonsense").unwrap();

        assert_eq!(Ledger::in_dir(dir.path()).record(["cd-self"]).total, 1);
    }

    #[test]
    fn an_unwritable_ledger_still_yields_a_verdict() {
        assert_eq!(
            Ledger::in_dir("/proc/nonexistent/ledger").record(["cd-self"]),
            Tallies { total: 1, repeats: vec![1] });
    }
}
