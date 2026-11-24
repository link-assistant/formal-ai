use super::{Language, SYNTHESIS_ROUTES, render};

#[test]
fn localized_skill_gaps_list_the_registered_routes_in_dispatch_order() {
    let routes = SYNTHESIS_ROUTES.join(", ");
    for language in [Language::English, Language::Russian] {
        let reply = render(Some("combine_measurements"), Some("python"), language);
        assert!(reply.contains(&routes), "{language:?}: {reply}");
        assert!(
            reply.contains("combine_measurements"),
            "{language:?}: {reply}"
        );
        assert!(reply.contains("python"), "{language:?}: {reply}");
    }
}
