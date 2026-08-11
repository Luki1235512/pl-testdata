# pl-testdata

A reference project demonstrating modern Rust engineering practices, built
around a genuinely useful tool: a synthetic Polish test-data generator for
QA engineers - PESEL numbers, and eventually other identifiers (NIP, REGON,
IBAN) and full test-person profiles.

All data produced by this project is **synthetic and intended for software
testing only**. It is not derived from, and must never be presented as,
real individuals' data.

## Architecture

```
pl-testdata/
Cargo.toml # workspace manifest, lists member crates
domain/
Cargo.toml
src/
lib.rs # Gender, DateOfBirth, Pesel — types, validation, tests
```

**`domain/`** holds every domain type and rule, with zero web/UI/framework
dependencies. Eeb or WASM front end is coming
later, and keeping the domain crate framework-agnostic now means it can be
reused unchanged by a CLI, a web server, or a WASM bundle and it means the
core business rules stay
testable in plain `cargo test`, with no browser or server required.

## Running

```bash
cargo test    # runs all tests across all workspace members
```

Current coverage: `Gender`, `DateOfBirth`, and `Pesel` - construction,
round-tripping, century-encoding edge cases (1800s/1900s/2000s/2100s/2200s
offsets), and rejection of malformed/invalid input.

## Tooling

- `rustfmt`, run on save via Zed + rust-analyzer.
- `clippy`, surfaced as inline diagnostics via rust-analyzer.

## Roadmap / not yet built

These are known, intentional gaps. Sequenced deliberately rather than
missing by accident:

- **Random generator.** `Pesel::from_parts` builds a specific PESEL from
  given inputs; nothing yet _randomly generates_ plausible ones. This needs
  the `rand` crate wired in with a seedable RNG, so generation is
  reproducible in tests (same seed → same output) while still being
  genuinely random in normal use.
- **Additional identifiers** - NIP, REGON, IBAN. Each with their own
  checksum/format rules, following the same validated-newtype pattern as
  `Pesel`.
- **A `Person` aggregate type** combining `Pesel`, name, address, and other
  registration-relevant fields into one coherent generated identity.
- **CLI or web/WASM front end.** The domain crate has no consumer yet by
  design. The plan is a WASM front end (via `wasm-bindgen`) deployed to
  GitHub Pages, built only once the domain layer is solid.
- **CI pipeline.** No GitHub Actions workflow yet (`cargo test`, `cargo
clippy`, `cargo fmt --check`). Worth adding once there's a front end
  whose deploy it should also gate.
