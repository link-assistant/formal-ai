//! The typed instruction set as the enumerator walks it.
//!
//! Interned operation ids and types, the measures a list can be filtered by,
//! the steps that extend a program of each type, and rendering of a program as
//! the JavaScript the answer shows.
//!
//! The JavaScript materialises every program of a length as an array; there
//! are millions at length three, so here the same programs are visited in
//! the same order by a depth-first walk over interned indices.

use std::collections::HashMap;
use std::sync::OnceLock;

use super::interpreter::Param;
use super::seed::{MetaSeed, meta_seed};

/// Index of an operation id in [`Catalog::ops`].
pub type OpId = u32;
/// Index of a type name in [`Catalog::types`].
pub type TypeId = u16;

/// A seed primitive or a composed measure, as a program step's operation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Prim {
    /// Operation id (`split_words`, or `split_characters∘count_items`).
    pub id: String,
    /// The id interned.
    pub op: OpId,
    /// Input type.
    pub from: String,
    /// Output type.
    pub to: String,
    /// The JavaScript of the operation.
    pub code: String,
    /// The parameter inference rule, empty when the operation takes none.
    pub infer: String,
    /// The runtime it needs, or empty.
    pub environment: String,
    /// True when the operation acts on the world (prints, runs, fails the
    /// process); such an operation is never part of a measure.
    pub effect: bool,
    /// The types of the values the operation reads from the request.
    pub takes: Vec<String>,
    /// For a measure: the primitive ids it composes (`["value"]` for the
    /// identity measure); for a seed primitive: empty.
    pub parts: Vec<OpId>,
    /// For a measure: the seed primitives it runs, in order.
    pub chain: Vec<usize>,
    /// True for a measure.
    pub measure: bool,
}

/// One program step: an operation, possibly mapped over a list, used as a
/// filter's measure, or applied to a file in place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    /// Index into [`Catalog::prims`].
    pub prim: u32,
    /// Index into [`Catalog::filters`] when the step filters a list.
    pub filter: Option<u16>,
    /// True when the operation is applied to each element.
    pub mapped: bool,
    /// True when the operation rewrites a file in place.
    pub rewrite: bool,
}

/// A filter or a selector with its comparison read from the seed's `test`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterDef {
    /// Filter id.
    pub id: String,
    /// The id interned.
    pub op: OpId,
    /// The JavaScript test.
    pub test: String,
    /// The measure's result type (`number` or `text`).
    pub measure: String,
    /// True for a selector: the step picks one element by its measure
    /// (`step.select` in `js/worker/formal_ai_worker_meta_synthesis.js`).
    pub select: bool,
}

#[derive(Debug, Clone, Copy)]
struct Extension {
    step: Step,
    to: TypeId,
}

/// The instruction set, interned.
#[derive(Debug, Default)]
pub struct Catalog {
    /// Every operation id the enumerator or the evidence can name.
    pub ops: Vec<String>,
    op_index: HashMap<String, OpId>,
    /// Seed primitives first (same order as the seed), then measures.
    pub prims: Vec<Prim>,
    /// The seed's filters, then its selectors.
    pub filters: Vec<FilterDef>,
    /// Every type a program can have.
    pub types: Vec<String>,
    type_index: HashMap<String, TypeId>,
    extensions: Vec<Vec<Extension>>,
    view: Vec<bool>,
    /// The `map_each` combinator's id.
    pub map_each: OpId,
    /// The `rewrite_file` combinator's id.
    pub rewrite_file: OpId,
    /// The `rewrite_file` combinator's code, `{f}` standing for the operation.
    pub rewrite_code: String,
}

/// The type an operation produces from a concrete input type, or `None`.
///
/// The prefix of every list type name (`list_text`, `list_number`, ...).
const LIST_PREFIX: &str = "list_";

/// Whether a type name denotes a list (`list_text`, `list_any`, ...).
///
/// Mirrors the `type.startsWith("list_")` tests of
/// `js/worker/formal_ai_worker_meta_synthesis.js`.
#[must_use]
pub fn is_list_type(name: &str) -> bool {
    name.strip_prefix(LIST_PREFIX).is_some()
}

/// Mirrors `metaApply` in `js/worker/formal_ai_worker_meta_synthesis.js`.
#[must_use]
pub fn apply_type(from: &str, to: &str, input: &str) -> Option<String> {
    if from == input || (from == "list_any" && is_list_type(input)) {
        return Some(if to == "list_any" {
            input.to_owned()
        } else {
            to.to_owned()
        });
    }
    None
}

impl Catalog {
    fn intern_op(&mut self, id: &str) -> OpId {
        if let Some(index) = self.op_index.get(id) {
            return *index;
        }
        let index = OpId::try_from(self.ops.len()).unwrap_or(OpId::MAX);
        self.ops.push(id.to_owned());
        self.op_index.insert(id.to_owned(), index);
        index
    }

    fn intern_type(&mut self, name: &str) -> TypeId {
        if let Some(index) = self.type_index.get(name) {
            return *index;
        }
        let index = TypeId::try_from(self.types.len()).unwrap_or(TypeId::MAX);
        self.types.push(name.to_owned());
        self.type_index.insert(name.to_owned(), index);
        index
    }

    /// The interned id of an operation, if the catalog knows it.
    #[must_use]
    pub fn op(&self, id: &str) -> Option<OpId> {
        self.op_index.get(id).copied()
    }

    /// The interned id of a type, if a program can have it.
    #[must_use]
    pub fn type_id(&self, name: &str) -> Option<TypeId> {
        self.type_index.get(name).copied()
    }

    /// The name of an interned type.
    #[must_use]
    pub fn type_name(&self, id: TypeId) -> &str {
        self.types.get(usize::from(id)).map_or("", String::as_str)
    }

    /// True when an operation only changes representation (split / join).
    #[must_use]
    pub fn is_view(&self, op: OpId) -> bool {
        self.view.get(op as usize).copied().unwrap_or(false)
    }

    /// The operation of a step.
    #[must_use]
    pub fn prim(&self, step: Step) -> &Prim {
        &self.prims[step.prim as usize]
    }

    /// Build the catalog from the seed.
    #[must_use]
    pub fn build(seed: &MetaSeed) -> Self {
        let mut catalog = Self::default();
        catalog.map_each = catalog.intern_op("map_each");
        catalog.rewrite_file = catalog.intern_op("rewrite_file");
        catalog.rewrite_code = seed
            .combinators
            .iter()
            .find(|combinator| combinator.id == "rewrite_file")
            .map(|combinator| combinator.code.clone())
            .unwrap_or_default();
        for combinator in &seed.combinators {
            catalog.intern_op(&combinator.id);
        }
        for primitive in &seed.primitives {
            let op = catalog.intern_op(&primitive.id);
            catalog.prims.push(Prim {
                id: primitive.id.clone(),
                op,
                from: primitive.from.clone(),
                to: primitive.to.clone(),
                code: primitive.code.clone(),
                infer: primitive.infer.clone(),
                environment: primitive.environment.clone(),
                effect: primitive.effect,
                takes: primitive.takes.clone(),
                parts: Vec::new(),
                chain: Vec::new(),
                measure: false,
            });
        }
        for (filter, select) in seed
            .filters
            .iter()
            .map(|filter| (filter, false))
            .chain(seed.selectors.iter().map(|selector| (selector, true)))
        {
            let op = catalog.intern_op(&filter.id);
            catalog.filters.push(FilterDef {
                id: filter.id.clone(),
                op,
                test: filter.test.clone(),
                measure: filter.measure.clone(),
                select,
            });
        }
        catalog.close_types(seed);
        let seed_count = seed.primitives.len();
        let list_types: Vec<String> = catalog
            .types
            .iter()
            .filter(|name| is_list_type(name))
            .cloned()
            .collect();
        let mut measures_by_element: HashMap<String, Vec<u32>> = HashMap::new();
        let mut text_measures: HashMap<String, Vec<u32>> = HashMap::new();
        for list in list_types {
            let element = list["list_".len()..].to_owned();
            if measures_by_element.contains_key(&element) {
                continue;
            }
            let measures = catalog.build_measures(&element, seed_count);
            measures_by_element.insert(element.clone(), measures);
            let contents = catalog.build_text_measures(&element, seed_count);
            text_measures.insert(element, contents);
        }
        let names = catalog.types.clone();
        let mut extensions = Vec::with_capacity(names.len());
        for name in &names {
            extensions.push(catalog.build_extensions(
                name,
                seed_count,
                &measures_by_element,
                &text_measures,
            ));
        }
        catalog.extensions = extensions;
        catalog.view = catalog.ops.iter().map(|id| seed.is_view(id)).collect();
        catalog
    }

    /// Every type reachable from the seed's types and the example types.
    fn close_types(&mut self, seed: &MetaSeed) {
        let mut pending: Vec<String> = [
            "text",
            "number",
            "list_number",
            "list_text",
            "list_any",
            "path",
            "unknown",
        ]
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
        for primitive in &seed.primitives {
            pending.push(primitive.from.clone());
            pending.push(primitive.to.clone());
        }
        while let Some(name) = pending.pop() {
            if name.is_empty() || self.type_index.contains_key(&name) {
                continue;
            }
            self.intern_type(&name);
            for primitive in &seed.primitives {
                if let Some(direct) = apply_type(&primitive.from, &primitive.to, &name) {
                    pending.push(direct);
                }
                if let Some(element) = name.strip_prefix("list_")
                    && let Some(mapped) = apply_type(&primitive.from, &primitive.to, element)
                    && !is_list_type(&mapped)
                {
                    pending.push(["list_", &mapped].concat());
                }
            }
            if let Some(element) = name.strip_prefix("list_") {
                pending.push(element.to_owned());
            }
        }
    }

    /// The measures a list of `element` can be filtered by: every short
    /// parameter-free program from the element type to a number, and a number
    /// element itself.
    ///
    /// Mirrors `metaMeasures` in `js/worker/formal_ai_worker_meta_synthesis.js`.
    fn build_measures(&mut self, element: &str, seed_count: usize) -> Vec<u32> {
        let mut out = Vec::new();
        if element == "number" {
            let op = self.intern_op("value");
            out.push(self.push_measure(Prim {
                id: String::from("value"),
                op,
                from: String::from("number"),
                to: String::from("number"),
                code: String::from("(input) => input"),
                infer: String::new(),
                environment: String::new(),
                effect: false,
                takes: Vec::new(),
                parts: vec![op],
                chain: Vec::new(),
                measure: true,
            }));
        }
        // A measure is any short parameter-free program from the element to a
        // number ("the lines of a file": read the file, split its lines, count).
        let candidates: Vec<(usize, String, String, String)> = self
            .prims
            .iter()
            .take(seed_count)
            .enumerate()
            .filter(|(_, primitive)| {
                primitive.infer.is_empty() && !primitive.effect && primitive.takes.is_empty()
            })
            .map(|(index, primitive)| {
                (
                    index,
                    primitive.from.clone(),
                    primitive.to.clone(),
                    primitive.environment.clone(),
                )
            })
            .collect();
        let mut frontier: Vec<(Vec<usize>, String, String)> =
            vec![(Vec::new(), element.to_owned(), String::new())];
        for _ in 1..=super::BOUNDS.measure_length {
            let mut next = Vec::new();
            for (chain, current, environment) in &frontier {
                for (index, from, to, own_environment) in &candidates {
                    let Some(produced) = apply_type(from, to, current) else {
                        continue;
                    };
                    let mut grown = chain.clone();
                    grown.push(*index);
                    let grown_environment = if environment.is_empty() {
                        own_environment.clone()
                    } else {
                        environment.clone()
                    };
                    if produced == "number" {
                        out.push(self.compose_measure(element, &grown, grown_environment));
                    } else {
                        next.push((grown, produced, grown_environment));
                    }
                }
            }
            frontier = next;
        }
        out
    }

    fn compose_measure(&mut self, element: &str, chain: &[usize], environment: String) -> u32 {
        let ids: Vec<String> = chain
            .iter()
            .map(|index| self.prims[*index].id.clone())
            .collect();
        let mut inner = String::from("input");
        for index in chain {
            inner = ["(", &self.prims[*index].code, ")(", &inner, ")"].concat();
        }
        let id = ids.join("∘");
        let op = self.intern_op(&id);
        let parts = ids.iter().map(|part| self.intern_op(part)).collect();
        self.push_measure(Prim {
            id,
            op,
            from: element.to_owned(),
            to: String::from("number"),
            code: ["(input) => ", &inner].concat(),
            infer: String::new(),
            environment,
            effect: false,
            takes: Vec::new(),
            parts,
            chain: chain.to_vec(),
            measure: true,
        })
    }

    /// The text measures of an element: the element itself when it is
    /// text, or what one operation reads from it (`read_file` for a path).
    ///
    /// Mirrors `metaMeasures(element, "text")` in
    /// `js/worker/formal_ai_worker_meta_synthesis.js`.
    fn build_text_measures(&mut self, element: &str, seed_count: usize) -> Vec<u32> {
        let mut out = Vec::new();
        if element == "text" {
            let op = self.intern_op("value");
            out.push(self.push_measure(Prim {
                id: String::from("value"),
                op,
                from: String::from("text"),
                to: String::from("text"),
                code: String::from("(input) => input"),
                infer: String::new(),
                environment: String::new(),
                effect: false,
                takes: Vec::new(),
                parts: vec![op],
                chain: Vec::new(),
                measure: true,
            }));
            return out;
        }
        let readers: Vec<usize> = self
            .prims
            .iter()
            .take(seed_count)
            .enumerate()
            .filter(|(_, primitive)| {
                primitive.from == element
                    && primitive.to == "text"
                    && primitive.infer.is_empty()
                    && !primitive.effect
                    && primitive.takes.is_empty()
            })
            .map(|(index, _)| index)
            .collect();
        for index in readers {
            let primitive = self.prims[index].clone();
            out.push(self.push_measure(Prim {
                parts: vec![primitive.op],
                chain: vec![index],
                measure: true,
                ..primitive
            }));
        }
        out
    }

    fn push_measure(&mut self, prim: Prim) -> u32 {
        let index = u32::try_from(self.prims.len()).unwrap_or(u32::MAX);
        self.prims.push(prim);
        index
    }

    /// The steps that extend a program of type `name`, in the order
    /// `metaPrograms` (`js/worker/formal_ai_worker_meta_synthesis.js`) appends them.
    fn build_extensions(
        &self,
        name: &str,
        seed_count: usize,
        measures: &HashMap<String, Vec<u32>>,
        text_measures: &HashMap<String, Vec<u32>>,
    ) -> Vec<Extension> {
        let mut out = Vec::new();
        let element = name.strip_prefix("list_");
        for (index, primitive) in self.prims.iter().enumerate().take(seed_count) {
            let prim = u32::try_from(index).unwrap_or(u32::MAX);
            if let Some(direct) = apply_type(&primitive.from, &primitive.to, name)
                && let Some(to) = self.type_id(&direct)
            {
                out.push(Extension {
                    step: Step {
                        prim,
                        filter: None,
                        mapped: false,
                        rewrite: false,
                    },
                    to,
                });
            }
            if let Some(element) = element
                && let Some(mapped) = apply_type(&primitive.from, &primitive.to, element)
                && !is_list_type(&mapped)
                && let Some(to) = self.type_id(&["list_", &mapped].concat())
            {
                out.push(Extension {
                    step: Step {
                        prim,
                        filter: None,
                        mapped: true,
                        rewrite: false,
                    },
                    to,
                });
            }
        }
        // A file is edited in place by any text transformation: read it,
        // transform the text, write it back ("rewrite_file").
        if (name == "path" || name == "list_path")
            && let Some(to) = self.type_id(name)
        {
            for (index, primitive) in self.prims.iter().enumerate().take(seed_count) {
                if primitive.from != "text"
                    || primitive.to != "text"
                    || !primitive.infer.is_empty()
                    || !primitive.takes.is_empty()
                {
                    continue;
                }
                out.push(Extension {
                    step: Step {
                        prim: u32::try_from(index).unwrap_or(u32::MAX),
                        filter: None,
                        mapped: name == "list_path",
                        rewrite: true,
                    },
                    to,
                });
            }
        }
        if let Some(element) = element
            && let Some(to) = self.type_id(name)
        {
            let numeric = measures.get(element).map(Vec::as_slice).unwrap_or_default();
            for measure in numeric {
                for (filter, definition) in self.filters.iter().enumerate() {
                    if definition.select || definition.measure != "number" {
                        continue;
                    }
                    out.push(Self::chooser(*measure, filter, to));
                }
            }
            // A filter comparing the element's content with a value.
            for (filter, definition) in self.filters.iter().enumerate() {
                if definition.select || definition.measure == "number" {
                    continue;
                }
                for measure in text_measures
                    .get(element)
                    .map(Vec::as_slice)
                    .unwrap_or_default()
                {
                    out.push(Self::chooser(*measure, filter, to));
                }
            }
            // A selector picks one element by its measure ("the longest word").
            if let Some(picked) = self.type_id(element) {
                for measure in numeric {
                    for (filter, definition) in self.filters.iter().enumerate() {
                        if definition.select {
                            out.push(Self::chooser(*measure, filter, picked));
                        }
                    }
                }
            }
        }
        out
    }

    /// A filter or selector step over a measure.
    fn chooser(measure: u32, filter: usize, to: TypeId) -> Extension {
        Extension {
            step: Step {
                prim: measure,
                filter: u16::try_from(filter).ok(),
                mapped: false,
                rewrite: false,
            },
            to,
        }
    }

    /// Visit every typed program from `from` of exactly `length` steps, in
    /// the order `metaPrograms` (`js/worker/formal_ai_worker_meta_synthesis.js`)
    /// lists them. `types[i]` is the type before step `i`; the last entry is
    /// the program's type. The walk stops after `ceiling` programs.
    pub fn for_each_program(
        &self,
        from: TypeId,
        length: usize,
        ceiling: &mut usize,
        visit: &mut dyn FnMut(&[Step], &[TypeId]),
    ) {
        let mut steps: Vec<Step> = Vec::with_capacity(length);
        let mut types: Vec<TypeId> = Vec::with_capacity(length + 1);
        types.push(from);
        self.walk(length, &mut steps, &mut types, ceiling, visit);
    }

    fn walk(
        &self,
        length: usize,
        steps: &mut Vec<Step>,
        types: &mut Vec<TypeId>,
        ceiling: &mut usize,
        visit: &mut dyn FnMut(&[Step], &[TypeId]),
    ) {
        if steps.len() == length {
            if *ceiling > 0 {
                *ceiling -= 1;
                visit(steps, types);
            }
            return;
        }
        let current = types.last().copied().unwrap_or_default();
        let Some(extensions) = self.extensions.get(usize::from(current)) else {
            return;
        };
        for extension in extensions {
            if *ceiling == 0 {
                return;
            }
            steps.push(extension.step);
            types.push(extension.to);
            self.walk(length, steps, types, ceiling, visit);
            steps.pop();
            types.pop();
        }
    }

    /// A step's operations for evidence scoring.
    ///
    /// Mirrors `metaStepOperations` in `js/worker/formal_ai_worker_meta_synthesis.js`.
    pub fn step_ops(&self, step: Step, out: &mut Vec<OpId>) {
        let prim = self.prim(step);
        if step.rewrite {
            out.push(prim.op);
            out.push(self.rewrite_file);
            if step.mapped {
                out.push(self.map_each);
            }
        } else if let Some(filter) = step.filter {
            let filter_op = self.filters[usize::from(filter)].op;
            if prim.id != "value" {
                out.push(prim.op);
            }
            out.push(filter_op);
        } else {
            out.push(prim.op);
            if step.mapped {
                out.push(self.map_each);
            }
        }
    }

    /// A step's operations with a filter's measure folded into the filter.
    ///
    /// Mirrors `step.filter ? [step.filter.id] : metaStepOperations(step)` in
    /// `js/worker/formal_ai_worker_meta_synthesis.js`.
    pub fn main_ops(&self, step: Step, out: &mut Vec<OpId>) {
        if let Some(filter) = step.filter {
            out.push(self.filters[usize::from(filter)].op);
        } else {
            self.step_ops(step, out);
        }
    }

    /// The filter or selector of a step.
    #[must_use]
    pub fn chooser_of(&self, step: Step) -> Option<&FilterDef> {
        step.filter.map(|index| &self.filters[usize::from(index)])
    }

    /// True when a step takes the program's parameter.
    ///
    /// Mirrors `metaStepIsParametric` in `js/worker/formal_ai_worker_meta_synthesis.js`.
    #[must_use]
    pub fn is_parametric(&self, step: Step) -> bool {
        let prim = self.prim(step);
        self.chooser_of(step).is_some_and(|chooser| !chooser.select)
            || !prim.infer.is_empty()
            || !prim.takes.is_empty()
    }

    /// The types of the values a parametric step reads: a filter's threshold
    /// (its measure type), an inferred number, or the values an operation
    /// takes.
    ///
    /// Mirrors `metaStepParameterTypes` in `js/worker/formal_ai_worker_meta_synthesis.js`.
    #[must_use]
    pub fn parameter_types(&self, step: Step) -> Vec<String> {
        let prim = self.prim(step);
        if let Some(chooser) = self.chooser_of(step).filter(|chooser| !chooser.select) {
            return vec![chooser.measure.clone()];
        }
        if !prim.infer.is_empty() {
            return vec![String::from("number")];
        }
        prim.takes.clone()
    }

    /// The environment one program step needs (`node` for the file system).
    ///
    /// Mirrors `metaStepEnvironment` in `js/worker/formal_ai_worker_meta_reasoner.js`.
    #[must_use]
    pub fn step_environment(&self, step: Step) -> &str {
        if step.rewrite {
            return "node";
        }
        &self.prim(step).environment
    }

    /// A short label for one program step.
    ///
    /// Mirrors `metaStepLabel` in `js/worker/formal_ai_worker_meta_synthesis.js`.
    #[must_use]
    pub fn step_label(&self, step: Step) -> String {
        let id = &self.prim(step).id;
        if step.rewrite {
            let inner = ["rewrite(", id, ")"].concat();
            return if step.mapped {
                ["each(", &inner, ")"].concat()
            } else {
                inner
            };
        }
        if let Some(filter) = step.filter {
            return [self.filters[usize::from(filter)].id.as_str(), "(", id, ")"].concat();
        }
        if step.mapped {
            ["each(", id, ")"].concat()
        } else {
            id.clone()
        }
    }

    /// The in-place edit of one file by a text transformation.
    ///
    /// Mirrors `metaRewriteCode` in `js/worker/formal_ai_worker_meta_synthesis.js`.
    #[must_use]
    pub fn rewrite_code_for(&self, code: &str) -> String {
        self.rewrite_code.replace("{f}", code)
    }

    /// Render a program as the JavaScript the answer shows and the verifier
    /// runs.
    ///
    /// Mirrors `metaRender` in `js/worker/formal_ai_worker_meta_synthesis.js`.
    #[must_use]
    pub fn render(&self, steps: &[Step], parameter: &Param) -> String {
        let literal = parameter.to_js();
        let mut lines = vec![
            String::from("function solution(input) {"),
            String::from("  let value = input;"),
        ];
        for step in steps {
            let prim = self.prim(*step);
            let code = parameter.substitute(&prim.code);
            if let Some(selector) = self.chooser_of(*step).filter(|chooser| chooser.select) {
                lines.push(
                    [
                        "  value = value.reduce((best, item) => ((",
                        &selector.test,
                        ")((",
                        &code,
                        ")(item), (",
                        &code,
                        ")(best)) ? item : best)); // ",
                        &selector.id,
                        " by ",
                        &prim.id,
                    ]
                    .concat(),
                );
                continue;
            }
            if step.rewrite {
                let rewrite = self.rewrite_code_for(&code);
                lines.push(if step.mapped {
                    [
                        "  value = value.map(",
                        &rewrite,
                        "); // each: rewrite_file by ",
                        &prim.id,
                    ]
                    .concat()
                } else {
                    [
                        "  value = (",
                        &rewrite,
                        ")(value); // rewrite_file by ",
                        &prim.id,
                    ]
                    .concat()
                });
                continue;
            }
            if let Some(filter) = step.filter {
                let filter = &self.filters[usize::from(filter)];
                let test = filter
                    .test
                    .strip_prefix("(measure, threshold) => ")
                    .unwrap_or(&filter.test);
                lines.push(
                    [
                        "  value = value.filter((item) => { const measure = (",
                        &code,
                        ")(item); const threshold = ",
                        &literal,
                        "; return ",
                        test,
                        "; }); // ",
                        &filter.id,
                        " by ",
                        &prim.id,
                    ]
                    .concat(),
                );
                continue;
            }
            lines.push(if step.mapped {
                ["  value = value.map(", &code, "); // each: ", &prim.id].concat()
            } else {
                ["  value = (", &code, ")(value); // ", &prim.id].concat()
            });
        }
        lines.push(String::from("  return value;"));
        lines.push(String::from("}"));
        lines.join("\n")
    }
}

/// The catalog of the embedded seed, built once.
#[must_use]
pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| Catalog::build(meta_seed()))
}
