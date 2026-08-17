# Skill ROI 4-Pillar Scorecard Specification

Detailed mathematical formulas, weights, and scoring criteria for measuring agent skill efficiency.

---

## 1. The 4 Pillars

### Pillar 1: Token Efficiency (Weight: 40%)

Measures reduction in total tokens consumed (input context + generated output):

$$\Delta_{\text{tokens}} = \frac{T_{\text{baseline}} - T_{\text{skill}}}{T_{\text{baseline}}} \times 100\%$$

- **Context Saturation**: Measures peak tokens against the target model's context window limit.
- **Input Savings**: Key driver for tools like `graphify` and `context-gatherer` that prevent multi-file reading.

### Pillar 2: Agent Overhead Reduction (Weight: 30%)

Measures conversation turns and tool calls:

$$\Delta_{\text{tool\_calls}} = \frac{C_{\text{baseline}} - C_{\text{skill}}}{C_{\text{baseline}}} \times 100\%$$

- **Futile Tool Calls**: Penalizes failed file reads, syntax errors, and futile regex searches.

### Pillar 3: Economic Cost Savings (Weight: 20%)

$$\Delta_{\text{cost}} = \frac{\text{Cost}_{\text{baseline}} - \text{Cost}_{\text{skill}}}{\text{Cost}_{\text{baseline}}} \times 100\%$$

Computed using standard model pricing (e.g. Gemini 2.5 Flash / Claude Sonnet / GPT-4o).

### Pillar 4: Latency / Duration Reduction (Weight: 10%)

$$\Delta_{\text{duration}} = \frac{\text{Duration}_{\text{baseline}} - \text{Duration}_{\text{skill}}}{\text{Duration}_{\text{baseline}}} \times 100\%$$

---

## 2. Composite ROI Score & Grading Scale

$$\text{Composite ROI} = 0.40 \cdot \Delta_{\text{tokens}} + 0.30 \cdot \Delta_{\text{tools}} + 0.20 \cdot \Delta_{\text{cost}} + 0.10 \cdot \Delta_{\text{duration}}$$

| Composite Score | ROI Grade | Interpretation |
| :--- | :--- | :--- |
| \(\ge 60.0\) | **S** | Massive efficiency gain (\(>70\%\) tokens/tools saved). |
| \(40.0 - 59.9\) | **A** | Strong efficiency gain (\(40-60\%\) saved). |
| \(20.0 - 39.9\) | **B** | Moderate improvement. |
| \(0.0 - 19.9\) | **C** | Marginal gain. |
| \(-20.0 - -0.1\) | **D** | Inefficient (increases token overhead). |
| \(< -20.0\) | **F** | Negative ROI (severe prompt bloat / hallucination). |
