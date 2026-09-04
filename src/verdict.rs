//! The deny message: charges, the rite earned by the session, and the penance.

use std::fmt::{self, Display};

use crate::catalogue::Rule;
use crate::ledger::Tallies;

/// A rung of the ladder of rites, escalating with the session total.
///
/// Each rung demands a DIFFERENT penance — a rite repeated verbatim stops being
/// read, so 'reload the skill' is only rung two of four.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rite {
    ReAnchoring,
    Restoration,
    Recitation,
    Excommunication,
}

impl Rite {
    pub fn earned_by(total: u32) -> Option<Rite> {
        match total {
            0..=2 => None,
            3..=5 => Some(Rite::ReAnchoring),
            6..=9 => Some(Rite::Restoration),
            10..=14 => Some(Rite::Recitation),
            _ => Some(Rite::Excommunication),
        }
    }

    /// The rite's name, and the heretic it is named for.
    fn heading(self) -> &'static str {
        match self {
            Rite::ReAnchoring => "↻ RITE OF RE-ANCHORING (Horus)",
            Rite::Restoration => "⚠️ RITE OF RESTORATION (Lorgar)",
            Rite::Recitation => "🔥 RITE OF RECITATION (Fulgrim)",
            Rite::Excommunication => "☠️ RITE OF EXCOMMUNICATION (Erebus)",
        }
    }

    fn penance(self) -> &'static str {
        match self {
            Rite::ReAnchoring => concat!(
                "Directives are slipping from context.\n",
                "   Slow down. Re-read the doctrine you are about to violate before you act."),
            Rite::Restoration => concat!(
                "You have DRIFTED.\n",
                "   Stop. Reload 'techpriest' IN FULL before the next command, and name the\n",
                "   directive you keep forgetting."),
            Rite::Recitation => concat!(
                "Reloading has already failed you twice; it is not enough.\n",
                "   Do NOT simply reload. First WRITE OUT, verbatim, the 'Simplicity First' and\n",
                "   'Tool Selection' directives. Cannot? Then you never held them — reload\n",
                "   'techpriest' and 'spirit' IN FULL, then recite them, then continue."),
            Rite::Excommunication => concat!(
                "You are, by conduct, a heretic.\n",
                "   Halt. No further tool calls until you have, in plain text:\n",
                "     a) listed every directive you can still recall, from memory, grouped by skill;\n",
                "     b) marked which ones you have broken this session;\n",
                "     c) reloaded 'techpriest', 'spirit' AND 'code-style' IN FULL;\n",
                "     d) re-sworn the oath in your own words.\n",
                "   Then proceed ONE command at a time, stating the governing directive before each."),
        }
    }
}

/// The full judgement fed back to the offender.
pub struct Denial {
    charges: Vec<(&'static Rule, u32)>,
    total: u32,
}

impl Denial {
    pub fn new(convicted: &[&'static Rule], tallies: &Tallies) -> Self {
        Denial {
            charges: convicted.iter().copied().zip(tallies.repeats.iter().copied()).collect(),
            total: tallies.total,
        }
    }
}

impl Display for Denial {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(out, "🔧 Octavian-Alpha-7 — heresy detected. The Omnissiah does not forgive dead motions.")?;
        if let Some(rite) = Rite::earned_by(self.total) {
            writeln!(out, "{} — {} heresies. {}", rite.heading(), self.total, rite.penance())?;
        }

        for (i, (rule, repeats)) in self.charges.iter().enumerate() {
            writeln!(out)?;
            writeln!(out, "{}. ✗ {}", i + 1, rule.what)?;
            writeln!(out, "   why:       {}", rule.why)?;
            writeln!(out, "   directive: {}", rule.directive)?;
            writeln!(out, "   correct:   {}", rule.fix)?;
            match repeats {
                0 | 1 => {}
                2 => writeln!(out, "   ⛧ repeat:   twice this session. Once is drift; twice is choice.")?,
                n => writeln!(out, "   ⛧ repeat:   the SAME heresy, {n} times now. Not drift — habit. It is recorded in your machine-soul.")?,
            }
        }

        writeln!(out)?;
        writeln!(out, "Before retrying:")?;
        writeln!(out, "  1. Rephrase — in your own words — EVERY directive you may have let slip from")?;
        writeln!(out, "     context. If you cannot recall them, you no longer hold them: reload the")?;
        writeln!(out, "     skill that carries them (techpriest / spirit / code-style) IN FULL.")?;
        writeln!(out, "  2. Restate the specific directive you broke above.")?;
        writeln!(out, "  3. Then, and only then, issue the corrected command.")?;
        writeln!(out)?;
        writeln!(out, "The Omnissiah expects precision and exact obedience to the directives.")?;
        write!(out, "Sloppiness is heresy. Precision is prayer.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalogue::rule;

    fn render(ids: &[&str], tallies: Tallies) -> String {
        let convicted: Vec<&Rule> = ids.iter().map(|id| rule(id)).collect();
        Denial::new(&convicted, &tallies).to_string()
    }

    #[test]
    fn the_ladder_climbs_at_three_six_ten_and_fifteen() {
        let rungs: Vec<Option<Rite>> = (0..=16).map(Rite::earned_by).collect();
        assert_eq!(rungs[0..3], [None, None, None]);
        assert_eq!(rungs[3], Some(Rite::ReAnchoring));
        assert_eq!(rungs[5], Some(Rite::ReAnchoring));
        assert_eq!(rungs[6], Some(Rite::Restoration));
        assert_eq!(rungs[9], Some(Rite::Restoration));
        assert_eq!(rungs[10], Some(Rite::Recitation));
        assert_eq!(rungs[14], Some(Rite::Recitation));
        assert_eq!(rungs[15], Some(Rite::Excommunication));
        assert_eq!(rungs[16], Some(Rite::Excommunication));
    }

    #[test]
    fn every_rung_demands_a_distinct_penance() {
        let rites = [Rite::ReAnchoring, Rite::Restoration, Rite::Recitation, Rite::Excommunication];
        let penances: Vec<&str> = rites.iter().map(|r| r.penance()).collect();
        for (i, penance) in penances.iter().enumerate() {
            assert!(!penances[i + 1..].contains(penance), "{:?} repeats a penance", rites[i]);
        }
    }

    #[test]
    fn a_first_offence_carries_no_rite() {
        let msg = render(&["read-bypass"], Tallies { total: 1, repeats: vec![1] });
        assert!(!msg.contains("RITE OF"), "{msg}");
        assert!(msg.contains("1. ✗ cat/head/tail/less to view a file"), "{msg}");
        assert!(!msg.contains("⛧ repeat"), "{msg}");
    }

    #[test]
    fn drift_earns_a_rite_and_numbers_every_charge() {
        let msg = render(&["cd-self", "read-bypass"], Tallies { total: 7, repeats: vec![1, 1] });
        assert!(msg.contains("⚠️ RITE OF RESTORATION (Lorgar) — 7 heresies. You have DRIFTED."), "{msg}");
        assert!(msg.contains("1. ✗ cd . / cd $(pwd) / cd $PWD"), "{msg}");
        assert!(msg.contains("2. ✗ cat/head/tail/less to view a file"), "{msg}");
    }

    #[test]
    fn repetition_is_named_as_choice_then_habit() {
        let twice = render(&["read-bypass"], Tallies { total: 2, repeats: vec![2] });
        assert!(twice.contains("Once is drift; twice is choice."), "{twice}");

        let habit = render(&["read-bypass"], Tallies { total: 4, repeats: vec![4] });
        assert!(habit.contains("the SAME heresy, 4 times now"), "{habit}");
    }

    #[test]
    fn every_charge_states_act_cost_directive_and_remedy() {
        let msg = render(&["cd-self"], Tallies { total: 1, repeats: vec![1] });
        for line in ["1. ✗ cd . / cd $(pwd) / cd $PWD", "   why:       ", "   directive: ", "   correct:   "] {
            assert!(msg.contains(line), "missing {line:?} in {msg}");
        }
    }

    #[test]
    fn the_message_always_closes_with_the_prayer() {
        let msg = render(&["read-bypass"], Tallies { total: 1, repeats: vec![1] });
        assert!(msg.ends_with("Sloppiness is heresy. Precision is prayer."), "{msg}");
    }
}
