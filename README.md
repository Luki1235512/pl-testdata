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

### Design decisions (and why)

- **Cargo workspace from day one**, even with a single member. Adding a
  `web` or `cli` crate later is just `cargo new`, not a restructure — the
  boundary between "domain logic" and "everything else" is set up before
  there's anything to entangle it with.
- **Newtype pattern for `Pesel`** — wraps a private `[u8; 11]`. The only way
  to obtain a `Pesel` is `Pesel::from_parts` (validated construction) or
  `Pesel::parse` (validated parsing). There is no path to a `Pesel` value
  with a bad checksum or malformed digits — invalid states are
  unrepresentable, not just checked.
- **Two separate construction paths, on purpose.** `from_parts` builds a
  correct PESEL from trusted components (it computes the checksum itself,
  so it can't get it wrong). `parse` validates an untrusted string from
  outside the program. Conflating "build correctly" with "validate
  externally" is a common source of bugs; keeping them as distinct methods
  makes the trust boundary explicit in the type signatures.
- **`DateOfBirth` wraps `chrono::NaiveDate`** rather than raw `(i32, u32,
u32)` fields passed around loosely. Calendar validity (leap years, days
  per month) is real, non-trivial logic — better delegated to a well-tested
  library and enforced once at construction than reimplemented or
  re-checked at every call site.
- **`Gender` is an enum, not a `bool` or `String`.** Eliminates an entire
  class of bugs (`"M"` vs `"male"` vs `true` meaning who-knows-what) — the
  compiler only accepts `Gender::Male` or `Gender::Female`, and every
  `match` on it is exhaustive by construction.
- **Errors are enums that implement `std::error::Error` and `Display`**,
  never bare strings. Callers can match on _why_ something failed
  (`PeselError::ChecksumMismatch { expected, actual }`, not just "invalid
  PESEL"), which matters both for good error messages in a future UI and
  for writing precise tests.

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
