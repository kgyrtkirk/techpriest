//! The command under judgement, parsed into the stages a shell would run.
//!
//! Parsing is delegated to `brush-parser`, a real bash grammar. Hand-splitting on
//! separator bytes could not tell a `;` inside `git commit -m "fix; ..."` from a
//! true one, so it invented heresies that were never written.

use std::path::{Path, PathBuf};

use brush_parser::ast;
use regex::Regex;

/// One stage of a shell command — a single command within a pipeline.
#[derive(Debug)]
pub struct Stage {
    /// The stage as the parser reconstructs it.
    pub text: String,
    /// The program invoked, empty when the stage is not a simple command.
    pub name: String,
    /// The arguments as written, unexpanded; empty when the stage is not a simple command.
    pub args: Vec<String>,
    /// Stdout feeds the next stage.
    pub pipes: bool,
    /// Stdin comes from the previous stage.
    pub fed: bool,
    redirects_stdout: bool,
    heredoc: bool,
}

impl Stage {
    pub fn matches(&self, pattern: &Regex) -> bool {
        pattern.is_match(&self.text)
    }

    /// True when the invoked program, stripped of its directory, matches.
    ///
    /// Judging the program rather than the whole text is what stops
    /// `git log --grep=python` from reading as an invocation of python.
    pub fn runs(&self, pattern: &Regex) -> bool {
        pattern.is_match(self.program())
    }

    /// The invoked program, without its directory.
    pub fn program(&self) -> &str {
        self.name.rsplit('/').next().unwrap_or(&self.name)
    }

    /// True when the stage reads its own arguments rather than upstream output.
    pub fn is_source(&self) -> bool {
        !self.fed
    }

    /// True when the stage sends stdout anywhere other than the transcript.
    pub fn diverts_output(&self) -> bool {
        self.pipes || self.redirects_stdout
    }

    pub fn is_heredoc(&self) -> bool {
        self.heredoc
    }

    /// The whole command as one unparsed stage, for input the grammar rejects.
    ///
    /// Bash would reject it too, so it will never run; judging it whole is merely
    /// the conservative reading.
    fn opaque(text: &str) -> Self {
        Stage {
            text: text.to_string(),
            name: text.split_whitespace().next().unwrap_or_default().to_string(),
            args: Vec::new(),
            pipes: false,
            fed: false,
            redirects_stdout: false,
            heredoc: false,
        }
    }
}

/// A Bash invocation and the context it runs in.
///
/// There is deliberately no way to match the raw command text: a pattern loosed on
/// the whole string reads arguments as invocations, which is how `git log
/// --grep=python` once counted as running python. Rules judge stages.
#[derive(Debug)]
pub struct Command<'a> {
    pub cwd: &'a str,
    pub in_repo: bool,
    stages: Vec<Stage>,
    loops: bool,
}

impl<'a> Command<'a> {
    pub fn new(text: &str, cwd: &'a str, in_repo: bool) -> Self {
        let (stages, loops) = parse(text).unwrap_or_else(|| (vec![Stage::opaque(text)], false));
        Command { cwd, in_repo, stages, loops }
    }

    #[cfg(test)]
    pub fn stages(&self) -> &[Stage] {
        &self.stages
    }

    pub fn any_stage(&self, judge: impl Fn(&Stage) -> bool) -> bool {
        self.stages.iter().any(judge)
    }

    /// True for any stage whose stdout is piped into a stage the successor test accepts.
    pub fn any_pipe(&self, source: impl Fn(&Stage) -> bool, sink: impl Fn(&Stage) -> bool) -> bool {
        self.stages.windows(2).any(|pair| pair[0].pipes && source(&pair[0]) && sink(&pair[1]))
    }

    /// True when the command drives a shell loop of its own.
    pub fn loops(&self) -> bool {
        self.loops
    }

    /// True when a path the stage names lies in a git working tree.
    ///
    /// Only arguments naming an existing path count, so the pattern and option
    /// values drop out; a stage naming none is judged by the working directory.
    pub fn targets_repo(&self, stage: &Stage) -> bool {
        let targets: Vec<PathBuf> = stage.args.iter().filter_map(|arg| self.existing_path(arg)).collect();
        if targets.is_empty() {
            return self.in_repo;
        }
        targets.iter().any(|path| in_git_tree(path))
    }

    /// The existing path an argument names, read up to its first glob character.
    fn existing_path(&self, arg: &str) -> Option<PathBuf> {
        let literal = arg.trim_matches(['\'', '"']).split(['*', '?', '[']).next()?;
        let path = match literal.strip_prefix("~/") {
            Some(rest) => PathBuf::from(std::env::var_os("HOME")?).join(rest),
            None => Path::new(self.cwd).join(literal),
        };
        path.exists().then_some(path)
    }
}

/// True when the path lies inside a git working tree, at any depth.
pub fn in_git_tree(path: &Path) -> bool {
    path.ancestors().any(|dir| dir.join(".git").exists())
}

/// Parses the command into its pipeline stages, and whether it loops.
///
/// Compound commands are kept whole rather than descended into: their body text
/// still reaches the whole-command rules, and a loop is already its own heresy.
fn parse(text: &str) -> Option<(Vec<Stage>, bool)> {
    let program = brush_parser::Parser::builder()
        .build(std::io::Cursor::new(text))
        .parse_program()
        .ok()?;

    let mut stages = Vec::new();
    let mut loops = false;
    for list in &program.complete_commands {
        for item in &list.0 {
            for (_, pipeline) in item.0.iter() {
                let last = pipeline.seq.len().saturating_sub(1);
                for (i, command) in pipeline.seq.iter().enumerate() {
                    loops |= is_loop(command);
                    stages.push(stage(command, i < last, i > 0));
                }
            }
        }
    }
    Some((stages, loops))
}

fn is_loop(command: &ast::Command) -> bool {
    matches!(
        command,
        ast::Command::Compound(
            ast::CompoundCommand::ForClause(_)
                | ast::CompoundCommand::ArithmeticForClause(_)
                | ast::CompoundCommand::WhileClause(_)
                | ast::CompoundCommand::UntilClause(_),
            _))
}

fn stage(command: &ast::Command, pipes: bool, fed: bool) -> Stage {
    let mut stage = Stage {
        text: command.to_string(),
        name: String::new(),
        args: Vec::new(),
        pipes,
        fed,
        redirects_stdout: false,
        heredoc: false,
    };

    let redirects: Vec<&ast::IoRedirect> = match command {
        ast::Command::Simple(simple) => {
            if let Some(word) = &simple.word_or_name {
                stage.name = word.value.clone();
            }
            stage.args = simple
                .suffix
                .iter()
                .flat_map(|s| s.0.iter())
                .filter_map(|item| match item {
                    ast::CommandPrefixOrSuffixItem::Word(word) => Some(word.value.clone()),
                    _ => None,
                })
                .collect();
            simple
                .prefix
                .iter()
                .flat_map(|p| p.0.iter())
                .chain(simple.suffix.iter().flat_map(|s| s.0.iter()))
                .filter_map(|item| match item {
                    ast::CommandPrefixOrSuffixItem::IoRedirect(redirect) => Some(redirect),
                    _ => None,
                })
                .collect()
        }
        ast::Command::Compound(_, list) | ast::Command::ExtendedTest(_, list) => {
            list.iter().flat_map(|l| l.0.iter()).collect()
        }
        ast::Command::Function(_) => Vec::new(),
    };

    for redirect in redirects {
        stage.redirects_stdout |= redirects_stdout(redirect);
        stage.heredoc |= matches!(redirect, ast::IoRedirect::HereDocument(..));
    }
    stage
}

/// True for a redirect that takes stdout away from the transcript.
///
/// Descriptor duplication like `2>&1` names an explicit descriptor other than
/// stdout and leaves the transcript alone.
fn redirects_stdout(redirect: &ast::IoRedirect) -> bool {
    match redirect {
        ast::IoRedirect::File(fd, kind, _) => {
            matches!(fd, None | Some(1))
                && matches!(
                    kind,
                    ast::IoFileRedirectKind::Write
                        | ast::IoFileRedirectKind::Append
                        | ast::IoFileRedirectKind::Clobber)
        }
        ast::IoRedirect::OutputAndError(..) => true,
        ast::IoRedirect::HereDocument(..) | ast::IoRedirect::HereString(..) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stages(text: &str) -> Vec<(String, bool)> {
        Command::new(text, "", false)
            .stages
            .into_iter()
            .map(|s| (s.text, s.pipes))
            .collect()
    }

    fn texts(text: &str) -> Vec<String> {
        stages(text).into_iter().map(|(t, _)| t).collect()
    }

    #[test]
    fn single_stage_is_never_piped() {
        assert_eq!(stages("cat foo.txt"), [("cat foo.txt".to_string(), false)]);
    }

    #[test]
    fn pipe_marks_only_the_upstream_stage() {
        assert_eq!(
            stages("cat f | grep x"),
            [("cat f".to_string(), true), ("grep x".to_string(), false)]);
    }

    #[test]
    fn logical_operators_are_not_pipes() {
        assert_eq!(texts("a && b || c"), ["a", "b", "c"]);
        assert_eq!(texts("a; b & c"), ["a", "b", "c"]);
        assert_eq!(texts("a\nb"), ["a", "b"]);
    }

    #[test]
    fn descriptor_duplication_is_not_a_stdout_redirect() {
        let cmd = Command::new("mvn -v 2>&1", "", false);
        assert_eq!(cmd.stages().len(), 1);
        assert!(!cmd.stages()[0].diverts_output());
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

    #[test]
    fn a_separator_inside_quotes_does_not_split() {
        assert_eq!(texts(r#"git commit -m "fix; cat the config""#).len(), 1);
        assert_eq!(texts("grep -E 'cat|dog' f.txt").len(), 1);
    }

    #[test]
    fn the_program_is_read_from_the_command_word_not_the_arguments() {
        let cmd = Command::new("git log --grep=python", "", false);
        assert_eq!(cmd.stages()[0].program(), "git");
    }

    #[test]
    fn an_absolute_program_keeps_its_path_but_reports_its_basename() {
        let cmd = Command::new("/usr/bin/mvn compile", "", false);
        assert_eq!(cmd.stages()[0].name, "/usr/bin/mvn");
        assert_eq!(cmd.stages()[0].program(), "mvn");
    }

    #[test]
    fn a_loop_is_recognised_in_both_its_written_forms() {
        assert!(Command::new("for f in a b c; do ls $f; done", "", false).loops());
        assert!(Command::new("for f in a b c\ndo\nls $f\ndone", "", false).loops());
        assert!(!Command::new("ls a b c", "", false).loops());
    }

    #[test]
    fn a_heredoc_is_recognised_from_its_redirect() {
        assert!(Command::new("cat <<'EOF' > f.txt\nbody\nEOF", "", false).stages()[0].is_heredoc());
    }

    #[test]
    fn an_argument_counts_as_a_path_only_when_it_exists() {
        let home = PathBuf::from(std::env::var_os("HOME").unwrap());
        let cmd = Command::new("grep -n Foo ~/", "/nowhere", false);
        assert_eq!(cmd.stages()[0].args, ["-n", "Foo", "~/"]);
        assert_eq!(cmd.existing_path("~/"), Some(home.join("")));
        assert_eq!(cmd.existing_path("Foo"), None);
    }

    #[test]
    fn input_the_grammar_rejects_is_judged_whole() {
        let cmd = Command::new("cat 'unterminated", "", false);
        assert_eq!(cmd.stages().len(), 1);
        assert_eq!(cmd.stages()[0].program(), "cat");
    }
}
