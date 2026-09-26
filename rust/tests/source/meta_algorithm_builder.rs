use std::fmt::Write as _;
use std::sync::OnceLock;

use crate::event_log::EventLog;
use crate::seed::{CODING_IDIOMS_LINO, parser::parse_lino, render_response, response_for};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodingSurface {
    CodingCatalog,
    ProgramSynthesis,
    ProgramBlueprint,
    NumericList,
    RuleSynthesis,
    InstallationConversion,
}

impl CodingSurface {
    pub(crate) const fn slug(self) -> &'static str {
        match self {
            Self::CodingCatalog => "coding_catalog",
            Self::ProgramSynthesis => "program_synthesis",
            Self::ProgramBlueprint => "program_blueprint",
            Self::NumericList => "numeric_list",
            Self::RuleSynthesis => "rule_synthesis",
            Self::InstallationConversion => "installation_conversion",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AlgorithmConstructionStage {
    id: String,
    output: String,
    verifier: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MetaAlgorithmDefinition {
    id: String,
    stage_evidence: String,
    surface_evidence: String,
    active_marker: String,
    stages: Vec<AlgorithmConstructionStage>,
    projections: Vec<(String, String)>,
}

const CODING_SURFACES: &[CodingSurface] = &[
    CodingSurface::CodingCatalog,
    CodingSurface::ProgramSynthesis,
    CodingSurface::ProgramBlueprint,
    CodingSurface::NumericList,
    CodingSurface::RuleSynthesis,
    CodingSurface::InstallationConversion,
];

fn definition() -> &'static MetaAlgorithmDefinition {
    static DEFINITION: OnceLock<MetaAlgorithmDefinition> = OnceLock::new();
    DEFINITION.get_or_init(|| {
        let tree = parse_lino(CODING_IDIOMS_LINO);
        let catalog = tree
            .children
            .iter()
            .find(|node| node.name == "coding_idioms")
            .expect("coding_idioms seed missing");
        let root = catalog
            .children
            .iter()
            .find(|node| node.name == "coding_meta_algorithm")
            .expect("coding_meta_algorithm seed missing");
        MetaAlgorithmDefinition {
            id: root.id.clone(),
            stage_evidence: root.find_child_value("stage_evidence").to_owned(),
            surface_evidence: root.find_child_value("surface_evidence").to_owned(),
            active_marker: root.find_child_value("active_marker").to_owned(),
            stages: root
                .children
                .iter()
                .filter(|node| node.name == "stage")
                .map(|node| AlgorithmConstructionStage {
                    id: node.id.clone(),
                    output: node.find_child_value("output").to_owned(),
                    verifier: node.find_child_value("verifier").to_owned(),
                })
                .collect(),
            projections: root
                .children
                .iter()
                .filter(|node| node.name == "surface")
                .map(|node| {
                    (
                        node.id.clone(),
                        node.find_child_value("projection").to_owned(),
                    )
                })
                .collect(),
        }
    })
}

fn render_template(template: &str, fields: &[(&str, &str)]) -> String {
    let mut rendered = template.to_owned();
    for (name, value) in fields {
        rendered = rendered.replace(&format!("{{{name}}}"), value);
    }
    rendered
}

fn projection_for<'a>(definition: &'a MetaAlgorithmDefinition, surface: &str) -> &'a str {
    definition
        .projections
        .iter()
        .find(|(slug, _)| slug == surface)
        .map_or("", |(_, projection)| projection)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetaAlgorithmBuilder {
    active_surface: CodingSurface,
}

impl MetaAlgorithmBuilder {
    pub(crate) const fn for_surface(active_surface: CodingSurface) -> Self {
        Self { active_surface }
    }

    pub(crate) fn record(self, log: &mut EventLog) {
        let definition = definition();
        log.append(
            "algorithm_construction:meta_algorithm",
            definition.id.clone(),
        );
        log.append(
            "algorithm_construction:active_surface",
            self.active_surface.slug().to_owned(),
        );
        for stage in &definition.stages {
            log.append(
                "algorithm_construction:stage",
                render_template(
                    &definition.stage_evidence,
                    &[
                        ("stage", &stage.id),
                        ("output", &stage.output),
                        ("verifier", &stage.verifier),
                    ],
                ),
            );
        }
        for surface in CODING_SURFACES {
            let slug = surface.slug();
            let projection = projection_for(definition, slug);
            log.append(
                "algorithm_construction:coding_surface",
                render_template(
                    &definition.surface_evidence,
                    &[("surface", slug), ("projection", projection)],
                ),
            );
        }
    }

    pub(crate) fn write_lino(self, output: &mut String) {
        let definition = definition();
        let _ = writeln!(output, "  meta_algorithm {:?}", definition.id);
        let _ = writeln!(
            output,
            "  active_coding_surface {:?}",
            self.active_surface.slug()
        );
        for stage in &definition.stages {
            let _ = writeln!(output, "  construction_stage {:?}", stage.id);
            let _ = writeln!(output, "  stage_output {:?}", stage.output);
            let _ = writeln!(output, "  stage_verifier {:?}", stage.verifier);
        }
        for surface in CODING_SURFACES {
            let slug = surface.slug();
            let _ = writeln!(output, "  coding_surface {slug:?}");
            let _ = writeln!(
                output,
                "  surface_projection {:?}",
                projection_for(definition, slug)
            );
        }
    }

    pub(crate) fn write_explanation(self, output: &mut String) {
        let definition = definition();
        output.push_str(
            &response_for("coding_meta_algorithm_heading", "en")
                .expect("coding_meta_algorithm_heading seed missing"),
        );
        output.push('\n');
        for (index, stage) in definition.stages.iter().enumerate() {
            let index = (index + 1).to_string();
            output.push_str(
                &render_response(
                    "coding_meta_algorithm_stage",
                    "en",
                    &[
                        ("index", &index),
                        ("stage", &stage.id),
                        ("output", &stage.output),
                        ("verifier", &stage.verifier),
                    ],
                )
                .expect("coding_meta_algorithm_stage seed missing"),
            );
            output.push('\n');
        }
        output.push('\n');
        output.push_str(
            &response_for("coding_meta_algorithm_solutions_heading", "en")
                .expect("coding_meta_algorithm_solutions_heading seed missing"),
        );
        output.push('\n');
        for surface in CODING_SURFACES {
            let active = if *surface == self.active_surface {
                definition.active_marker.as_str()
            } else {
                ""
            };
            let slug = surface.slug();
            output.push_str(
                &render_response(
                    "coding_meta_algorithm_surface",
                    "en",
                    &[
                        ("surface", slug),
                        ("active", active),
                        ("projection", projection_for(definition, slug)),
                    ],
                )
                .expect("coding_meta_algorithm_surface seed missing"),
            );
            output.push('\n');
        }
    }
}
