# Isolated secret scanner

The default scanner uses this private manifest and committed dependency lock.
Run `bash scripts/check-secrets.sh` from the repository; it installs the lock in
a temporary directory using `npm ci --ignore-scripts --engine-strict`, scans the
selected filenames and removes the installation on success or failure. Package
installation never consumes the scanner's NUL filename stream.

To update the default deliberately, change the manifest, regenerate the lock
with `npm install --package-lock-only --ignore-scripts --prefix .github/secretlint`,
and keep the shell default and engine/version controls in sync. Run the maintained
`bounded-subprocess-boundaries.test.mjs` suite and the actual locked-engine probe.
Node 22 or later is required by Secretlint 14. Explicit custom-version overrides
retain their separate npm package request; they do not use this default lock.
