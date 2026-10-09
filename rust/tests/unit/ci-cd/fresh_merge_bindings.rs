//! Observe the exact environment on steps that invoke the fresh-merge script.
pub(super) struct FreshMergeBinding {
    pub(super) invocations: usize,
    pub(super) pins: Vec<String>,
}
impl FreshMergeBinding {
    pub(super) fn shared_resolver(&self) -> Option<&'static str> {
        if self.pins.len() != 1 {
            return None;
        }
        let normalized: String = self.pins[0]
            .split('#')
            .next()?
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect();
        match normalized.as_str() {
            "${{needs.base.outputs.commit}}" => Some("needs.base.outputs.commit"),
            "${{inputs.base-commit}}" => Some("inputs.base-commit"),
            _ => None,
        }
    }
}
fn merge_invocation(line: &str) -> bool {
    let command = line
        .trim()
        .strip_prefix("run:")
        .unwrap_or(line.trim())
        .trim();
    let words: Vec<_> = command.split_whitespace().collect();
    matches!(
        words.as_slice(),
        ["bash", "scripts/simulate-fresh-merge.sh", ..] | ["scripts/simulate-fresh-merge.sh", ..]
    )
}
fn binding(step: &[&str]) -> Option<FreshMergeBinding> {
    let mut in_run = false;
    let mut in_environment = false;
    let mut invocations = 0;
    let mut pins = Vec::new();
    for line in step {
        if line.starts_with("        run:") {
            in_run = true;
            in_environment = false;
        } else if line.starts_with("        env:") {
            in_environment = true;
            in_run = false;
        } else if !line.is_empty() && !line.starts_with("          ") {
            in_environment = false;
            in_run = false;
        }
        if in_run && merge_invocation(line) {
            invocations += 1;
        }
        if in_environment && let Some(pin) = line.strip_prefix("          BASE_COMMIT:") {
            pins.push(pin.trim().to_owned());
        }
    }
    (invocations > 0).then_some(FreshMergeBinding { invocations, pins })
}
pub(super) fn merge_step_bindings(workflow: &str) -> Vec<FreshMergeBinding> {
    let mut bindings = Vec::new();
    let mut step = Vec::new();
    for line in workflow.lines() {
        if line.starts_with("      - ") {
            if let Some(observed) = binding(&step) {
                bindings.push(observed);
            }
            step.clear();
            step.push(line);
        } else if !step.is_empty() && (line.is_empty() || line.starts_with("        ")) {
            step.push(line);
        } else {
            if let Some(observed) = binding(&step) {
                bindings.push(observed);
            }
            step.clear();
        }
    }
    if let Some(observed) = binding(&step) {
        bindings.push(observed);
    }
    bindings
}
#[test]
fn every_merge_pin_is_bound_to_its_actual_invoking_step() {
    let cases = [
        "      - name: merge\n        env:\n          NATIVE_BASE_COMMIT: ${{ needs.base.outputs.commit }}\n        run: bash scripts/simulate-fresh-merge.sh",
        "      - name: metadata\n        env:\n          BASE_COMMIT: ${{ needs.base.outputs.commit }}\n        run: echo metadata\n      - name: merge\n        run: bash scripts/simulate-fresh-merge.sh",
        "      - name: merge\n        env:\n          BASE_COMMIT: ${{ github.sha }}\n        run: bash scripts/simulate-fresh-merge.sh",
        "      - name: merge\n        env:\n          # BASE_COMMIT: ${{ needs.base.outputs.commit }}\n        run: bash scripts/simulate-fresh-merge.sh",
        "      - name: merge\n        run: bash scripts/simulate-fresh-merge.sh",
        "      - name: merge\n        env:\n          BASE_COMMIT: ${{ needs.base.outputs.commit }}\n          BASE_COMMIT: ${{ inputs.base-commit }}\n        run: bash scripts/simulate-fresh-merge.sh",
    ];
    for workflow in cases {
        let observed = merge_step_bindings(workflow);
        assert_eq!(observed.len(), 1, "{workflow}");
        assert_eq!(observed[0].invocations, 1, "{workflow}");
        assert_eq!(observed[0].shared_resolver(), None, "{workflow}");
    }
    let valid = "      - name: merge\n        env:\n          BASE_COMMIT: ${{ needs.base.outputs.commit }}\n        run: |\n          bash scripts/simulate-fresh-merge.sh\n          bash scripts/simulate-fresh-merge.sh";
    let observed = merge_step_bindings(valid);
    assert_eq!(observed[0].invocations, 2);
    assert_eq!(
        observed[0].shared_resolver(),
        Some("needs.base.outputs.commit")
    );
    assert!(
        merge_step_bindings(
            "      - name: metadata\n        run: echo scripts/simulate-fresh-merge.sh"
        )
        .is_empty()
    );
}
