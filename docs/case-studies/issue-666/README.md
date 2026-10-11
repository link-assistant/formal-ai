# Extension distribution (#666)

Drafted listing metadata, raster icon, gallery banner, existing UI screenshot,
telemetry statement and two-registry publishing workflow. The chosen id preserves
the existing manifest identity: `link-assistant.formal-ai-vscode`, rather than
renaming an installed extension. The workflow retains a VSIX artifact and gates
publishing on the prepared extension version compared with the preceding tag.
Marketplace/Open VSX tokens are optional: absent configuration is explicitly
reported as unavailable, with no false claim of publication.

A clean-profile Marketplace install-by-id smoke compares the installed version
with the artifact version, alongside the existing extension test suite. Publisher
account/namespace creation, initial live publication, Open VSX installation smoke,
and live extension-host behavior verification remain pending. No builds/tests or
publication were performed in this session.

Versioning: Rust Cargo.toml is canonical. Existing desktop and extension resource
preparation scripts synchronize their package versions to it; unprepared source
package.json files may show earlier versions. The engine package follows the
same policy through generate-web-distribution.py. Main should fold this policy
into CONTRIBUTING.md's release section.

Primary references reviewed on 2026-09-30:
[VS Code publication guide](https://code.visualstudio.com/api/working-with-extensions/publishing-extension)
requires a raster listing icon and describes publisher registration. It also
notes global PAT retirement on 2026-12-01; Microsoft Entra federation should
replace the drafted PAT path before then.
[Open VSX publication guide](https://github.com/eclipse-openvsx/openvsx/wiki/Publishing-Extensions)
describes namespace/token setup. ovsx 1.2.0 was read from the publisher's npm
registry, without installing or executing it.
