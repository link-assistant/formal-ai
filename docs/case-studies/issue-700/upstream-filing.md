# Drafted upstream filing: link-foundation/si-units

Ready-to-file issue body for the upstream repository. **Do not file from
a fork** — main files it after review, per the batch protocol. Drafted
2026-09-30 on `qa-reasoning-coding-bulk-fixes`, out of issue #700 (E58).

---

## Title: Extract the SI dimension table and conversion algebra into a shared si-units package

### Summary

formal-ai needs unit conversion between any two units of the same
physical dimension (5 km → miles, 3 hp → watts, 32 psi → kPa) without a
pairwise record per combination. We implemented this as seed data plus
an exact-rational engine:

- a data table mapping ~66 units (SI, imperial, US customary, nautical,
  digital) to their SI dimension vector and exact factor to the coherent
  SI unit of that dimension;
- a dimension algebra (multiply, invert, compare) over the seven SI base
  dimensions;
- conversion as `value × factor(from) / factor(to)` in exact i128
  rational arithmetic, with honest `Incompatible`/`UnknownUnit`/`Overflow`
  outcomes instead of guesses.

This is generic infrastructure — a language-learning assistant is just
one consumer. Propose extracting it into `si-units`: a small, zero-dependency
package (Rust core, optional JS/WASM binding for the web editor) exposing:

```rust
pub struct Dimension(/* exponents over L M T I K N J */);
impl Dimension {
    pub fn parse(expr: &str) -> Option<Self>;   // "L2MT-3"
    pub fn multiply(&self, other: &Self) -> Option<Self>;
    pub fn inverse(&self) -> Option<Self>;
}
pub struct UnitEntry { pub name: String, pub dimension: Dimension, pub factor: Rational, }

pub enum Conversion { Converted(Rational), Incompatible(Dimension, Dimension), UnknownUnit(String), Overflow }
pub fn convert(value: Rational, from: &str, to: &str) -> Conversion;
```

```js
import { convert, dimensionOf } from "si-units";
convert("3 horsepower", "watt"); // { ok: true, value: "2237.099616" }
dimensionOf("psi");              // "L-1MT-2"
```

### Data provenance

Factors come from the defining agreements (international yard and pound,
standard atmosphere 101325 Pa, IT calorie 4.184 J, mechanical horsepower
745.699872 W, torr = 101325/760 Pa); exact rationals are kept (5/18 for
km/h). Unit identity anchors to Wikidata entities where we already carry
them (metre Q11573, kilometer Q828224, inch Q218593) — the package could
adopt the Q-ids as stable keys.

### Non-goals (first cut)

- Affine conversions (°C/°F offsets) — needs an offset field; we kept
  factor-only algebra deliberately so it cannot answer wrongly.
- Currencies (market data, not a quantity system).

### Offer

We can contribute the Rust engine and the data table (MIT or Apache-2.0,
whichever the foundation prefers) from the formal-ai tree.
