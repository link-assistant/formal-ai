#[test]
fn offline_capture_absence_at_every_endpoint_preserves_existing_health() {
    use formal_ai::service_accessibility::ServiceStatus;
    use formal_ai::source_fetch::FetchError;
    use formal_ai::source_walk::{
        WalkSourceOutcome, endpoint_key, observe_failure, trusted_sources,
    };
    let record = trusted_sources()
        .into_iter()
        .next()
        .expect("registry source");
    let url = "https://example.test/subject";
    for entry in [true, false] {
        for initial in [
            None,
            Some(ServiceStatus::Reachable),
            Some(ServiceStatus::Unreachable),
        ] {
            let mut health = ServiceAccessibilityCache::new(std::env::temp_dir());
            if let Some(status) = initial {
                health.observe(
                    endpoint_key(&record, url),
                    status,
                    "original observation",
                    20,
                );
            }
            let before = health.records().clone();
            let mut outcome = WalkSourceOutcome::new(&record.id, "no_items", url);
            observe_failure(
                &record,
                url,
                &FetchError::OfflineCacheMiss(url.to_owned()),
                &mut health,
                30,
                &mut outcome,
                entry,
            );
            assert_eq!(outcome.status, "offline_cache_miss");
            assert!(outcome.detail.contains(url));
            assert_eq!(
                health.records(),
                &before,
                "offline absence cannot rewrite service health"
            );
        }
    }
}

#[test]
fn live_provider_failures_keep_their_endpoint_specific_status() {
    use formal_ai::source_fetch::FetchError;
    use formal_ai::source_walk::{WalkSourceOutcome, observe_failure, trusted_sources};
    let record = trusted_sources()
        .into_iter()
        .next()
        .expect("registry source");
    let url = "https://example.test/subject";
    for entry in [true, false] {
        for (error, expected) in [
            (
                FetchError::Transport("no cached capture is arbitrary transport prose".into()),
                if entry {
                    "unreachable"
                } else {
                    "fallback_failed"
                },
            ),
            (
                FetchError::HttpStatus {
                    url: url.into(),
                    status: 404,
                },
                if entry { "no_entry" } else { "fallback_failed" },
            ),
        ] {
            let mut health = ServiceAccessibilityCache::new(std::env::temp_dir());
            let mut outcome = WalkSourceOutcome::new(&record.id, "no_items", url);
            observe_failure(&record, url, &error, &mut health, 30, &mut outcome, entry);
            assert_eq!(outcome.status, expected);
            assert_eq!(
                !health.records().is_empty(),
                entry && error.speaks_for_the_service()
            );
        }
    }
}
