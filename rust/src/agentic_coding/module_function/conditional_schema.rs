//! Source-only schema propagation. Plain-data and intrinsic ownership remain preconditions.
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
type Checked<T> = Result<T, &'static str>;
#[derive(Clone, Default)]
struct TypeNode {
    kind: String,
    binding: Option<usize>,
    fields: BTreeMap<String, usize>,
    item: Option<usize>,
    value: Option<Value>,
    options: Vec<usize>,
}
#[derive(Default)]
struct Arena {
    nodes: Vec<TypeNode>,
}
#[derive(Clone, Copy)]
struct Operand {
    type_id: usize,
    fresh: bool,
}
type Environment = BTreeMap<String, Operand>;
impl Arena {
    fn node(&mut self, kind: &str) -> usize {
        let identifier = self.nodes.len();
        self.nodes.push(TypeNode {
            kind: kind.to_owned(),
            ..TypeNode::default()
        });
        identifier
    }
    fn resolve(&self, mut identifier: usize) -> usize {
        while let Some(next) = self.nodes[identifier].binding {
            identifier = next;
        }
        identifier
    }
    fn kind(&self, identifier: usize) -> &str {
        &self.nodes[self.resolve(identifier)].kind
    }
    fn array(&mut self) -> usize {
        let identifier = self.node("array");
        let item = self.node("variable");
        self.nodes[identifier].item = Some(item);
        identifier
    }
    fn optional(&mut self, value: usize) -> usize {
        let identifier = self.node("optional");
        self.nodes[identifier].item = Some(value);
        identifier
    }
    fn scalar(kind: &str) -> bool {
        matches!(
            kind,
            "text" | "number" | "index" | "boolean" | "null" | "literal" | "scalar"
        )
    }
    fn constrain(&mut self, left: usize, right: usize) -> Checked<usize> {
        let left = self.resolve(left);
        let right = self.resolve(right);
        if left == right {
            return Ok(left);
        }
        let a = self.nodes[left].clone();
        let b = self.nodes[right].clone();
        if a.kind == "variable"
            || (a.kind == "scalar" && Self::scalar(&b.kind))
            || (a.kind == "sequence" && matches!(b.kind.as_str(), "array" | "text"))
        {
            self.nodes[left].binding = Some(right);
            return Ok(right);
        }
        if b.kind == "variable"
            || (b.kind == "scalar" && Self::scalar(&a.kind))
            || (b.kind == "sequence" && matches!(a.kind.as_str(), "array" | "text"))
        {
            self.nodes[right].binding = Some(left);
            return Ok(left);
        }
        if a.kind == "number" && b.kind == "index" {
            self.nodes[left].binding = Some(right);
            return Ok(right);
        }
        if b.kind == "number" && a.kind == "index" {
            self.nodes[right].binding = Some(left);
            return Ok(left);
        }
        if a.kind != b.kind {
            return Err("IncompatibleSourceConstraints");
        }
        match a.kind.as_str() {
            "record" => {
                for (name, value) in b.fields {
                    if let Some(previous) = a.fields.get(&name) {
                        self.constrain(*previous, value)?;
                    } else {
                        self.nodes[left].fields.insert(name, value);
                    }
                }
            }
            "array" | "optional" => {
                self.constrain(a.item.ok_or("InvalidType")?, b.item.ok_or("InvalidType")?)?;
            }
            "literal" | "exceptLiteral" if a.value != b.value => {
                return Err("IncompatibleDiscriminants");
            }
            _ => {}
        }
        Ok(left)
    }
    fn require(&mut self, identifier: usize, kind: &str) -> Checked<usize> {
        let required = self.node(kind);
        self.constrain(identifier, required)
    }
    fn field(&mut self, identifier: usize, name: &str) -> Checked<usize> {
        let record = self.require(identifier, "record")?;
        if let Some(value) = self.nodes[record].fields.get(name) {
            return Ok(*value);
        }
        let value = self.node("variable");
        self.nodes[record].fields.insert(name.to_owned(), value);
        Ok(value)
    }
    fn clone_type(
        &mut self,
        identifier: usize,
        memo: &mut BTreeMap<usize, usize>,
    ) -> Checked<usize> {
        let identifier = self.resolve(identifier);
        if let Some(value) = memo.get(&identifier) {
            return Ok(*value);
        }
        let old = self.nodes[identifier].clone();
        let copy = self.node(&old.kind);
        memo.insert(identifier, copy);
        for (key, value) in old.fields {
            let value = self.clone_type(value, memo)?;
            self.nodes[copy].fields.insert(key, value);
        }
        if let Some(item) = old.item {
            let item = self.clone_type(item, memo)?;
            self.nodes[copy].item = Some(item);
        }
        self.nodes[copy].value = old.value;
        Ok(copy)
    }
    fn schema(&self, identifier: usize, seen: &BTreeSet<usize>) -> Checked<Value> {
        let identifier = self.resolve(identifier);
        if seen.contains(&identifier) {
            return Err("RecursiveSchema");
        }
        let mut next = seen.clone();
        next.insert(identifier);
        let node = &self.nodes[identifier];
        match node.kind.as_str() {
            "variable" => Ok(json!({"kind":"json"})),
            "sequence" => Err("UnresolvedSequenceIntrinsic"),
            "record" => {
                let fields = node
                    .fields
                    .iter()
                    .map(|(key, value)| Ok((key.clone(), self.schema(*value, &next)?)))
                    .collect::<Checked<Map<String, Value>>>()?;
                Ok(json!({"kind":"record","fields":fields}))
            }
            "array" => Ok(
                json!({"kind":"array","item":self.schema(node.item.ok_or("InvalidType")?,&next)?}),
            ),
            "optional" => Ok(
                json!({"kind":"optional","value":self.schema(node.item.ok_or("InvalidType")?,&next)?}),
            ),
            "union" => Ok(
                json!({"kind":"union","options":node.options.iter().map(|value| self.schema(*value,&next)).collect::<Checked<Vec<_>>>()?}),
            ),
            _ => Ok(if let Some(value) = &node.value {
                json!({"kind":node.kind,"value":value})
            } else {
                json!({"kind":node.kind})
            }),
        }
    }
    fn expression(&mut self, node: &Value, environment: &Environment) -> Checked<Operand> {
        let plain = |type_id| Operand {
            type_id,
            fresh: false,
        };
        match node["op"].as_str().unwrap_or("") {
            "literal" => Ok(plain(self.node(match &node["value"] {
                Value::Null => "null",
                Value::String(_) => "text",
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                _ => return Err("UnsupportedSourceIR"),
            }))),
            "binding" => environment
                .get(node["name"].as_str().unwrap_or(""))
                .copied()
                .ok_or("UnknownBinding"),
            "member" => {
                let receiver = self.expression(&node["receiver"], environment)?;
                if node["name"] == "length" {
                    self.require(receiver.type_id, "sequence")?;
                    return Ok(plain(self.node("number")));
                }
                Ok(plain(self.field(
                    receiver.type_id,
                    node["name"].as_str().ok_or("MissingMember")?,
                )?))
            }
            "index" => {
                let receiver = self.expression(&node["receiver"], environment)?;
                let array = self.array();
                let receiver = self.constrain(receiver.type_id, array)?;
                let index = self.expression(&node["index"], environment)?;
                if !(node["index"]["op"] == "literal" && node["index"]["value"].as_u64().is_some())
                {
                    self.require(index.type_id, "index")?;
                }
                let item = self.nodes[self.resolve(receiver)]
                    .item
                    .ok_or("InvalidType")?;
                Ok(plain(self.optional(item)))
            }
            "===" | "!==" => {
                self.expression(&node["left"], environment)?;
                self.expression(&node["right"], environment)?;
                Ok(plain(self.node("boolean")))
            }
            ">" => {
                let left = self.expression(&node["left"], environment)?;
                self.require(left.type_id, "number")?;
                let right = self.expression(&node["right"], environment)?;
                self.require(right.type_id, "number")?;
                Ok(plain(self.node("boolean")))
            }
            "??" => {
                let left = self.expression(&node["left"], environment)?;
                let value = self.node("variable");
                let optional = self.optional(value);
                let left = self.constrain(left.type_id, optional)?;
                let right = self.expression(&node["right"], environment)?;
                if self.kind(right.type_id) == "null" {
                    Ok(plain(left))
                } else {
                    let value = self.nodes[self.resolve(left)].item.ok_or("InvalidType")?;
                    self.constrain(value, right.type_id)?;
                    Ok(plain(value))
                }
            }
            "conditional" => {
                let condition = self.expression(&node["condition"], environment)?;
                self.require(condition.type_id, "boolean")?;
                let yes = self.expression(&node["yes"], environment)?;
                let no = self.expression(&node["no"], environment)?;
                if self.kind(yes.type_id) == "null" {
                    Ok(plain(if self.kind(no.type_id) == "optional" {
                        no.type_id
                    } else {
                        self.optional(no.type_id)
                    }))
                } else if self.kind(no.type_id) == "null" {
                    Ok(plain(if self.kind(yes.type_id) == "optional" {
                        yes.type_id
                    } else {
                        self.optional(yes.type_id)
                    }))
                } else {
                    Ok(plain(self.constrain(yes.type_id, no.type_id)?))
                }
            }
            "template" => {
                for part in node["parts"].as_array().ok_or("MissingParts")? {
                    let value = self.expression(part, environment)?;
                    self.require(value.type_id, "scalar")?;
                }
                Ok(plain(self.node("text")))
            }
            "map" => {
                let receiver = self.expression(&node["receiver"], environment)?;
                let array = self.array();
                let receiver = self.constrain(receiver.type_id, array)?;
                let parameter = node["parameter"].as_str().ok_or("MissingParameter")?;
                if environment.contains_key(parameter) {
                    return Err("ShadowedMapper");
                }
                let item = self.nodes[self.resolve(receiver)]
                    .item
                    .ok_or("InvalidType")?;
                let mut child = environment.clone();
                child.insert(parameter.to_owned(), plain(item));
                let value = self.expression(&node["body"], &child)?;
                self.require(value.type_id, "scalar")?;
                let output = self.node("array");
                self.nodes[output].item = Some(value.type_id);
                Ok(Operand {
                    type_id: output,
                    fresh: true,
                })
            }
            "push" => {
                let receiver = self.expression(&node["receiver"], environment)?;
                if !receiver.fresh {
                    return Err("CallerMutation");
                }
                let array = self.array();
                let receiver = self.constrain(receiver.type_id, array)?;
                let argument = self.expression(&node["argument"], environment)?;
                self.require(argument.type_id, "scalar")?;
                let item = self.nodes[self.resolve(receiver)]
                    .item
                    .ok_or("InvalidType")?;
                self.constrain(item, argument.type_id)?;
                Ok(plain(self.node("number")))
            }
            "join" => {
                let receiver = self.expression(&node["receiver"], environment)?;
                let array = self.array();
                let receiver = self.constrain(receiver.type_id, array)?;
                let item = self.nodes[self.resolve(receiver)]
                    .item
                    .ok_or("InvalidType")?;
                self.require(item, "scalar")?;
                let separator = self.expression(&node["argument"], environment)?;
                self.require(separator.type_id, "text")?;
                Ok(plain(self.node("text")))
            }
            _ => Err("UnsupportedSourceIR"),
        }
    }
    fn program(&mut self, compiled: &Value) -> Checked<(usize, usize)> {
        if compiled["status"] != "parsed" {
            return Err("MissingConditionalIR");
        }
        let parameter = compiled["parameter"].as_str().ok_or("MissingParameter")?;
        let mut input = self.node("variable");
        let mut environment = Environment::from([(
            parameter.to_owned(),
            Operand {
                type_id: input,
                fresh: false,
            },
        )]);
        let mut result = None;
        for statement in compiled["ir"].as_array().ok_or("MissingIR")? {
            match statement["op"].as_str().unwrap_or("") {
                "const" => {
                    let name = statement["name"].as_str().ok_or("MissingBinding")?;
                    if environment.contains_key(name) {
                        return Err("ShadowedLocal");
                    }
                    let value = self.expression(&statement["value"], &environment)?;
                    if !value.fresh {
                        return Err("NonFreshLocal");
                    }
                    environment.insert(name.to_owned(), value);
                }
                "if" => {
                    let condition = self.expression(&statement["condition"], &environment)?;
                    self.require(condition.type_id, "boolean")?;
                    self.expression(&statement["body"]["value"], &environment)?;
                }
                "return" => {
                    let node = &statement["value"];
                    let condition = &node["condition"];
                    let path = if node["op"] == "conditional"
                        && condition["op"] == "==="
                        && condition["right"]["op"] == "literal"
                    {
                        property_path(&condition["left"])
                    } else {
                        None
                    };
                    if let Some((root, path)) = path.filter(|(root, _)| root == parameter) {
                        self.expression(condition, &environment)?;
                        let yes_input = self.clone_type(input, &mut BTreeMap::new())?;
                        let no_input = self.clone_type(input, &mut BTreeMap::new())?;
                        let mut yes_field = yes_input;
                        let mut no_field = no_input;
                        for name in path {
                            yes_field = self.field(yes_field, &name)?;
                            no_field = self.field(no_field, &name)?;
                        }
                        let yes_literal = self.node("literal");
                        self.nodes[yes_literal].value = Some(condition["right"]["value"].clone());
                        self.constrain(yes_field, yes_literal)?;
                        let no_literal = self.node("exceptLiteral");
                        self.nodes[no_literal].value = Some(condition["right"]["value"].clone());
                        self.constrain(no_field, no_literal)?;
                        let mut true_branch_environment = environment.clone();
                        let mut false_branch_environment = environment.clone();
                        true_branch_environment.insert(
                            root.clone(),
                            Operand {
                                type_id: yes_input,
                                fresh: false,
                            },
                        );
                        false_branch_environment.insert(
                            root,
                            Operand {
                                type_id: no_input,
                                fresh: false,
                            },
                        );
                        let yes = self
                            .expression(&node["yes"], &true_branch_environment)?
                            .type_id;
                        let no = self
                            .expression(&node["no"], &false_branch_environment)?
                            .type_id;
                        result = Some(if self.kind(no) == "null" {
                            if self.kind(yes) == "optional" {
                                yes
                            } else {
                                self.optional(yes)
                            }
                        } else if self.kind(yes) == "null" {
                            if self.kind(no) == "optional" {
                                no
                            } else {
                                self.optional(no)
                            }
                        } else {
                            self.constrain(yes, no)?
                        });
                        input = self.node("union");
                        self.nodes[input].options = vec![yes_input, no_input];
                    } else {
                        result = Some(self.expression(node, &environment)?.type_id);
                    }
                }
                _ => return Err("UnsupportedStatement"),
            }
        }
        Ok((input, result.ok_or("MissingResultType")?))
    }
}
fn property_path(node: &Value) -> Option<(String, Vec<String>)> {
    if node["op"] == "binding" {
        return Some((node["name"].as_str()?.to_owned(), Vec::new()));
    }
    if node["op"] == "member" {
        let (root, mut path) = property_path(&node["receiver"])?;
        path.push(node["name"].as_str()?.to_owned());
        return Some((root, path));
    }
    None
}
/// Mirrors deriveConditionalSchemaGraph without converting a conditional graph into a pure callable.
#[must_use]
pub(super) fn derive_conditional_schema_graph(first: &Value, second: &Value) -> Value {
    let result = (|| -> Checked<Value> {
        let mut arena = Arena::default();
        let (input, produced) = arena.program(&first["contract"]["conditionalIR"])?;
        let (consumed, result) = arena.program(&second["contract"]["conditionalIR"])?;
        let produced = arena.resolve(produced);
        if arena.kind(produced) != "optional" {
            return Err("MissingOptionalResult");
        }
        arena.constrain(arena.nodes[produced].item.ok_or("InvalidType")?, consumed)?;
        let seen = BTreeSet::new();
        let schemas = json!({"input":arena.schema(input,&seen)?,"produced":arena.schema(produced,&seen)?,"consumed":arena.schema(consumed,&seen)?,"result":arena.schema(result,&seen)?});
        Ok(
            json!({"kind":"conditional-schema-graph","schemas":schemas,"calls":[first["identity"],second["identity"]],"bindings":[first["binding"],second["binding"]],"guard":{"kind":"nonnull","call":0},
   "sourceEffects":"conditional","effects":"unknown","moduleEffects":[first["moduleEffects"],second["moduleEffects"]],"preconditions":first["contract"]["conditionalIR"]["preconditions"],"suppliedPerTaskSchemas":false,"authored":false,"goal":"unbound"}),
        )
    })();
    match result {
        Ok(graph) => graph,
        Err(reason) => json!({"kind":"gap","reason":reason}),
    }
}
