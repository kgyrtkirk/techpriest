//! Doc comments read out of Java source, reduced to what the apidoc rules judge.
//!
//! No Java grammar here: a doc comment is a `/**` opening its own line, through
//! the next `*/`. Banners (`/***`) and the empty `/**/` are not doc comments.

use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

/// One doc comment.
#[derive(Debug, PartialEq, Eq)]
pub struct Doc {
    /// The comment's lines, margins and delimiters stripped.
    pub body: String,
    /// The first paragraph of the main description, joined onto one line.
    pub lead: String,
}

impl Doc {
    fn parse(comment: &str) -> Doc {
        let inner = comment[3..comment.len() - 2].trim_end_matches('*');
        let lines: Vec<&str> = inner.lines().map(|line| line.trim_start().trim_start_matches('*').trim()).collect();

        let lead_lines: Vec<&str> = lines
            .iter()
            .copied()
            .take_while(|line| !line.starts_with('@'))
            .skip_while(|line| line.is_empty())
            .take_while(|line| !line.is_empty())
            .collect();
        let joined = lead_lines.join(" ");
        let lead = BLOCK_HTML.split(&joined).map(str::trim).find(|part| !part.is_empty()).unwrap_or_default();

        Doc { body: lines.join("\n").trim().to_string(), lead: lead.to_string() }
    }

    /// The first sentence of the lead, when a `.` closes it.
    pub fn mission(&self) -> Option<&str> {
        SENTENCE_END.find(&self.lead).map(|end| &self.lead[..end.start() + 1])
    }

    /// True when the doc defers to the documentation of the member it overrides.
    pub fn inherits(&self) -> bool {
        self.lead.starts_with("{@inheritDoc}")
    }

    /// The first line of the body, which locates the doc for a reader.
    pub fn opening(&self) -> &str {
        self.body.lines().next().unwrap_or_default()
    }
}

pattern!(COMMENT = r"(?ms)^[ \t]*(/\*\*[^*/].*?\*/)");
pattern!(PACKAGE = r"(?m)^[ \t]*package\s+[\w.]+\s*;");
pattern!(BLOCK_HTML = r"(?i)</?(p|pre|ul|ol|dl|table|blockquote|div|h[1-6])\b[^>]*>");
pattern!(SENTENCE_END = r"\.(\s|<|$)");

/// The doc comments in a Java source file.
///
/// A doc comment ahead of the `package` declaration documents nothing — it is a
/// licence header — except in `package-info.java`, where it documents the package.
pub fn docs(path: &str, source: &str) -> Vec<Doc> {
    let documents_package = Path::new(path).file_name().is_some_and(|name| name == "package-info.java");
    let header_end = if documents_package { 0 } else { PACKAGE.find(source).map_or(0, |m| m.start()) };

    COMMENT
        .captures_iter(source)
        .filter_map(|found| found.get(1))
        .filter(|comment| comment.start() >= header_end)
        .map(|comment| Doc::parse(comment.as_str()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(source: &str) -> Doc {
        let mut found = docs("Foo.java", source);
        assert_eq!(found.len(), 1, "expected one doc in {source:?}");
        found.remove(0)
    }

    #[test]
    fn the_lead_is_the_first_paragraph_joined() {
        let found = doc("/**\n * Returns the\n * foo.\n *\n * More detail.\n */");
        assert_eq!(found.lead, "Returns the foo.");
    }

    #[test]
    fn a_block_tag_ends_the_description() {
        assert_eq!(doc("/**\n * Returns foo\n * @param x the x\n */").lead, "Returns foo");
        assert_eq!(doc("/** @param x the x */").lead, "");
    }

    #[test]
    fn a_block_html_tag_ends_the_lead_but_a_leading_one_does_not_hide_it() {
        assert_eq!(doc("/** Returns foo.<p>More detail. */").lead, "Returns foo.");
        assert_eq!(doc("/** <p>Returns foo.</p> */").lead, "Returns foo.");
        assert_eq!(doc("/**\n * Resolves in order:\n * <ul><li>a</li></ul>\n */").lead, "Resolves in order:");
    }

    #[test]
    fn margins_and_closing_stars_are_stripped() {
        let found = doc("  /**\n   ** Returns foo.\n   **/");
        assert_eq!(found.body, "Returns foo.");
        assert_eq!(found.opening(), "Returns foo.");
    }

    #[test]
    fn the_mission_ends_at_the_first_closing_dot() {
        assert_eq!(doc("/** Resolves {@code a.b} names. Then more. */").mission(), Some("Resolves {@code a.b} names."));
        assert_eq!(doc("/** Returns foo.<br>bar */").mission(), Some("Returns foo."));
        assert_eq!(doc("/** Returns foo */").mission(), None);
    }

    #[test]
    fn inherited_docs_are_recognised() {
        assert!(doc("/** {@inheritDoc} */").inherits());
        assert!(!doc("/** Returns foo. */").inherits());
    }

    #[test]
    fn only_a_doc_comment_opening_its_own_line_counts() {
        let source = "/* plain */\n/***********/\n/**/\nString s = \"/** not a doc */\";\n/** Real. */\n";
        assert_eq!(docs("Foo.java", source), [Doc { body: "Real.".into(), lead: "Real.".into() }]);
    }

    #[test]
    fn a_header_ahead_of_the_package_documents_nothing() {
        let source = "/** Licensed to nobody */\npackage a.b;\n\n/** Real. */\nclass Foo {}\n";
        assert_eq!(docs("Foo.java", source).len(), 1);
        assert_eq!(docs("src/a/b/package-info.java", source).len(), 2);
    }
}
