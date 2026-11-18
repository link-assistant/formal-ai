//! Meaning graphs: a frame, ordered slot values, polarity and mood.
//! JavaScript root: js/agentic/crate/cnl_grammar.mjs.
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Slot values in insertion order, like a JavaScript object with word keys.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Slots(pub Vec<(String, String)>);

impl Slots {
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(slot, _)| slot == name)
            .map(|(_, value)| value.as_str())
    }

    pub fn insert(&mut self, name: &str, value: &str) {
        match self.0.iter_mut().find(|(slot, _)| slot == name) {
            Some(entry) => value.clone_into(&mut entry.1),
            None => self.0.push((name.to_owned(), value.to_owned())),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0
            .iter()
            .map(|(slot, value)| (slot.as_str(), value.as_str()))
    }
}

impl<const N: usize> From<[(&str, &str); N]> for Slots {
    fn from(items: [(&str, &str); N]) -> Self {
        let mut slots = Self::default();
        for (name, value) in items {
            slots.insert(name, value);
        }
        slots
    }
}

impl Serialize for Slots {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (slot, value) in &self.0 {
            map.serialize_entry(slot, value)?;
        }
        map.end()
    }
}

struct SlotsVisitor;

impl<'de> Visitor<'de> for SlotsVisitor {
    type Value = Slots;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map of slot names to string values")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Slots, A::Error> {
        let mut slots = Slots::default();
        while let Some((name, value)) = access.next_entry::<String, String>()? {
            slots.insert(&name, &value);
        }
        Ok(slots)
    }
}

impl<'de> Deserialize<'de> for Slots {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(SlotsVisitor)
    }
}

/// One proposition, independent of its wording.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Graph {
    pub frame: String,
    pub slots: Slots,
    pub polarity: String,
    pub mood: String,
}

impl Graph {
    #[must_use]
    pub fn new(frame: &str, slots: Slots, polarity: &str, mood: &str) -> Self {
        Self {
            frame: frame.to_owned(),
            slots,
            polarity: polarity.to_owned(),
            mood: mood.to_owned(),
        }
    }
}

/// Canonical identity of a meaning graph, independent of wording.
#[must_use]
pub fn graph_key(graph: &Graph) -> String {
    let mut names: Vec<&(String, String)> = graph.slots.0.iter().collect();
    names.sort_by(|left, right| super::js_order(&left.0, &right.0));
    let slots: Vec<String> = names
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect();
    format!(
        "{}({}) {} {}",
        graph.frame,
        slots.join(","),
        graph.polarity,
        graph.mood
    )
}
