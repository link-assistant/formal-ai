use ::formal_ai::callback_values::{
    CallbackTextReceiver, CallbackTrimReceiver, CallbackValueReceiver, callback_has_length,
    callback_nonempty, callback_text, callback_trim, callback_value,
};
use std::cell::Cell;

#[test]
fn string_trim_matches_ecmascript_boundary_whitespace_and_preserves_other_controls() {
    assert_eq!(callback_trim("\u{feff}\u{a0}value\u{3000}"), "value");
    assert_eq!(callback_trim("\u{85}value\u{85}"), "\u{85}value\u{85}");
    assert_eq!(
        callback_trim("\u{180e}value\u{180e}"),
        "\u{180e}value\u{180e}"
    );
    assert_eq!(
        ::formal_ai::meta_reasoner::text::js_trim("\u{feff} x \u{feff}"),
        "x"
    );
}
#[test]
fn strict_nonempty_comparison_does_not_coerce_json_primitives() {
    assert!(!callback_nonempty(""));
    assert!(callback_nonempty("0"));
    assert!(!callback_nonempty(&serde_json::json!("")));
    for value in [
        serde_json::Value::Null,
        serde_json::json!(0),
        serde_json::json!(false),
        serde_json::json!([]),
        serde_json::json!({}),
    ] {
        assert!(callback_nonempty(&value));
    }
}
#[test]
fn positive_length_preserves_empty_and_nonempty_text_and_sequence_cases() {
    assert!(!callback_has_length(""));
    assert!(callback_has_length("😀"));
    assert!(!callback_has_length(&[] as &[u8]));
    assert!(callback_has_length(&[0_u8][..]));
}
#[test]
fn text_and_value_getters_and_custom_trim_are_invoked_once_and_return_results_unchanged() {
    struct Receiver<'a>(&'a Cell<usize>);
    impl CallbackTextReceiver for Receiver<'_> {
        type Output = Result<u8, &'static str>;
        fn text(self) -> Self::Output {
            self.0.set(self.0.get() + 1);
            Err("getter failure")
        }
    }
    impl CallbackValueReceiver for Receiver<'_> {
        type Output = u8;
        fn value(self) -> Self::Output {
            self.0.set(self.0.get() + 1);
            42
        }
    }
    impl CallbackTrimReceiver for Receiver<'_> {
        type Output = u8;
        fn trim(self) -> Self::Output {
            self.0.set(self.0.get() + 1);
            7
        }
    }
    let calls = Cell::new(0);
    assert_eq!(callback_text(Receiver(&calls)), Err("getter failure"));
    assert_eq!(calls.get(), 1);
    assert_eq!(callback_value(Receiver(&calls)), 42);
    assert_eq!(calls.get(), 2);
    assert_eq!(callback_trim(Receiver(&calls)), 7);
    assert_eq!(calls.get(), 3);
}
#[test]
fn actual_native_research_consumer_matches_the_fixed_javascript_token_vectors() {
    use ::formal_ai::agentic_coding::coding_research_observation::observe_coding_research_acceptance_tokens;
    for (source, expected) in [
        ("const n = 3;", vec!["const", "n", "=", "3", ";"]),
        (
            "item.value + item.text",
            vec!["item", ".", "value", "+", "item", ".", "text"],
        ),
        ("[\"X\", 7]", vec!["[", "\"X\"", ",", "7", "]"]),
        (
            "function f(x) { return x; }",
            vec!["function", "f", "(", "x", ")", "{", "return", "x", ";", "}"],
        ),
    ] {
        let actual = observe_coding_research_acceptance_tokens(source);
        assert_eq!(actual["texts"], serde_json::json!(expected));
        assert_eq!(actual["complete"], true);
    }
}
#[test]
fn actual_seed_value_adapter_preserves_the_native_links_value_view() {
    let node = ::formal_ai::lino_adapters::links_notation::LinoNode {
        name: "relation".to_owned(),
        id: "exact value".to_owned(),
        children: vec![],
    };
    assert_eq!(callback_value(&node), "exact value");
}

#[test]
fn named_property_factory_captures_names_and_preserves_one_dispatch_and_borrowed_errors() {
    use ::formal_ai::callback_values::{CallbackPropertyReceiver, callback_property};
    use std::cell::RefCell;
    struct Receiver<'a> {
        calls: &'a Cell<usize>,
        names: &'a RefCell<Vec<String>>,
        value: &'a str,
    }
    impl<'a> CallbackPropertyReceiver for Receiver<'a> {
        type Output = Result<&'a str, &'static str>;
        fn property(self, name: &str) -> Self::Output {
            self.calls.set(self.calls.get() + 1);
            self.names.borrow_mut().push(name.to_owned());
            if name == "kind" {
                Ok(self.value)
            } else {
                Err("missing property")
            }
        }
    }
    let calls = Cell::new(0);
    let names = RefCell::new(Vec::new());
    let value = String::from("exact borrowed property");
    let property_name = String::from("kind");
    let observe = callback_property::<Receiver<'_>>(&property_name);
    let absent = callback_property::<Receiver<'_>>("other");
    assert_eq!(
        calls.get(),
        0,
        "factory construction is not receiver dispatch"
    );
    let output = observe(Receiver {
        calls: &calls,
        names: &names,
        value: &value,
    })
    .unwrap();
    assert_eq!(output.as_ptr(), value.as_ptr());
    assert_eq!(calls.get(), 1);
    assert_eq!(
        absent(Receiver {
            calls: &calls,
            names: &names,
            value: &value
        }),
        Err("missing property")
    );
    assert_eq!(calls.get(), 2);
    assert_eq!(&*names.borrow(), &["kind", "other"]);
}

#[test]
fn named_property_factory_returns_owned_values_and_does_not_intercept_receiver_panics() {
    use ::formal_ai::callback_values::{CallbackPropertyReceiver, callback_property};
    struct OwnedReceiver(String);
    impl CallbackPropertyReceiver for OwnedReceiver {
        type Output = Result<String, &'static str>;
        fn property(self, name: &str) -> Self::Output {
            if name == "owned" {
                Ok(self.0)
            } else {
                Err("missing owned property")
            }
        }
    }
    struct PanickingReceiver;
    impl CallbackPropertyReceiver for PanickingReceiver {
        type Output = ();
        fn property(self, name: &str) {
            panic!("receiver property {name}");
        }
    }
    assert_eq!(
        callback_property("owned")(OwnedReceiver("exact owned output".to_owned())),
        Ok("exact owned output".to_owned())
    );
    assert!(std::panic::catch_unwind(|| callback_property("observed")(PanickingReceiver)).is_err());
}

#[test]
fn actual_research_kind_property_consumer_preserves_token_vectors_and_malformed_refusal() {
    use ::formal_ai::agentic_coding::coding_research_observation::observe_coding_research_acceptance_tokens;
    for (source, texts, kinds) in [
        (
            "const value = 7;",
            vec!["const", "value", "=", "7", ";"],
            vec![
                "identifier",
                "identifier",
                "punctuator",
                "numeric",
                "punctuator",
            ],
        ),
        (
            "[\"X\", 7]",
            vec!["[", "\"X\"", ",", "7", "]"],
            vec![
                "punctuator",
                "string",
                "punctuator",
                "numeric",
                "punctuator",
            ],
        ),
    ] {
        let actual = observe_coding_research_acceptance_tokens(source);
        assert_eq!(
            actual,
            serde_json::json!({"texts":texts,"kinds":kinds,"complete":true})
        );
    }
    assert_eq!(
        observe_coding_research_acceptance_tokens("\""),
        serde_json::json!({"texts":[],"kinds":[],"complete":false})
    );
}
