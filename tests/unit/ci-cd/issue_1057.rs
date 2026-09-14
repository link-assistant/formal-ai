//! Regression coverage for issue #1057: Docker layers evicted the compiler cache.
//!
//! The GitHub Actions cache is one 10GB pool shared by everything in the
//! repository. It reached **10.01GB**, and the split was:
//!
//! ```text
//! buildkit blobs   5.26 GB   (42 blobs)
//! sccache          2.44 GB
//! everything else  2.31 GB
//! ```
//!
//! Docker layers held more than twice what sccache did, so GitHub evicted
//! compilation entries to make room for them. The macOS specification lane's
//! Rust hit rate fell from 48% to 27% between consecutive runs and the lane was
//! killed at its 1400s budget with the tests never starting -- a compile cache
//! that shrinks under load is worse than none, because the budgets were sized
//! for a cache that hits.
//!
//! Two writers were paying for layers nobody reads:
//!
//! - the pull-request image check, which since issue #1055 copies a prebuilt
//!   binary and therefore compiles nothing worth keeping;
//! - the Docker Hub publish steps, which export the same layers the GHCR step
//!   in the same job just exported.
//!
//! Both now read the cache and do not write to it. `mode=max` survives only
//! where a from-source build produces layers a later run can genuinely reuse.

use super::workflow_fixtures::release_workflow;

/// The pull-request image check does not write layers it did not build.
#[test]
fn the_prebuilt_image_check_does_not_export_layers() {
    let workflow = release_workflow();

    let step = workflow
        .split("- name: Build image")
        .nth(1)
        .expect("the pull-request check builds an image");
    let step = step.split("- name:").next().unwrap_or(step);

    assert!(
        step.contains("BINARY_SOURCE=prebuilt"),
        "this check copies a prebuilt binary (issue #1055)"
    );
    assert!(
        !step.contains("cache-to: type=gha"),
        "it compiles nothing worth caching, and every byte it exports evicts \
         an sccache entry from the shared 10GB pool. Step:\n{step}"
    );
}

/// One cache writer per release path, not two.
#[test]
fn the_docker_hub_steps_reuse_what_ghcr_exported() {
    let workflow = release_workflow();

    for step in workflow.split("\n      - name: ") {
        let name = step.lines().next().unwrap_or_default();
        if !name.contains("Publish Docker image to Docker Hub") {
            continue;
        }
        assert!(
            !step.contains("cache-to: type=gha"),
            "`{name}` exports the same layers the GHCR step in this job already \
             exported; a second writer only doubles what the pool holds"
        );
    }
}

/// A from-source publish still populates the cache.
///
/// Removing every writer would be the opposite mistake: the next release would
/// rebuild every layer from nothing.
#[test]
fn a_from_source_publish_still_exports_layers() {
    let workflow = release_workflow();

    assert!(
        workflow.contains("cache-to: type=gha,mode=max"),
        "at least one publish path must still export layers, or each release \
         rebuilds the image from scratch"
    );
}
