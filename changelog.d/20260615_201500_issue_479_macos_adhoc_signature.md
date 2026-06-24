Fix macOS desktop release signing by re-sealing ad-hoc `.app` bundles with
`codesign` before DMG upload, and document the `v0.205.0` CI failure that left
Linux/Windows assets present but macOS assets absent.
