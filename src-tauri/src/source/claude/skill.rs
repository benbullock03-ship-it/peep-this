//! Peep skill calls in hook payloads (AV-54): a verbatim `/peep:<skill> [args]` prompt, or the `Skill` tool.

use serde_json::Value;

/// `(skill, args)` without the `peep:` namespace; other plugins' skills give `None`.
pub fn peep_call(hook: &str, v: &Value) -> Option<(String, String)> {
    let (skill, args) = match hook {
        "UserPromptSubmit" => {
            let rest = v.get("prompt")?.as_str()?.trim_start().strip_prefix("/peep:")?;
            rest.split_once(char::is_whitespace).unwrap_or((rest, ""))
        }
        "PreToolUse" if v.get("tool_name")?.as_str()? == "Skill" => {
            let input = v.get("tool_input")?;
            let skill = input.get("skill")?.as_str()?.strip_prefix("peep:")?;
            (skill, input.get("args").and_then(Value::as_str).unwrap_or(""))
        }
        _ => return None,
    };
    (!skill.is_empty()).then(|| (skill.to_string(), args.trim().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture(name: &str) -> Value {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/legacy/hooks").join(name);
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn peep_slash_prompt_and_skill_tool_parse() {
        let mut v = fixture("user_prompt_submit_slash_command.json");
        assert_eq!(peep_call("UserPromptSubmit", &v), None, "caveman: is not peep:");
        v["prompt"] = json!("/peep:ticket close AV-12\nmore");
        assert_eq!(peep_call("UserPromptSubmit", &v), Some(("ticket".into(), "close AV-12\nmore".into())));
        v["prompt"] = json!("/peep:packup");
        assert_eq!(peep_call("UserPromptSubmit", &v), Some(("packup".into(), "".into())));
        v["prompt"] = json!("please run /peep:ticket AV-1");
        assert_eq!(peep_call("UserPromptSubmit", &v), None);

        let mut v = fixture("pre_tool_use_skill.json");
        assert_eq!(peep_call("PreToolUse", &v), None);
        v["tool_input"]["skill"] = json!("peep:shoot");
        assert_eq!(peep_call("PreToolUse", &v), Some(("shoot".into(), "probe".into())));
        assert_eq!(peep_call("PostToolUse", &fixture("post_tool_use_skill.json")), None);
    }
}
