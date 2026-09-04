//! PreToolUse guard purging common Bash heresies.
//!
//! Reads the hook payload on stdin. Silence means the command is sanctioned; a
//! denial on stdout names what broke, why, the directive that was forgotten, and
//! the correct incantation.

mod catalogue;
mod command;
mod ledger;
mod verdict;

use std::io::{self, Read};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use command::Command;
use ledger::Ledger;
use verdict::Denial;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Payload {
    cwd: String,
    session_id: String,
    tool_input: ToolInput,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ToolInput {
    command: String,
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

/// Judges the payload, returning the deny reason when the command is heretical.
fn judge(payload: &Payload) -> Option<String> {
    let text = payload.tool_input.command.trim();
    if text.is_empty() {
        return None;
    }

    let cmd = Command::new(text, &payload.cwd, in_repo(&payload.cwd));
    let convicted = catalogue::detect(&cmd);
    if convicted.is_empty() {
        return None;
    }

    let session = if payload.session_id.is_empty() { "unknown" } else { &payload.session_id };
    let tallies = Ledger::for_session(session).record(convicted.iter().map(|rule| rule.id));
    Some(Denial::new(&convicted, &tallies).to_string())
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

    #[test]
    fn a_sanctioned_command_is_judged_silently() {
        assert_eq!(judge(&payload(r#"{"tool_input":{"command":"git grep -n foo"}}"#)), None);
    }

    #[test]
    fn a_heretical_command_is_denied_with_its_charge() {
        let denial = judge(&payload(r#"{"session_id":"test-judge","tool_input":{"command":"cat f.txt"}}"#))
            .expect("cat must be denied");
        assert!(denial.contains("cat/head/tail/less to view a file"), "{denial}");
    }

    #[test]
    fn a_blank_or_absent_command_is_not_judged() {
        assert_eq!(judge(&payload(r#"{"tool_input":{"command":"   "}}"#)), None);
        assert_eq!(judge(&payload("{}")), None);
        assert_eq!(judge(&payload("not json at all")), None);
    }

    #[test]
    fn unknown_payload_fields_are_tolerated() {
        let parsed = payload(r#"{"cwd":"/tmp","session_id":"s1","tool_name":"Bash","future":{"x":1},"tool_input":{"command":"ls","timeout":5}}"#);
        assert_eq!(parsed.cwd, "/tmp");
        assert_eq!(parsed.session_id, "s1");
        assert_eq!(parsed.tool_input.command, "ls");
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
        assert!(in_repo("/home/dev/claude/druid"));
        assert!(!in_repo("/proc"));
    }
}
