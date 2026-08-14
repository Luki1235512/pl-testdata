# pl-testdata

A synthetic Polish test-data generator for
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
lib.rs # Gender, DateOfBirth, Pesel - types, validation, tests
generate.rs # generate_pesel - seeded, constrained random Pesel generation
person.rs # Person, generate_person - a name paired with a coherent Pesel
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

Current coverage: `Gender`, `DateOfBirth`, `Pesel`, `generate::generate_pesel`,
and `person::generate_person` - construction, round-tripping, century-encoding
edge cases (1800s/1900s/2000s/2100s/2200s offsets), rejection of
malformed/invalid input, seeded-RNG determinism, constraint satisfaction
(date range, gender), and the `Person`-to-`Pesel` consistency invariants
(gender and date of birth always agree, gender-inflected surnames use the
correct suffix). A property-based test (`proptest`) checks that generated
PESELs satisfy arbitrary valid constraints and round-trip through `Pesel::parse`.

## Tooling

- `rustfmt`, run on save via Zed + rust-analyzer.
- `clippy`, surfaced as inline diagnostics via rust-analyzer.

## Roadmap / not yet built

- **Additional identifiers** - NIP, REGON, IBAN. Each with their own
  checksum/format rules, following the same validated-newtype pattern as
  `Pesel`.
- **Address, phone, email fields on `Person`.** Each deserves its own
  small generator (postal codes and phone number formats have their own
  structure) rather than being bolted on all at once.
- **CLI or web/WASM front end.** The domain crate has a consumer-ready
  type (`Person`) but no consumer yet by design. The plan is a WASM front
  end (via `wasm-bindgen`) deployed to GitHub Pages, built only once the
  domain layer is solid.
- **CI pipeline.** No GitHub Actions workflow yet (`cargo test`, `cargo
clippy`, `cargo fmt --check`). Worth adding once there's a front end
  whose deploy it should also gate.
