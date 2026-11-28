use super::{SESSION_ENV, declared_session, unresolved_session};

#[test]
fn declared_session_trims_the_configured_environment_value() {
    temp_env::with_var(SESSION_ENV, Some("  scoped-session  "), || {
        assert_eq!(declared_session().as_deref(), Some("scoped-session"));
    });
    temp_env::with_var(SESSION_ENV, Some("  "), || {
        assert_eq!(declared_session(), None);
    });
}

#[test]
fn unresolved_session_names_the_environment_setting() {
    let diagnostic = unresolved_session();
    assert!(diagnostic.contains(SESSION_ENV));
    assert!(!diagnostic.contains("{variable}"));
}
