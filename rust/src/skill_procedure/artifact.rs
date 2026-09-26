//! Durable, source-grounded artifact serialization and integrity validation.

use crate::engine::{KNOWLEDGE_SCHEMA_VERSION, stable_id};
use crate::intent_formalization::{OrderedRequirementSpan, impulse_id_for};
use crate::links_format::push_lino_node;
use crate::seed::{
    self, ROLE_SKILL_PROCEDURE_STEP_OBJECT, ROLE_SKILL_PROCEDURE_STEP_VERB,
    ROLE_TRANSLATION_LANGUAGE, parser::LinoNode,
};

use super::{
    CompiledProcedure, MINIMUM_STEPS, ProcedureArtifactError, ProcedureRequirement, ProcedureStep,
    ProcedureTrigger, canonical_program, meaning_has_role, requirement_id,
};

impl CompiledProcedure {
    /// Persist the complete executable artifact, including the canonical
    /// program plus source formalization and provenance.
    #[must_use]
    pub fn artifact_links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "compiled_procedure_artifact", Some(&self.id));
        push_lino_node(
            &mut out,
            2,
            "schema_version",
            Some(KNOWLEDGE_SCHEMA_VERSION),
        );
        push_lino_node(&mut out, 2, "impulse_id", Some(&self.impulse_id));
        push_lino_node(
            &mut out,
            2,
            "source_description",
            Some(&self.source_description),
        );
        push_lino_node(
            &mut out,
            2,
            "canonical_program",
            Some(&self.canonical_program),
        );
        for requirement in &self.requirements {
            push_lino_node(&mut out, 2, "requirement", Some(&requirement.id));
            push_lino_node(&mut out, 4, "index", Some(&requirement.index.to_string()));
            push_lino_node(&mut out, 4, "source_text", Some(&requirement.source_text));
            push_lino_node(
                &mut out,
                4,
                "span_start",
                Some(&requirement.source_span.0.to_string()),
            );
            push_lino_node(
                &mut out,
                4,
                "span_end",
                Some(&requirement.source_span.1.to_string()),
            );
        }
        push_lino_node(&mut out, 2, "trigger", None);
        push_lino_node(
            &mut out,
            4,
            "requirement_id",
            Some(&self.trigger.requirement_id),
        );
        push_lino_node(&mut out, 4, "source_text", Some(&self.trigger.source_text));
        push_lino_node(
            &mut out,
            4,
            "span_start",
            Some(&self.trigger.source_span.0.to_string()),
        );
        push_lino_node(
            &mut out,
            4,
            "span_end",
            Some(&self.trigger.source_span.1.to_string()),
        );
        for object in &self.trigger.objects {
            push_lino_node(&mut out, 4, "object", Some(object));
        }
        for step in &self.steps {
            push_lino_node(&mut out, 2, "step", Some(&step.id));
            push_lino_node(&mut out, 4, "index", Some(&step.index.to_string()));
            push_lino_node(&mut out, 4, "requirement_id", Some(&step.requirement_id));
            push_lino_node(&mut out, 4, "kind", Some(&step.kind));
            for object in &step.objects {
                push_lino_node(&mut out, 4, "object", Some(object));
            }
            if let Some(language) = &step.target_language {
                push_lino_node(&mut out, 4, "target_language", Some(language));
            }
            push_lino_node(&mut out, 4, "source_text", Some(&step.source_text));
            push_lino_node(
                &mut out,
                4,
                "span_start",
                Some(&step.source_span.0.to_string()),
            );
            push_lino_node(
                &mut out,
                4,
                "span_end",
                Some(&step.source_span.1.to_string()),
            );
        }
        out
    }

    /// Parse and integrity-check a persisted artifact before it is executed.
    pub fn from_artifact_links_notation(text: &str) -> Result<Self, ProcedureArtifactError> {
        let parsed = seed::parser::parse_lino(text);
        let root = parsed
            .children
            .iter()
            .find(|node| node.name == "compiled_procedure_artifact")
            .ok_or_else(|| ProcedureArtifactError::new("missing compiled procedure artifact"))?;
        if root.id.is_empty() {
            return Err(ProcedureArtifactError::new("artifact id is empty"));
        }
        if artifact_child(root, "schema_version")? != KNOWLEDGE_SCHEMA_VERSION {
            return Err(ProcedureArtifactError::new(
                "unsupported compiled procedure artifact schema",
            ));
        }
        let source_description = artifact_child(root, "source_description")?;
        let impulse_id = artifact_child(root, "impulse_id")?;
        let canonical_program_text = artifact_child(root, "canonical_program")?;
        let requirements = root
            .children
            .iter()
            .filter(|node| node.name == "requirement")
            .map(|node| {
                Ok(ProcedureRequirement {
                    id: node.id.clone(),
                    index: artifact_usize(node, "index")?,
                    source_text: artifact_child(node, "source_text")?,
                    source_span: (
                        artifact_usize(node, "span_start")?,
                        artifact_usize(node, "span_end")?,
                    ),
                })
            })
            .collect::<Result<Vec<_>, ProcedureArtifactError>>()?;
        let trigger_node = root
            .children
            .iter()
            .find(|node| node.name == "trigger")
            .ok_or_else(|| ProcedureArtifactError::new("missing trigger"))?;
        let trigger = ProcedureTrigger {
            requirement_id: artifact_child(trigger_node, "requirement_id")?,
            objects: child_values(trigger_node, "object"),
            source_text: artifact_child(trigger_node, "source_text")?,
            source_span: (
                artifact_usize(trigger_node, "span_start")?,
                artifact_usize(trigger_node, "span_end")?,
            ),
        };
        let steps = root
            .children
            .iter()
            .filter(|node| node.name == "step")
            .map(|node| {
                Ok(ProcedureStep {
                    id: node.id.clone(),
                    index: artifact_usize(node, "index")?,
                    requirement_id: artifact_child(node, "requirement_id")?,
                    kind: artifact_child(node, "kind")?,
                    objects: child_values(node, "object"),
                    target_language: optional_artifact_child(node, "target_language"),
                    source_text: artifact_child(node, "source_text")?,
                    source_span: (
                        artifact_usize(node, "span_start")?,
                        artifact_usize(node, "span_end")?,
                    ),
                })
            })
            .collect::<Result<Vec<_>, ProcedureArtifactError>>()?;
        let procedure = Self {
            id: root.id.clone(),
            source_description,
            impulse_id,
            requirements,
            trigger,
            steps,
            canonical_program: canonical_program_text,
        };
        procedure.validate_artifact()?;
        Ok(procedure)
    }

    fn validate_artifact(&self) -> Result<(), ProcedureArtifactError> {
        if self.requirements.is_empty() || self.steps.len() < MINIMUM_STEPS {
            return Err(ProcedureArtifactError::new(
                "artifact does not contain a complete multi-step procedure",
            ));
        }
        if self.impulse_id != impulse_id_for(&self.source_description) {
            return Err(ProcedureArtifactError::new(
                "source description failed impulse identity validation",
            ));
        }
        for (index, requirement) in self.requirements.iter().enumerate() {
            let source_requirement = OrderedRequirementSpan {
                source_text: requirement.source_text.clone(),
                source_span: requirement.source_span,
            };
            if requirement.index != index + 1
                || requirement.id
                    != requirement_id(&self.impulse_id, index + 1, &source_requirement)
                || !span_matches(
                    &self.source_description,
                    requirement.source_span,
                    &requirement.source_text,
                )
            {
                return Err(ProcedureArtifactError::new(format!(
                    "invalid_requirement_provenance:{}",
                    requirement.id
                )));
            }
        }
        let trigger_index = self
            .requirements
            .iter()
            .position(|requirement| requirement.id == self.trigger.requirement_id)
            .ok_or_else(|| ProcedureArtifactError::new("invalid trigger requirement"))?;
        let trigger_requirement = &self.requirements[trigger_index];
        if self.requirements.len() != trigger_index + self.steps.len() + 1
            || self.trigger.source_text != trigger_requirement.source_text
            || self.trigger.source_span != trigger_requirement.source_span
            || !span_matches(
                &self.source_description,
                self.trigger.source_span,
                &self.trigger.source_text,
            )
        {
            return Err(ProcedureArtifactError::new("invalid trigger provenance"));
        }
        for (index, step) in self.steps.iter().enumerate() {
            let requirement = &self.requirements[trigger_index + index + 1];
            if step.index != index + 1
                || step.requirement_id != requirement.id
                || step.source_text != requirement.source_text
                || step.source_span != requirement.source_span
                || !span_matches(
                    &self.source_description,
                    step.source_span,
                    &step.source_text,
                )
            {
                return Err(ProcedureArtifactError::new(format!(
                    "invalid_step_provenance:{}",
                    step.id
                )));
            }
        }
        if self
            .trigger
            .objects
            .iter()
            .chain(self.steps.iter().flat_map(|step| &step.objects))
            .any(|slug| !meaning_has_role(slug, ROLE_SKILL_PROCEDURE_STEP_OBJECT))
            || self
                .steps
                .iter()
                .any(|step| !meaning_has_role(&step.kind, ROLE_SKILL_PROCEDURE_STEP_VERB))
            || self.steps.iter().any(|step| {
                step.target_language
                    .as_ref()
                    .is_some_and(|slug| !meaning_has_role(slug, ROLE_TRANSLATION_LANGUAGE))
            })
        {
            return Err(ProcedureArtifactError::new(
                "artifact contains an untyped operation or argument",
            ));
        }
        let canonical = canonical_program(&self.trigger, &self.steps);
        if canonical != self.canonical_program
            || stable_id("compiled_procedure", &canonical) != self.id
        {
            return Err(ProcedureArtifactError::new(
                "canonical program or content id failed integrity validation",
            ));
        }
        for step in &self.steps {
            let expected = stable_id(
                "compiled_procedure_step",
                &format!(
                    "{}:{}:{}:{}",
                    self.id,
                    step.index,
                    step.kind,
                    step.arguments().join("+")
                ),
            );
            if step.id != expected {
                return Err(ProcedureArtifactError::new(format!(
                    "step_id_integrity_failure:{}",
                    step.id
                )));
            }
        }
        Ok(())
    }
}

/// Find a complete artifact embedded in a solver/Agent response.
pub fn extract_compiled_procedure_artifact(
    text: &str,
) -> Result<CompiledProcedure, ProcedureArtifactError> {
    let marker = "compiled_procedure_artifact ";
    let start = text
        .find(marker)
        .ok_or_else(|| ProcedureArtifactError::new("response contains no procedure artifact"))?;
    let tail = &text[start..];
    let end = tail.find("\n```").unwrap_or(tail.len());
    CompiledProcedure::from_artifact_links_notation(&tail[..end])
}

fn artifact_child(node: &LinoNode, name: &str) -> Result<String, ProcedureArtifactError> {
    let value = node.find_child_value(name);
    if value.is_empty() {
        Err(ProcedureArtifactError::new(format!(
            "missing_field:{name}:{}",
            node.name
        )))
    } else {
        Ok(value.to_owned())
    }
}

fn optional_artifact_child(node: &LinoNode, name: &str) -> Option<String> {
    let value = node.find_child_value(name);
    (!value.is_empty()).then(|| value.to_owned())
}

fn artifact_usize(node: &LinoNode, name: &str) -> Result<usize, ProcedureArtifactError> {
    artifact_child(node, name)?
        .parse()
        .map_err(|_| ProcedureArtifactError::new(format!("invalid_field:{name}:{}", node.name)))
}

fn child_values(node: &LinoNode, name: &str) -> Vec<String> {
    node.children
        .iter()
        .filter(|child| child.name == name)
        .map(|child| child.id.clone())
        .collect()
}

fn span_matches(source: &str, span: (usize, usize), expected: &str) -> bool {
    source.get(span.0..span.1) == Some(expected)
}
