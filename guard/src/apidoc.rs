//! The catalogue of apidoc heresies, judged on the doc comments a Java edit
//! writes or rewrites.
//!
//! To add an apidoc heresy, add one `Rule` here. Its written form lives in the
//! code-style rite; change one, change the other.

use std::collections::HashSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::javadoc::{self, Doc};
use crate::rule::Rule;

/// The most words a mission statement may spend. Rule `apidoc-long-mission` names it.
const MAX_MISSION_WORDS: usize = 20;

pub static APIDOC: &[Rule<Doc>] = &[
    Rule {
        id: "apidoc-missing-mission",
        what: "apidoc without a mission statement",
        why: "bare tags leave the reader to reassemble the contract from its parts",
        directive: "Code-style rite: 'First sentence stands alone: a clear, short mission statement ending with a `.`'",
        fix: "open the doc with one short sentence stating the contract, closed by '.'",
        convicts: |doc| doc.lead.is_empty(),
    },
    Rule {
        id: "apidoc-unterminated-mission",
        what: "apidoc mission statement not closed by '.'",
        why: "javadoc cuts the summary at the first '. ' — without one the summary is a fragment or runs on",
        directive: "Code-style rite: 'First sentence stands alone: a clear, short mission statement ending with a `.`'",
        fix: "close the first sentence with '.'; a lead ending in ':' before a list does not stand alone",
        convicts: |doc| !doc.lead.is_empty() && doc.mission().is_none(),
    },
    Rule {
        id: "apidoc-long-mission",
        what: "apidoc mission statement over 20 words",
        why: "the first sentence is the summary every index and hover shows — reader's time is priceless",
        directive: "Code-style rite: 'First sentence stands alone: a clear, short mission statement ending with a `.`'",
        fix: "cut the first sentence to the contract alone; detail goes to a later paragraph, or into a better name",
        convicts: |doc| doc.mission().is_some_and(|mission| words(mission) > MAX_MISSION_WORDS),
    },
];

pattern!(INLINE_TAG = r"\{@[^{}]*\}");
pattern!(HTML_TAG = r"<[^>]*>");

/// Words as the reader sees them: an inline tag reads as one, markup as none.
fn words(text: &str) -> usize {
    let tags_collapsed = INLINE_TAG.replace_all(text, "#");
    HTML_TAG.replace_all(&tags_collapsed, "").split_whitespace().count()
}

/// The doc comments an edit turning `before` into `after` answers for.
///
/// A doc is the editor's once its lead is new — a lead-less doc once any of it
/// is. Docs deferring to `{@inheritDoc}` state no mission of their own.
pub fn judged(path: &str, before: &str, after: &str) -> Vec<Doc> {
    let known: HashSet<String> = javadoc::docs(path, before).iter().map(|doc| key(doc).to_string()).collect();
    javadoc::docs(path, after)
        .into_iter()
        .filter(|doc| !doc.inherits() && !known.contains(key(doc)))
        .collect()
}

fn key(doc: &Doc) -> &str {
    if doc.lead.is_empty() { &doc.body } else { &doc.lead }
}

/// Convicts every heresy the docs commit, in catalogue order.
pub fn detect(docs: &[Doc]) -> Vec<&'static Rule<Doc>> {
    APIDOC.iter().filter(|rule| docs.iter().any(|doc| rule.convicts(doc))).collect()
}

/// The opening line of every doc the rule convicts, so the offender can find them.
pub fn cite(rule: &Rule<Doc>, docs: &[Doc]) -> Vec<String> {
    docs.iter().filter(|doc| rule.convicts(doc)).map(|doc| doc.opening().to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Twenty-three words.
    const LONG: &str = "/** Returns the segments of the data source that overlap the given interval, \
                        ordered first by their version and then by their partition number. */";

    /// One doc per rule, proving the rule is reachable at all.
    const SAMPLES: &[(&str, &str)] = &[
        ("apidoc-missing-mission", "/** @param x the x */"),
        ("apidoc-unterminated-mission", "/** Returns the foo */"),
        ("apidoc-long-mission", LONG),
    ];

    fn judge(source: &str) -> Vec<&'static str> {
        detect(&judged("Foo.java", "", source)).iter().map(|rule| rule.id).collect()
    }

    #[test]
    fn every_rule_is_reachable_by_a_sample() {
        for (id, source) in SAMPLES {
            assert!(judge(source).contains(id), "{source} should convict {id}");
        }
    }

    #[test]
    fn every_rule_has_a_sample() {
        for rule in APIDOC {
            assert!(SAMPLES.iter().any(|(id, _)| *id == rule.id), "{} lacks a sample doc", rule.id);
        }
    }

    #[test]
    fn rules_are_fully_and_uniquely_identified() {
        crate::rule::assert_well_formed(APIDOC);
    }

    #[test]
    fn the_long_mission_charge_names_the_limit() {
        let charge = APIDOC.iter().find(|rule| rule.id == "apidoc-long-mission").unwrap().what;
        assert!(charge.contains(&format!(" {MAX_MISSION_WORDS} words")), "{charge}");
    }

    #[test]
    fn sanctioned_docs_pass() {
        for source in [
            "/** Returns the foo. */",
            "/**\n * Returns the foo.\n *\n * @param x the x\n * @return the foo\n */",
            "/** {@inheritDoc} */",
            "/** {@inheritDoc}\n * @param x the x */",
            "/** <p>Returns the foo.</p> */",
            "/** Returns the foo, e.g. the bar. */",
            "/** Returns foo.<p>Then a very long paragraph that goes on and on well past any limit a mission could have. */",
            "/**\n * Resolves the value.\n * <ul><li>first</li></ul>\n */",
        ] {
            assert_eq!(judge(source), [] as [&str; 0], "should be sanctioned: {source}");
        }
    }

    #[test]
    fn an_inline_tag_is_one_word_and_markup_is_none() {
        assert_eq!(words("Returns {@link java.util.Map#get(Object) the mapped value} for <b>this</b> key."), 5);
        let mission = "/** Returns {@link Map#get(Object) the mapped value} {@code a b c d e f g h i j k l m n o p q r s t}. */";
        assert_eq!(judge(mission), [] as [&str; 0]);
    }

    #[test]
    fn twenty_words_pass_and_twenty_one_do_not() {
        let twenty = format!("/** {}. */", ["word"; 20].join(" "));
        let twenty_one = format!("/** {}. */", ["word"; 21].join(" "));
        assert_eq!(judge(&twenty), [] as [&str; 0]);
        assert_eq!(judge(&twenty_one), ["apidoc-long-mission"]);
    }

    #[test]
    fn a_lead_introducing_a_list_does_not_stand_alone() {
        assert_eq!(judge("/**\n * Resolves the value in order:\n * <ul><li>a</li></ul>\n */"), ["apidoc-unterminated-mission"]);
    }

    #[test]
    fn every_heresy_is_convicted_once_in_catalogue_order() {
        let source = format!("{LONG}\n/** Returns foo */\n/** @return bar */\n/** Returns baz */\n");
        assert_eq!(
            judge(&source),
            ["apidoc-missing-mission", "apidoc-unterminated-mission", "apidoc-long-mission"]);
    }

    #[test]
    fn citations_name_each_offending_doc() {
        let docs = judged("Foo.java", "", "/** Returns foo */\n/** Fine. */\n/**\n * Returns baz\n */\n");
        let rule = APIDOC.iter().find(|rule| rule.id == "apidoc-unterminated-mission").unwrap();
        assert_eq!(cite(rule, &docs), ["Returns foo", "Returns baz"]);
    }

    mod edits {
        use super::*;

        const BEFORE: &str = "package a;\n\n/** Returns the foo */\nclass Foo {\n  /** @return bar */\n  int bar;\n}\n";

        fn ids(after: &str) -> Vec<&'static str> {
            detect(&judged("Foo.java", BEFORE, after)).iter().map(|rule| rule.id).collect()
        }

        #[test]
        fn an_untouched_heretical_doc_is_not_the_editors_to_answer_for() {
            assert_eq!(ids(&BEFORE.replace("int bar;", "long bar;")), [] as [&str; 0]);
        }

        #[test]
        fn an_untouched_lead_spares_a_doc_whose_tags_were_edited() {
            let after = "package a;\n\n/** Returns the foo\n * @see Bar */\nclass Foo {\n  /** @return bar */\n  int bar;\n}\n";
            assert_eq!(ids(after), [] as [&str; 0]);
        }

        #[test]
        fn a_rewritten_lead_is_judged() {
            assert_eq!(ids(&BEFORE.replace("Returns the foo", "Returns a foo")), ["apidoc-unterminated-mission"]);
        }

        #[test]
        fn a_lead_less_doc_is_judged_once_any_of_it_changes() {
            assert_eq!(ids(&BEFORE.replace("@return bar", "@return the bar")), ["apidoc-missing-mission"]);
        }

        #[test]
        fn a_new_doc_is_judged() {
            assert_eq!(ids(&BEFORE.replace("  int bar;", "  int bar;\n  /** @return baz */\n  int baz;")), ["apidoc-missing-mission"]);
        }

        #[test]
        fn a_fixed_doc_is_sanctioned() {
            assert_eq!(ids(&BEFORE.replace("Returns the foo */", "Returns the foo. */")), [] as [&str; 0]);
        }

        #[test]
        fn a_licence_header_is_not_an_apidoc() {
            assert_eq!(ids(&format!("/** Copyright nobody */\n{BEFORE}")), [] as [&str; 0]);
        }
    }
}
