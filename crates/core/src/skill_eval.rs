use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScenarioAssertion {
    #[serde(rename = "type")]
    pub assertion_type: String,
    pub pattern: Option<String>,
    pub threshold: Option<f64>,
    pub command: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScenarioPenalty {
    #[serde(rename = "type")]
    pub penalty_type: String,
    pub max_allowed: Option<usize>,
    pub penalty_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct EvalCriteria {
    #[serde(default)]
    pub assertions: Vec<ScenarioAssertion>,
    #[serde(default)]
    pub penalties: Vec<ScenarioPenalty>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillScenario {
    #[serde(default = "default_version")]
    pub version: String,
    pub skill: String,
    pub scenario_id: String,
    pub description: String,
    #[serde(default = "default_complexity")]
    pub complexity: String,
    pub fixture_dir: Option<String>,
    pub max_turns: Option<usize>,
    pub timeout_seconds: Option<u64>,
    pub prompt: String,
    #[serde(default)]
    pub eval_criteria: EvalCriteria,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

fn default_complexity() -> String {
    "medium".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TelemetrySnapshot {
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub total_tokens: usize,
    pub peak_context_tokens: usize,
    pub turns: usize,
    pub tool_calls: usize,
    pub failed_tool_calls: usize,
    pub duration_ms: u64,
    pub estimated_cost_usd: f64,
    pub success: bool,
    pub assertion_score: f64,
}

impl TelemetrySnapshot {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        input_tokens: usize,
        output_tokens: usize,
        peak_context_tokens: usize,
        turns: usize,
        tool_calls: usize,
        failed_tool_calls: usize,
        duration_ms: u64,
        cost_usd: f64,
        success: bool,
        assertion_score: f64,
    ) -> Self {
        Self {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
            peak_context_tokens,
            turns,
            tool_calls,
            failed_tool_calls,
            duration_ms,
            estimated_cost_usd: cost_usd,
            success,
            assertion_score,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TwinSessionResult {
    pub scenario_id: String,
    pub skill_name: String,
    pub baseline: TelemetrySnapshot,
    pub with_skill: TelemetrySnapshot,
    pub token_savings_pct: f64,
    pub tool_call_reduction_pct: f64,
    pub duration_reduction_pct: f64,
    pub cost_savings_pct: f64,
    pub composite_roi_score: f64,
    pub roi_grade: String,
}

/// Computes percentage delta: ((baseline - test) / baseline) * 100.0
/// Positive values indicate savings/reduction (desirable).
pub fn compute_savings_pct(baseline: f64, test: f64) -> f64 {
    if baseline.abs() < f64::EPSILON {
        0.0
    } else {
        ((baseline - test) / baseline) * 100.0
    }
}

pub fn calculate_roi_grade(composite_score: f64) -> String {
    if composite_score >= 60.0 {
        "S".to_string()
    } else if composite_score >= 40.0 {
        "A".to_string()
    } else if composite_score >= 20.0 {
        "B".to_string()
    } else if composite_score >= 0.0 {
        "C".to_string()
    } else if composite_score >= -20.0 {
        "D".to_string()
    } else {
        "F".to_string()
    }
}

pub fn compute_twin_result(
    scenario_id: &str,
    skill_name: &str,
    baseline: TelemetrySnapshot,
    with_skill: TelemetrySnapshot,
) -> TwinSessionResult {
    let token_savings_pct =
        compute_savings_pct(baseline.total_tokens as f64, with_skill.total_tokens as f64);
    let tool_call_reduction_pct =
        compute_savings_pct(baseline.tool_calls as f64, with_skill.tool_calls as f64);
    let duration_reduction_pct =
        compute_savings_pct(baseline.duration_ms as f64, with_skill.duration_ms as f64);
    let cost_savings_pct =
        compute_savings_pct(baseline.estimated_cost_usd, with_skill.estimated_cost_usd);

    // Composite ROI formula: 40% Token Savings + 30% Tool Call Reduction + 20% Cost Savings + 10% Latency
    let composite_roi_score = (token_savings_pct * 0.40)
        + (tool_call_reduction_pct * 0.30)
        + (cost_savings_pct * 0.20)
        + (duration_reduction_pct * 0.10);

    let roi_grade = calculate_roi_grade(composite_roi_score);

    TwinSessionResult {
        scenario_id: scenario_id.to_string(),
        skill_name: skill_name.to_string(),
        baseline,
        with_skill,
        token_savings_pct,
        tool_call_reduction_pct,
        duration_reduction_pct,
        cost_savings_pct,
        composite_roi_score,
        roi_grade,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_savings_pct() {
        assert!((compute_savings_pct(100.0, 20.0) - 80.0).abs() < f64::EPSILON);
        assert!((compute_savings_pct(100.0, 100.0) - 0.0).abs() < f64::EPSILON);
        assert!((compute_savings_pct(100.0, 120.0) - -20.0).abs() < f64::EPSILON);
        assert!((compute_savings_pct(0.0, 50.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_roi_grade() {
        assert_eq!(calculate_roi_grade(75.0), "S");
        assert_eq!(calculate_roi_grade(45.0), "A");
        assert_eq!(calculate_roi_grade(25.0), "B");
        assert_eq!(calculate_roi_grade(5.0), "C");
        assert_eq!(calculate_roi_grade(-10.0), "D");
        assert_eq!(calculate_roi_grade(-30.0), "F");
    }

    #[test]
    fn test_compute_twin_result() {
        let baseline =
            TelemetrySnapshot::new(40000, 2000, 42000, 10, 20, 4, 30000, 0.010, true, 1.0);
        let with_skill = TelemetrySnapshot::new(8000, 1000, 9000, 3, 4, 0, 8000, 0.002, true, 1.0);

        let res = compute_twin_result("test_scenario", "graphify", baseline, with_skill);
        assert_eq!(res.scenario_id, "test_scenario");
        assert_eq!(res.skill_name, "graphify");
        assert!(res.token_savings_pct > 70.0);
        assert!(res.tool_call_reduction_pct > 75.0);
        assert_eq!(res.roi_grade, "S");
    }

    #[test]
    fn test_parse_scenario_yaml() {
        let yaml = r#"
version: "1.0.0"
skill: "graphify"
scenario_id: "find_circular_dependency"
description: "Locate circular imports in multi-file workspace"
complexity: "medium"
prompt: "Find circular imports"
eval_criteria:
  assertions:
    - type: "max_tool_calls"
      threshold: 5.0
"#;
        let scenario: SkillScenario = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(scenario.skill, "graphify");
        assert_eq!(scenario.scenario_id, "find_circular_dependency");
        assert_eq!(scenario.eval_criteria.assertions.len(), 1);
        assert_eq!(
            scenario.eval_criteria.assertions[0].assertion_type,
            "max_tool_calls"
        );
    }
}
