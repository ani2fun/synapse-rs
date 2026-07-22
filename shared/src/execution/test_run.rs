//! The authored test suite + pure judging — shared because the workbench (client) and the
//! submission judge (server) apply the SAME rules.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::execution::RunResult;

/// One declared stdin argument. The authored JSON writes `type`; the field is `tpe` here
/// (mapped at the codec) since `type` is a reserved word.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ArgSpec {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub tpe: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
}

/// One authored case: values per declared arg + the optional expected stdout. `sample` marks
/// the browser-visible cases — the judge runs every case, but only samples cross the wire so a
/// student cannot hard-code the hidden suite. Absent in `.tests.json` ⇒ `false` ⇒ hidden.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TestCase {
    pub args: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(default, skip_serializing_if = "is_hidden")]
    pub sample: bool,
}

/// `skip_serializing_if` for `sample`: a hidden (non-sample) case omits the key entirely, so the
/// wire payload the browser sees carries no redundant markers. The `&bool` signature is serde's
/// requirement, not a choice.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_hidden(sample: &bool) -> bool {
    !*sample
}

/// The whole authored suite (a testcases fence or a `.tests.json` sidecar).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TestSpec {
    pub args: Vec<ArgSpec>,
    pub cases: Vec<TestCase>,
}

impl TestSpec {
    /// The browser-visible projection: the declared args plus only the `sample` cases, each with
    /// its flag cleared (the samples ARE the payload, so the marker adds nothing). This is the
    /// ONLY `TestSpec` the catalog serves to a page — the hidden judge cases never leave the server.
    #[must_use]
    pub fn samples(&self) -> TestSpec {
        TestSpec {
            args: self.args.clone(),
            cases: self
                .cases
                .iter()
                .filter(|case| case.sample)
                .map(|case| TestCase {
                    args: case.args.clone(),
                    expected: case.expected.clone(),
                    sample: false,
                })
                .collect(),
        }
    }
}

/// A judged case's verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Accepted,
    WrongAnswer,
    Errored,
    /// Ran clean with no expected output declared — counts as a pass.
    Finished,
}

/// The stdin a case feeds the program: ONE LINE PER DECLARED ARG, in declaration order
/// (missing values become empty lines), with a trailing newline.
pub fn stdin_for(args: &[ArgSpec], values: &BTreeMap<String, String>) -> String {
    let mut lines: Vec<&str> = args
        .iter()
        .map(|arg| values.get(&arg.id).map_or("", String::as_str))
        .collect();
    lines.push(""); // the trailing newline
    lines.join("\n")
}

/// Judge one run: a non-clean run is `Errored`; a clean run with no expected output is
/// `Finished`; otherwise TRIMMED stdout comparison.
pub fn judge(result: &RunResult, expected: Option<&str>) -> Verdict {
    if !result.status.is_success() {
        return Verdict::Errored;
    }
    match expected {
        None => Verdict::Finished,
        Some(expected) if result.stdout.trim() == expected.trim() => Verdict::Accepted,
        Some(_) => Verdict::WrongAnswer,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::execution::RunStatus;

    fn spec_args() -> Vec<ArgSpec> {
        ["a", "b"]
            .iter()
            .map(|id| ArgSpec {
                id: (*id).to_owned(),
                label: id.to_uppercase(),
                tpe: "int".to_owned(),
                placeholder: None,
            })
            .collect()
    }

    fn run(status: RunStatus, stdout: &str) -> RunResult {
        RunResult {
            status,
            stdout: stdout.to_owned(),
            stderr: String::new(),
            compile_output: String::new(),
            time_seconds: None,
            memory_kb: None,
        }
    }

    #[test]
    fn stdin_is_one_line_per_declared_arg_in_order_with_trailing_newline() {
        let values = BTreeMap::from([("b".to_owned(), "2".to_owned()), ("a".to_owned(), "1".to_owned())]);
        assert_eq!(stdin_for(&spec_args(), &values), "1\n2\n");
        // A missing value is an EMPTY line, keeping positions aligned.
        let sparse = BTreeMap::from([("b".to_owned(), "2".to_owned())]);
        assert_eq!(stdin_for(&spec_args(), &sparse), "\n2\n");
    }

    #[test]
    fn judging_rules() {
        assert_eq!(
            judge(&run(RunStatus::Accepted, "42\n"), Some("42")),
            Verdict::Accepted
        );
        assert_eq!(
            judge(&run(RunStatus::Accepted, "41"), Some("42")),
            Verdict::WrongAnswer
        );
        assert_eq!(
            judge(&run(RunStatus::RuntimeError, ""), Some("42")),
            Verdict::Errored
        );
        assert_eq!(
            judge(&run(RunStatus::Accepted, "anything"), None),
            Verdict::Finished
        );
    }

    #[test]
    fn the_authored_json_writes_type_not_tpe() {
        let spec: TestSpec = serde_json::from_str(
            r#"{"args":[{"id":"n","label":"N","type":"int"}],"cases":[{"args":{"n":"3"},"expected":"6"}]}"#,
        )
        .unwrap();
        assert_eq!(spec.args[0].tpe, "int");
        let written = serde_json::to_string(&spec).unwrap();
        assert!(written.contains("\"type\":\"int\""));
        assert!(!written.contains("tpe"));
    }

    #[test]
    fn sample_defaults_to_hidden_and_omits_the_key_when_false() {
        // A `.tests.json` case with no `sample` field decodes as hidden (judge-only)…
        let spec: TestSpec = serde_json::from_str(
            r#"{"args":[{"id":"n","label":"N","type":"int"}],"cases":[{"args":{"n":"3"},"expected":"6"}]}"#,
        )
        .unwrap();
        assert!(!spec.cases[0].sample);
        // …and re-serializes with no `sample` key, so hidden cases stay unmarked on the wire.
        assert!(!serde_json::to_string(&spec).unwrap().contains("sample"));
    }

    #[test]
    fn samples_keeps_only_sampled_cases_and_clears_the_flag() {
        let spec: TestSpec = serde_json::from_str(
            r#"{"args":[{"id":"n","label":"N","type":"int"}],"cases":[
                {"args":{"n":"6"},"expected":"[1, 2, 3, 6]","sample":true},
                {"args":{"n":"999"},"expected":"[1, 3, 9, 27, 37, 111, 333, 999]"}
            ]}"#,
        )
        .unwrap();
        let samples = spec.samples();
        assert_eq!(samples.cases.len(), 1, "only the sampled case survives");
        assert_eq!(samples.cases[0].args["n"], "6");
        assert!(!samples.cases[0].sample, "the served marker is cleared");
        assert_eq!(samples.args, spec.args, "declared args are preserved");
        // The hidden case's expected output never appears in the served projection.
        assert!(!serde_json::to_string(&samples).unwrap().contains("999"));
    }
}
