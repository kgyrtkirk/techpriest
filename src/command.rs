//! The command under judgement, split into the stages a shell would run.

use regex::Regex;

/// One stage of a shell command, with the pipes on either side of it.
#[derive(Debug, PartialEq, Eq)]
pub struct Stage<'a> {
    pub text: &'a str,
    /// Stdout feeds the next stage.
    pub pipes: bool,
    /// Stdin comes from the previous stage.
    pub fed: bool,
}

impl Stage<'_> {
    pub fn matches(&self, pattern: &Regex) -> bool {
        pattern.is_match(self.text)
    }

    /// True when the stage reads its own arguments rather than upstream output.
    pub fn is_source(&self) -> bool {
        !self.fed
    }

    /// True when the stage sends stdout anywhere other than the transcript.
    pub fn diverts_output(&self) -> bool {
        self.pipes || redirects_stdout(self.text)
    }

    pub fn is_heredoc(&self) -> bool {
        self.text.contains("<<")
    }
}

/// True for a real stdout redirect, ignoring descriptor duplication like `2>&1`.
fn redirects_stdout(text: &str) -> bool {
    text.match_indices('>').any(|(at, _)| !text[at + 1..].starts_with('&'))
}

/// A Bash invocation and the context it runs in.
#[derive(Debug)]
pub struct Command<'a> {
    pub text: &'a str,
    pub cwd: &'a str,
    pub in_repo: bool,
    stages: Vec<Stage<'a>>,
}

impl<'a> Command<'a> {
    pub fn new(text: &'a str, cwd: &'a str, in_repo: bool) -> Self {
        Command { text, cwd, in_repo, stages: split(text) }
    }

    /// True when the pattern appears anywhere in the whole command.
    pub fn matches(&self, pattern: &Regex) -> bool {
        pattern.is_match(self.text)
    }

    #[cfg(test)]
    pub fn stages(&self) -> &[Stage<'a>] {
        &self.stages
    }

    pub fn any_stage(&self, judge: impl Fn(&Stage<'a>) -> bool) -> bool {
        self.stages.iter().any(judge)
    }

    /// True for any stage whose stdout is piped into a stage the successor test accepts.
    pub fn any_pipe(&self, source: impl Fn(&Stage<'a>) -> bool, sink: impl Fn(&Stage<'a>) -> bool) -> bool {
        self.stages.windows(2).any(|pair| pair[0].pipes && source(&pair[0]) && sink(&pair[1]))
    }
}

/// Splits on `;`, `&`, `&&`, `||`, newlines and `|`, recording which breaks were pipes.
///
/// Deliberately naive about quoting — a guard that under-splits merely judges a
/// larger span of text, which never fabricates a heresy that is not written there.
fn split(text: &str) -> Vec<Stage<'_>> {
    let bytes = text.as_bytes();
    let mut stages = Vec::new();
    let mut start = 0;
    let mut i = 0;
    let mut fed = false;
    while i < bytes.len() {
        let (width, pipes) = match bytes[i] {
            b'|' if bytes.get(i + 1) == Some(&b'|') => (2, false),
            b'|' => (1, true),
            // `2>&1` and friends duplicate a descriptor; the `&` is not a separator.
            b'&' if i > 0 && bytes[i - 1] == b'>' => {
                i += 1;
                continue;
            }
            b'&' if bytes.get(i + 1) == Some(&b'&') => (2, false),
            b'&' | b';' | b'\n' => (1, false),
            _ => {
                i += 1;
                continue;
            }
        };
        stages.push(Stage { text: &text[start..i], pipes, fed });
        fed = pipes;
        i += width;
        start = i;
    }
    stages.push(Stage { text: &text[start..], pipes: false, fed });
    stages
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stages(text: &str) -> Vec<(&str, bool)> {
        split(text).into_iter().map(|s| (s.text, s.pipes)).collect()
    }

    #[test]
    fn single_stage_is_never_piped() {
        assert_eq!(stages("cat foo.txt"), [("cat foo.txt", false)]);
    }

    #[test]
    fn pipe_marks_only_the_upstream_stage() {
        assert_eq!(stages("cat f | grep x"), [("cat f ", true), (" grep x", false)]);
    }

    #[test]
    fn logical_operators_are_not_pipes() {
        assert_eq!(stages("a && b || c"), [("a ", false), (" b ", false), (" c", false)]);
        assert_eq!(stages("a; b & c"), [("a", false), (" b ", false), (" c", false)]);
        assert_eq!(stages("a\nb"), [("a", false), ("b", false)]);
    }

    #[test]
    fn descriptor_duplication_does_not_split() {
        assert_eq!(stages("mvn -v 2>&1"), [("mvn -v 2>&1", false)]);
    }

    #[test]
    fn a_stage_knows_which_side_of_the_pipe_it_is_on() {
        let cmd = Command::new("git diff | grep x | wc -l", "", false);
        let sides: Vec<(bool, bool)> = cmd.stages().iter().map(|s| (s.fed, s.pipes)).collect();
        assert_eq!(sides, [(false, true), (true, true), (true, false)]);
    }

    #[test]
    fn redirect_in_one_stage_leaves_the_others_clean() {
        let cmd = Command::new("head -3 a.txt; ls > out", "", false);
        assert!(!cmd.stages()[0].diverts_output());
        assert!(cmd.stages()[1].diverts_output());
    }
}
