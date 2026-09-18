//! PreToolUse guard purging common Bash and apidoc heresies.
//!
//! Reads the hook payload on stdin. Silence means the call is sanctioned; a
//! denial on stdout names what broke, why, the directive that was forgotten, and
//! the correct incantation. `Bash` calls are judged by the shell catalogue;
//! `Edit` and `Write` calls on Java source by the apidoc catalogue.

macro_rules! pattern {
    ($name:ident = $source:literal) => {
        static $name: LazyLock<Regex> =
            LazyLock::new(|| Regex::new($source).expect("guard pattern must compile"));
    };
}

mod apidoc;
mod catalogue;
mod command;
mod javadoc;
mod ledger;
mod rule;
mod verdict;

use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use command::Command;
use ledger::{Ledger, Tallies};
use rule::Rule;
use verdict::Denial;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Payload {
    cwd: String,
    session_id: String,
    tool_name: String,
    tool_input: ToolInput,
}

/// The inputs of every judged tool; each tool fills its own.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ToolInput {
    command: String,
    file_path: String,
    content: String,
    old_string: String,
    new_string: String,
    replace_all: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Decision {
    hook_specific_output: HookOutput,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HookOutput {
    hook_event_name: &'static str,
    permission_decision: &'static str,
    permission_decision_reason: String,
}

impl Decision {
    fn deny(reason: String) -> Self {
        Decision {
            hook_specific_output: HookOutput {
                hook_event_name: "PreToolUse",
                permission_decision: "deny",
                permission_decision_reason: reason,
            },
        }
    }
}

fn main() {
    let mut stdin = String::new();
    if io::stdin().read_to_string(&mut stdin).is_err() {
        return;
    }
    if let Some(denial) = judge(&serde_json::from_str(&stdin).unwrap_or_default()) {
        println!("{}", serde_json::to_string(&Decision::deny(denial)).expect("decision must serialise"));
    }
}

/// Judges the payload, returning the deny reason when the call is heretical.
fn judge(payload: &Payload) -> Option<String> {
    match payload.tool_name.as_str() {
        "Bash" => judge_command(payload),
        "Edit" | "Write" => judge_edit(payload),
        _ => None,
    }
}

fn judge_command(payload: &Payload) -> Option<String> {
    let text = payload.tool_input.command.trim();
    if text.is_empty() {
        return None;
    }

    let cmd = Command::new(text, &payload.cwd, in_repo(&payload.cwd));
    let convicted = catalogue::detect(&cmd);
    if convicted.is_empty() {
        return None;
    }
    Some(Denial::new(&convicted, &record(payload, &convicted)).to_string())
}

/// Judges the doc comments a Java edit writes, against the file as it stands on disk.
fn judge_edit(payload: &Payload) -> Option<String> {
    let input = &payload.tool_input;
    if !input.file_path.ends_with(".java") {
        return None;
    }

    let before = fs::read_to_string(Path::new(&payload.cwd).join(&input.file_path)).unwrap_or_default();
    let after = match payload.tool_name.as_str() {
        "Write" => input.content.clone(),
        _ if input.replace_all => before.replace(&input.old_string, &input.new_string),
        _ => before.replacen(&input.old_string, &input.new_string, 1),
    };

    let docs = apidoc::judged(&input.file_path, &before, &after);
    let convicted = apidoc::detect(&docs);
    if convicted.is_empty() {
        return None;
    }
    Some(Denial::new(&convicted, &record(payload, &convicted)).citing(|rule| apidoc::cite(rule, &docs)).to_string())
}

/// Tallies the convictions against the payload's session.
fn record<J>(payload: &Payload, convicted: &[&Rule<J>]) -> Tallies {
    let session = if payload.session_id.is_empty() { "unknown" } else { &payload.session_id };
    Ledger::for_session(session).record(convicted.iter().map(|rule| rule.id))
}

/// True when the command runs inside a git working tree, where the git-grep directives bind.
fn in_repo(cwd: &str) -> bool {
    let start: PathBuf = if cwd.is_empty() {
        std::env::current_dir().unwrap_or_default()
    } else {
        PathBuf::from(cwd)
    };
    start.ancestors().any(|dir: &Path| dir.join(".git").exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(json: &str) -> Payload {
        serde_json::from_str(json).unwrap_or_default()
    }

    fn edit(dir: &Path, tool_input: serde_json::Value) -> Payload {
        payload(&serde_json::json!({
            "cwd": dir,
            "session_id": "test-judge-edit",
            "tool_name": tool_input["content"].as_str().map_or("Edit", |_| "Write"),
            "tool_input": tool_input,
        }).to_string())
    }

    #[test]
    fn a_sanctioned_command_is_judged_silently() {
        assert_eq!(judge(&payload(r#"{"tool_name":"Bash","tool_input":{"command":"git grep -n foo"}}"#)), None);
    }

    #[test]
    fn a_heretical_command_is_denied_with_its_charge() {
        let denial = judge(&payload(r#"{"session_id":"test-judge","tool_name":"Bash","tool_input":{"command":"cat f.txt"}}"#))
            .expect("cat must be denied");
        assert!(denial.contains("cat/head/tail/less to view a file"), "{denial}");
    }

    #[test]
    fn a_blank_or_absent_command_is_not_judged() {
        assert_eq!(judge(&payload(r#"{"tool_name":"Bash","tool_input":{"command":"   "}}"#)), None);
        assert_eq!(judge(&payload("{}")), None);
        assert_eq!(judge(&payload("not json at all")), None);
    }

    #[test]
    fn only_the_judged_tools_are_judged() {
        assert_eq!(judge(&payload(r#"{"tool_name":"Read","tool_input":{"command":"cat f.txt"}}"#)), None);
        assert_eq!(judge(&payload(r#"{"tool_input":{"command":"cat f.txt"}}"#)), None);
    }

    #[test]
    fn unknown_payload_fields_are_tolerated() {
        let parsed = payload(r#"{"cwd":"/tmp","session_id":"s1","tool_name":"Bash","future":{"x":1},"tool_input":{"command":"ls","timeout":5}}"#);
        assert_eq!(parsed.cwd, "/tmp");
        assert_eq!(parsed.session_id, "s1");
        assert_eq!(parsed.tool_name, "Bash");
        assert_eq!(parsed.tool_input.command, "ls");
    }

    #[test]
    fn a_heretical_apidoc_written_is_denied_and_cited() {
        let dir = tempfile::tempdir().unwrap();
        let written = edit(dir.path(), serde_json::json!({
            "file_path": dir.path().join("Foo.java"),
            "content": "package a;\n\n/**\n * Returns the foo\n */\nclass Foo {}\n",
        }));
        let denial = judge(&written).expect("an unterminated mission must be denied");
        assert!(denial.contains("1. ✗ apidoc mission statement not closed by '.'"), "{denial}");
        assert!(denial.contains("   at:        Returns the foo\n"), "{denial}");
    }

    #[test]
    fn an_edit_is_judged_on_the_file_it_produces() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Foo.java"), "package a;\n\n/** Legacy heresy */\nclass Foo {\n  int x;\n}\n").unwrap();

        let untouched = edit(dir.path(), serde_json::json!({
            "file_path": "Foo.java", "old_string": "int x;", "new_string": "long x;",
        }));
        assert_eq!(judge(&untouched), None);

        let heretical = edit(dir.path(), serde_json::json!({
            "file_path": "Foo.java", "old_string": "  int x;", "new_string": "  /** @return x */\n  int x;",
        }));
        assert!(judge(&heretical).expect("a tag-only doc must be denied").contains("apidoc without a mission statement"));
    }

    #[test]
    fn replace_all_reaches_past_the_first_occurrence() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Foo.java"), "int a; // Q\n/** Returns Q. */\nint b;\n").unwrap();
        let wordy = ["word"; 21].join(" ");

        let first = edit(dir.path(), serde_json::json!({
            "file_path": "Foo.java", "old_string": "Q", "new_string": wordy,
        }));
        assert_eq!(judge(&first), None);

        let every = edit(dir.path(), serde_json::json!({
            "file_path": "Foo.java", "old_string": "Q", "new_string": wordy, "replace_all": true,
        }));
        assert!(judge(&every).expect("a wordy mission must be denied").contains("apidoc mission statement over 20 words"));
    }

    #[test]
    fn only_java_source_is_judged_for_apidoc() {
        let dir = tempfile::tempdir().unwrap();
        let written = edit(dir.path(), serde_json::json!({
            "file_path": dir.path().join("Foo.kt"),
            "content": "/** @return nothing */\n",
        }));
        assert_eq!(judge(&written), None);
    }

    /// Both catalogues tally into one session ledger.
    #[test]
    fn rule_ids_are_unique_across_catalogues() {
        for rule in apidoc::APIDOC {
            assert!(catalogue::CATALOGUE.iter().all(|other| other.id != rule.id), "{} is claimed twice", rule.id);
        }
    }

    #[test]
    fn the_decision_speaks_the_hook_protocol() {
        let json = serde_json::to_value(Decision::deny("because".into())).unwrap();
        assert_eq!(json["hookSpecificOutput"]["hookEventName"], "PreToolUse");
        assert_eq!(json["hookSpecificOutput"]["permissionDecision"], "deny");
        assert_eq!(json["hookSpecificOutput"]["permissionDecisionReason"], "because");
    }

    #[test]
    fn a_git_working_tree_is_recognised_from_any_depth() {
        let root = tempfile::tempdir().unwrap();
        let nested = root.path().join("src/main/java");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::create_dir(root.path().join(".git")).unwrap();

        assert!(in_repo(nested.to_str().unwrap()));
        assert!(!in_repo(tempfile::tempdir().unwrap().path().to_str().unwrap()));
    }
}
