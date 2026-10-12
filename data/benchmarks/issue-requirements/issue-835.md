## Summary

Formal AI should be able to **check files (images and other content) for legality** — ideally across as many jurisdictions as possible. This issue proposes a capability + reference model for such a check.

**Important reality check up front:** there is **no single global system or database** that can instantly verify a file's legality in *all* countries at once. Laws differ drastically by nation and there is no unified international compliance registry. So "legality check" must be decomposed into several independent categories, each with its own standards and tooling. Formal AI should implement the check as a **pipeline of category-specific detectors**, and clearly report *which category, which jurisdiction, and with what confidence* — never claim a blanket "legal everywhere".

## The three legal categories a file check must cover

### 1. State secrets, military & border restrictions
Many countries enforce strict national-security photography laws; possessing/transporting certain images can trigger arrest.
- **Prohibited subjects:** military bases, border checkpoints, airports, government buildings, critical infrastructure (bridges, dams, power plants).
- **High-risk regions:** e.g. North Korea, China, Egypt, Iran, Saudi Arabia, Cuba (non-exhaustive).
- **Detector approach:** object/scene classification to flag sensitive subjects; GPS/EXIF geolocation to flag high-risk regions.

### 2. Forbidden & extremist content
- **Child sexual abuse material (CSAM):** the **only** category with a truly global enforcement mechanism. Industry/law-enforcement worldwide use perceptual-hash matching such as **Microsoft PhotoDNA** and hash databases managed by **NCMEC**. Formal AI must **never store, generate, or reproduce** such content; the correct behavior is hash-based detection + refusal/report, not analysis. (See safety note below.)
- **Political / terrorist imagery:** banned-organization symbols, flags, propaganda are illegal in many jurisdictions (e.g. Germany, Russia, UK, several Middle-Eastern states).
- **Blasphemy / nudity:** religious-defamation or adult content can carry criminal penalties in strict-law jurisdictions (e.g. Pakistan, Iran, Afghanistan).
- **Detector approach:** perceptual-hash matching against known-illegal-content databases (via an authorized provider — we do **not** host such content ourselves), plus symbol/logo classifiers and NSFW classifiers, with **per-jurisdiction** policy mapping.

### 3. Copyright & intellectual property
- Using/storing someone else's copyrighted image without permission implicates international treaties like the **Berne Convention** (protects works across ~181 countries).
- **Detector approach:**
  - **Reverse-image search** (Google Images / Lens, TinEye) to find the original creator / prior publication.
  - **Metadata / EXIF analysis** (e.g. Metapicz-style parsing) to read embedded copyright holder, camera signature, GPS coordinates.
  - **Usage-rights / license filtering** to assess licensing restrictions.

## Proposed capability for Formal AI

A `check_file_legality(file)` capability that:

1. Runs each category detector independently (secrets/military, forbidden/extremist incl. CSAM-hash, copyright/IP).
2. Extracts and reports **EXIF/metadata** (author, GPS, timestamps, camera) with provenance.
3. Returns a **structured, per-category, per-jurisdiction risk report** with confidence scores and cited evidence — explicitly stating it is **not** a global "legal/illegal" verdict and **not** legal advice.
4. Is designed so the same pipeline generalizes from images to **other file types** (documents, audio, video) where analogous detectors exist.

## Open questions
- Which CSAM-hash-matching provider/integration can we use legally and safely **without** ever hosting the underlying content ourselves (e.g. an authorized PhotoDNA/NCMEC-backed API)?
- How do we maintain and version the **per-jurisdiction policy mapping** (which subjects/symbols/content are restricted where)?
- What is the right UX to communicate uncertainty and the "no global guarantee" caveat to users?

## Safety note
This capability is **defensive**: detect and refuse/flag illegal content, help users avoid inadvertently carrying restricted material, and respect copyright. It must **never** be built in a way that stores, reproduces, or helps produce illegal content (especially CSAM). CSAM handling must go through an authorized detection provider — hash-match and report only.

---

*Drafted from a research session on whether images/files can be checked for legality worldwide. Tools referenced as prior art: Google Lens/Images, TinEye, Microsoft PhotoDNA + NCMEC, Metapicz. This is not legal advice.*

