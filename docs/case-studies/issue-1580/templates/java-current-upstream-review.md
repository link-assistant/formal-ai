# Java compatibility follow-up

The complete Java repository comparison remains unfinished in [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581). This addendum records current source observations on 2026-11-19; it does not replace earlier pinned or historical records.

All three linked issue bodies and all nine current comments were read. [JaCoCo #2347](https://github.com/jacoco/jacoco/issues/2347) remains open; current Maven Central metadata lists 0.8.16. Its older reporting/Doxia API needs a compatible isolated plugin realm. Formal AI has no Maven reporting plugin; the applicable practice is to preserve coverage and verify actual consuming APIs when replacing transitive dependencies.

[Plexus #127](https://github.com/codehaus-plexus/plexus-build-api/issues/127) is closed and [PR #139](https://github.com/codehaus-plexus/plexus-build-api/pull/139) merged on November 14. Current metadata for **org.codehaus.plexus:plexus-build-api** lists **1.4.0**. The complete published `DefaultProgress.java` source was downloaded and read: it retains `@MojoExecutionScoped` and removes `@Singleton`. This verifies publication of the source repair; a Maven injection run was not performed. The earlier captured advice to retain 1.2.0 is stale for this source defect. An initial lookup under the former `org.sonatype.plexus` group is retained and excluded from current-version conclusions.

[Spotless #2808](https://github.com/diffplug/spotless/issues/2808) remains open; current metadata lists 3.11.0. Its full current workaround exports five required javac packages to the formatter JVM. That grants access at the actual runtime boundary. Hiding Unsafe warnings alone does not repair the formatter. Formal AI has no Spotless dependency, so these JVM flags are not copied into its workflow.

The exact issue/comment payloads, Maven metadata, source artifact, source hashes and applicability decisions are retained under `experiments/issue-1580/evidence/checks/java-upstream-compatibility/`. The complete 62-active/145-historical file applicability map, Maven execution and current dependency safety remain open. Supplied source review earns zero independent Formal AI reasoning units.
