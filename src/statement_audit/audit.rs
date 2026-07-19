use std::collections::{BTreeMap, BTreeSet};

use crate::associative_persistence::AssociativeMemory;
use crate::engine::stable_id;
use crate::links_format::push_lino_node;
use crate::probability::{
    rank_probability_candidates, ProbabilityCandidate, ProbabilityRankingConfig, ProbabilityStore,
};
use crate::relative_meta_logic::{RelativeEvidence, SourceTier, Stance, StatementAssessment};
use crate::world_model::{Context, Statement as WorldStatement};

use super::extract::{extract_corpus, proposed_resolution, ExtractedStatement};
use super::model::{
    AttachedEvidence, AuditConfig, AuditFinding, AuditedStatement, Contradiction, EvidenceCapture,
    EvidenceSelector, FindingKind, RepositoryAudit, RepositoryCorpus,
};

#[must_use]
pub fn audit_corpus(
    corpus: &RepositoryCorpus,
    captures: &[EvidenceCapture],
    config: AuditConfig,
) -> RepositoryAudit {
    let mut statements = extract_corpus(corpus)
        .into_iter()
        .map(|extracted| assess_statement(extracted, corpus, captures))
        .collect::<Vec<_>>();
    statements.sort_by(|left, right| left.location.cmp(&right.location));

    let contradictions = weigh_exclusive_claims(&mut statements, config.temperature);
    let findings = collect_findings(&statements, &contradictions);
    let learning = learn(&statements, &contradictions, &findings);

    RepositoryAudit {
        statements,
        contradictions,
        findings,
        learning,
        skipped_paths: corpus.skipped_paths.clone(),
    }
}

fn assess_statement(
    extracted: ExtractedStatement,
    corpus: &RepositoryCorpus,
    captures: &[EvidenceCapture],
) -> AuditedStatement {
    let id = stable_id(
        "audited_statement",
        &format!(
            "{}:{}:{}",
            extracted.location.path, extracted.location.line, extracted.text
        ),
    );
    let mut applicable = captures
        .iter()
        .filter(|capture| capture_matches(capture, &extracted))
        .cloned()
        .collect::<Vec<_>>();
    if let Some(claim) = extracted
        .claim
        .as_ref()
        .filter(|claim| claim.predicate == "path_exists")
    {
        let exists = corpus.tracked_paths.contains(&claim.subject);
        applicable.push(
            EvidenceCapture::for_claim(
                &claim.subject,
                &claim.predicate,
                Some(claim.value.clone()),
                "repository_index",
                "git:index",
                SourceTier::OriginalFirstParty,
                if exists {
                    Stance::Supports
                } else {
                    Stance::Contradicts
                },
                1.0,
            )
            .with_capture("repository_snapshot", "tracked_path_set"),
        );
    }

    let relative = applicable
        .iter()
        .map(|capture| {
            RelativeEvidence::new(
                &capture.source_label,
                capture.tier,
                capture.stance,
                capture.strength,
            )
        })
        .collect::<Vec<_>>();
    let evidence = applicable
        .into_iter()
        .zip(relative.iter())
        .map(|(capture, relative)| AttachedEvidence {
            capture,
            effective_mass: relative.effective_mass(),
        })
        .collect();
    let assessment = StatementAssessment::assess_assumed_true(&extracted.text, &relative);

    AuditedStatement {
        id,
        text: extracted.text,
        location: extracted.location,
        claim: extracted.claim,
        relative_weight: probability_f32(assessment.posterior.get()),
        evidence,
        assessment,
    }
}

fn capture_matches(capture: &EvidenceCapture, statement: &ExtractedStatement) -> bool {
    match &capture.selector {
        EvidenceSelector::StatementText(text) => text == &statement.text,
        EvidenceSelector::Claim {
            subject,
            predicate,
            value,
        } => statement.claim.as_ref().is_some_and(|claim| {
            &claim.subject == subject
                && &claim.predicate == predicate
                && value.as_ref().is_none_or(|value| value == &claim.value)
        }),
    }
}

fn weigh_exclusive_claims(
    statements: &mut [AuditedStatement],
    temperature: f32,
) -> Vec<Contradiction> {
    let mut groups: BTreeMap<(String, String), Vec<usize>> = BTreeMap::new();
    for (index, statement) in statements.iter().enumerate() {
        if let Some(claim) = statement.claim.as_ref().filter(|claim| claim.exclusive) {
            groups
                .entry((claim.subject.clone(), claim.predicate.clone()))
                .or_default()
                .push(index);
        }
    }

    let mut contradictions = Vec::new();
    for ((subject, predicate), indexes) in groups {
        let values = indexes
            .iter()
            .filter_map(|index| statements[*index].claim.as_ref())
            .map(|claim| claim.value.clone())
            .collect::<BTreeSet<_>>();
        if values.len() < 2 {
            continue;
        }
        let candidates = indexes
            .iter()
            .map(|index| {
                ProbabilityCandidate::new(
                    &statements[*index].id,
                    probability_f32(statements[*index].assessment.posterior.get()),
                )
            })
            .collect::<Vec<_>>();
        let ranking = rank_probability_candidates(
            &candidates,
            &ProbabilityStore::new(),
            ProbabilityRankingConfig {
                temperature: temperature.max(f32::EPSILON),
                ..ProbabilityRankingConfig::default()
            },
        );
        for index in &indexes {
            if let Some(weight) = ranking.probability_for(&statements[*index].id) {
                statements[*index].relative_weight = weight;
            }
        }
        let statement_ids = indexes
            .iter()
            .map(|index| statements[*index].id.clone())
            .collect::<Vec<_>>();
        let id = stable_id(
            "requirement_contradiction",
            &format!("{subject}:{predicate}:{}", statement_ids.join(":")),
        );
        contradictions.push(Contradiction {
            id,
            subject,
            predicate,
            statement_ids,
            proposed_resolution: proposed_resolution(),
        });
    }
    contradictions
}

#[allow(clippy::cast_possible_truncation)]
const fn probability_f32(value: f64) -> f32 {
    value as f32
}

fn collect_findings(
    statements: &[AuditedStatement],
    contradictions: &[Contradiction],
) -> Vec<AuditFinding> {
    let mut findings = statements
        .iter()
        .filter(|statement| statement.assessment.posterior.get() < 0.5)
        .map(|statement| AuditFinding {
            id: stable_id("audit_finding", &format!("improbable:{}", statement.id)),
            kind: FindingKind::ImprobableClaim,
            statement_ids: vec![statement.id.clone()],
        })
        .collect::<Vec<_>>();
    findings.extend(contradictions.iter().map(|contradiction| AuditFinding {
        id: stable_id(
            "audit_finding",
            &format!("contradiction:{}", contradiction.id),
        ),
        kind: FindingKind::RequirementContradiction,
        statement_ids: contradiction.statement_ids.clone(),
    }));
    findings
}

fn learn(
    statements: &[AuditedStatement],
    contradictions: &[Contradiction],
    findings: &[AuditFinding],
) -> AssociativeMemory {
    let mut context = Context::new("repository_statement_audit");
    let worlds = statements.iter().map(|statement| {
        let mut world = WorldStatement::new(&statement.text);
        world.id.clone_from(&statement.id);
        world.evidence = statement
            .evidence
            .iter()
            .map(|evidence| {
                RelativeEvidence::new(
                    &evidence.capture.source_label,
                    evidence.capture.tier,
                    evidence.capture.stance,
                    evidence.capture.strength,
                )
            })
            .collect();
        world
    });
    let _ = context.extend_statements(worlds);
    let mut memory = AssociativeMemory::from_context(&context);
    for statement in statements {
        for evidence in &statement.evidence {
            let evidence_id = evidence_id(&statement.id, &evidence.capture);
            memory.persist_identified(
                evidence_id.clone(),
                format!(
                    "{}:{}:{}",
                    evidence.capture.source_url,
                    evidence.capture.captured_at,
                    evidence.capture.sha256
                ),
            );
            memory.associate(&statement.id, &evidence_id);
        }
    }
    for contradiction in contradictions {
        memory.persist_identified(
            contradiction.id.clone(),
            contradiction.proposed_resolution.clone(),
        );
        for statement_id in &contradiction.statement_ids {
            memory.associate(&contradiction.id, statement_id);
        }
    }
    for finding in findings {
        memory.persist_identified(finding.id.clone(), finding.kind.slug());
        for statement_id in &finding.statement_ids {
            memory.associate(&finding.id, statement_id);
        }
    }
    memory
}

fn evidence_id(statement_id: &str, capture: &EvidenceCapture) -> String {
    stable_id(
        "evidence_provenance",
        &format!(
            "{statement_id}:{}:{}:{}",
            capture.source_url, capture.captured_at, capture.sha256
        ),
    )
}

impl RepositoryAudit {
    /// Serialize the audit and learned network as deterministic append-only links.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut output = String::new();
        push_lino_node(&mut output, 0, "repository_statement_audit", None);
        push_lino_node(&mut output, 2, "summary", None);
        push_lino_node(
            &mut output,
            4,
            "statement_count",
            Some(&self.statements.len().to_string()),
        );
        push_lino_node(
            &mut output,
            4,
            "contradiction_count",
            Some(&self.contradictions.len().to_string()),
        );
        push_lino_node(
            &mut output,
            4,
            "finding_count",
            Some(&self.findings.len().to_string()),
        );
        push_lino_node(
            &mut output,
            4,
            "skipped_path_count",
            Some(&self.skipped_paths.len().to_string()),
        );
        push_lino_node(&mut output, 2, "statements", None);
        for statement in &self.statements {
            push_lino_node(&mut output, 4, &statement.id, None);
            push_lino_node(&mut output, 6, "type", Some("audited_statement"));
            push_lino_node(&mut output, 6, "text", Some(&statement.text));
            push_lino_node(
                &mut output,
                6,
                "posterior",
                Some(&statement.assessment.posterior.get().to_string()),
            );
            push_lino_node(
                &mut output,
                6,
                "relative_weight",
                Some(&statement.relative_weight.to_string()),
            );
            push_lino_node(&mut output, 6, "source_location", None);
            push_lino_node(&mut output, 8, "path", Some(&statement.location.path));
            push_lino_node(
                &mut output,
                8,
                "line",
                Some(&statement.location.line.to_string()),
            );
            push_lino_node(&mut output, 8, "kind", Some(statement.location.kind.slug()));
            if let Some(claim) = &statement.claim {
                push_lino_node(&mut output, 6, "claim", None);
                push_lino_node(&mut output, 8, "subject", Some(&claim.subject));
                push_lino_node(&mut output, 8, "predicate", Some(&claim.predicate));
                push_lino_node(&mut output, 8, "value", Some(&claim.value));
                push_lino_node(
                    &mut output,
                    8,
                    "exclusive",
                    Some(&claim.exclusive.to_string()),
                );
            }
            push_lino_node(&mut output, 6, "learning", None);
            push_lino_node(
                &mut output,
                8,
                "reads",
                Some(&self.learning.reads(&statement.id).to_string()),
            );
            push_lino_node(
                &mut output,
                8,
                "writes",
                Some(&self.learning.writes(&statement.id).to_string()),
            );
            push_lino_node(
                &mut output,
                8,
                "retention_score",
                Some(&self.learning.retention_score(&statement.id).to_string()),
            );
            if !statement.evidence.is_empty() {
                push_lino_node(&mut output, 6, "evidence", None);
            }
            for evidence in &statement.evidence {
                let id = evidence_id(&statement.id, &evidence.capture);
                push_lino_node(&mut output, 8, &id, None);
                push_lino_node(
                    &mut output,
                    10,
                    "source_label",
                    Some(&evidence.capture.source_label),
                );
                push_lino_node(
                    &mut output,
                    10,
                    "source_url",
                    Some(&evidence.capture.source_url),
                );
                push_lino_node(
                    &mut output,
                    10,
                    "source_tier",
                    Some(evidence.capture.tier.slug()),
                );
                push_lino_node(
                    &mut output,
                    10,
                    "stance",
                    Some(evidence.capture.stance.slug()),
                );
                push_lino_node(
                    &mut output,
                    10,
                    "asserted_strength",
                    Some(&evidence.capture.strength.to_string()),
                );
                push_lino_node(
                    &mut output,
                    10,
                    "effective_mass",
                    Some(&evidence.effective_mass.to_string()),
                );
                push_lino_node(
                    &mut output,
                    10,
                    "captured_at",
                    Some(&evidence.capture.captured_at),
                );
                push_lino_node(&mut output, 10, "sha256", Some(&evidence.capture.sha256));
            }
        }
        push_lino_node(&mut output, 2, "contradictions", None);
        for contradiction in &self.contradictions {
            push_lino_node(&mut output, 4, &contradiction.id, None);
            push_lino_node(&mut output, 6, "type", Some("requirement_contradiction"));
            push_lino_node(&mut output, 6, "subject", Some(&contradiction.subject));
            push_lino_node(&mut output, 6, "predicate", Some(&contradiction.predicate));
            push_lino_node(
                &mut output,
                6,
                "statement_ids",
                Some(&contradiction.statement_ids.join(" ")),
            );
            push_lino_node(
                &mut output,
                6,
                "proposed_resolution",
                Some(&contradiction.proposed_resolution),
            );
        }
        push_lino_node(&mut output, 2, "findings", None);
        for finding in &self.findings {
            push_lino_node(&mut output, 4, &finding.id, None);
            push_lino_node(&mut output, 6, "type", Some("audit_finding"));
            push_lino_node(&mut output, 6, "kind", Some(finding.kind.slug()));
            push_lino_node(
                &mut output,
                6,
                "statement_ids",
                Some(&finding.statement_ids.join(" ")),
            );
        }
        push_lino_node(&mut output, 2, "associations", None);
        for (from, to) in self.association_links() {
            let id = stable_id("audit_association", &format!("{from}:{to}"));
            push_lino_node(&mut output, 4, &id, None);
            push_lino_node(&mut output, 6, "from", Some(&from));
            push_lino_node(&mut output, 6, "to", Some(&to));
        }
        push_lino_node(&mut output, 2, "skipped_paths", None);
        for path in &self.skipped_paths {
            let id = stable_id("skipped_path", path);
            push_lino_node(&mut output, 4, &id, Some(path));
        }
        output
    }

    fn association_links(&self) -> BTreeSet<(String, String)> {
        let mut links = BTreeSet::new();
        for statement in &self.statements {
            for evidence in &statement.evidence {
                links.insert((
                    statement.id.clone(),
                    evidence_id(&statement.id, &evidence.capture),
                ));
            }
        }
        for contradiction in &self.contradictions {
            for statement_id in &contradiction.statement_ids {
                links.insert((contradiction.id.clone(), statement_id.clone()));
            }
        }
        for finding in &self.findings {
            for statement_id in &finding.statement_ids {
                links.insert((finding.id.clone(), statement_id.clone()));
            }
        }
        links
    }
}
