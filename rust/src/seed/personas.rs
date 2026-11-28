//! Roleplay triggers and optional named identity metadata from `personas.lino`.

use super::PERSONAS_LINO;
use super::parser::{parse_lino, split_pipe_list};

/// Canonical name and optional knowledge-graph identifier of a named role.
#[derive(Debug, Clone, Default)]
pub struct Persona {
    pub display_name: String,
    pub wikidata: String,
    pub aliases: Vec<String>,
}

/// Syntax for explicitly naming a role and the facts to frame in that role.
#[derive(Debug, Clone, Default)]
pub struct PersonaSeeds {
    pub triggers: Vec<String>,
    pub personas: Vec<Persona>,
}

impl PersonaSeeds {
    /// The text after the longest recognized leading trigger.
    #[must_use]
    pub fn request<'a>(&self, prompt: &'a str) -> Option<&'a str> {
        let prompt = prompt.trim_start();
        let lower = prompt.to_lowercase();
        self.triggers
            .iter()
            .filter_map(|trigger| {
                let suffix = lower.strip_prefix(trigger)?;
                let boundary = |ch: char| ch.is_whitespace() || matches!(ch, ':' | '：');
                if trigger.ends_with(|ch: char| ch.is_ascii_alphabetic())
                    && suffix.chars().next().is_some_and(|ch| !boundary(ch))
                {
                    return None;
                }
                prompt
                    .get(trigger.len()..)
                    .map(|rest| (trigger.len(), rest))
            })
            .max_by_key(|(length, _)| *length)
            .map(|(_, rest)| rest.trim_start())
    }

    /// Match a complete role name to an optional canonical identity.
    #[must_use]
    pub fn pick_persona(&self, role: &str) -> Option<&Persona> {
        let role = role.to_lowercase();
        self.personas.iter().find(|persona| {
            persona.display_name.to_lowercase() == role || persona.aliases.contains(&role)
        })
    }
}

#[must_use]
pub fn persona_seeds() -> PersonaSeeds {
    let tree = parse_lino(PERSONAS_LINO);
    let Some(root) = tree.children.iter().find(|c| c.name == "personas") else {
        return PersonaSeeds::default();
    };
    PersonaSeeds {
        triggers: split_pipe_list(root.find_child_value("trigger"))
            .into_iter()
            .map(|trigger| trigger.to_lowercase())
            .collect(),
        personas: root
            .children
            .iter()
            .filter(|entry| entry.name == "persona" && !entry.id.is_empty())
            .map(|entry| Persona {
                display_name: entry.id.clone(),
                wikidata: entry.find_child_value("wikidata").to_owned(),
                aliases: split_pipe_list(entry.find_child_value("aliases"))
                    .into_iter()
                    .map(|alias| alias.to_lowercase())
                    .collect(),
            })
            .collect(),
    }
}
