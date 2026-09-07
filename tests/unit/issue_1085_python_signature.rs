//! Issue #1085 (D5.3): Python signature and import parsing shared by the
//! synthesis handlers, moved out of the handler into `coding::python_signature`.

use formal_ai::python_signature::{import_preamble, is_parameter_list, split_top_level_commas};

#[test]
fn imports_are_collected_once_in_prompt_order() {
    let prompt = "Complete this.\n\nfrom typing import List\nimport math\nfrom typing import List\n\ndef f(x: List[int]) -> int:\n";
    assert_eq!(
        import_preamble(prompt),
        "from typing import List\nimport math"
    );
    assert_eq!(import_preamble("no imports here"), "");
}

#[test]
fn a_parameter_list_is_names_and_a_call_is_not() {
    assert!(is_parameter_list(""));
    assert!(is_parameter_list("numbers: List[float], threshold: float"));
    assert!(is_parameter_list("test_tup1, test_tup2"));
    assert!(is_parameter_list("*args, key=None, **kwargs"));
    assert!(is_parameter_list("pair: tuple[int, int] = (1, 2)"));
    assert!(!is_parameter_list("(3, 4, 5, 6),(5, 7, 4, 10)"));
    assert!(!is_parameter_list("[1.0, 2.0], 0.5"));
    assert!(!is_parameter_list("'hello'"));
}

#[test]
fn top_level_commas_respect_brackets() {
    assert_eq!(
        split_top_level_commas("a: dict[str, int], b=(1, 2), c"),
        vec!["a: dict[str, int]", " b=(1, 2)", " c"]
    );
}
