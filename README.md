# pl-testdata

A synthetic Polish test-data generator for
QA engineers - PESEL numbers, NIP numbers, and full test-person profiles.
Eventually other identifiers (REGON, IBAN) as well.

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
nip.rs # Nip, NipError, generate_nip - validated NIP construction/parsing/generation
person.rs # Person, generate_person - a name paired with a coherent Pesel
web/
Cargo.toml
src/
lib.rs # module declarations, so the binary and integration tests share one crate
main.rs # binary entry point: tracing setup + axum::serve
routes.rs # GET /health, GET /, POST /generate, POST /api/v1/persons
dto.rs # PersonDto, GenerateRequest/GenerateForm - the domain <-> HTTP boundary
error.rs # ApiError - maps domain error types to HTTP 400s
templates.rs # dependency-free HTML rendering (form + results table)
tests/
api.rs # integration tests against the router, including a seeded-determinism test
```

**`domain/`** holds every domain type and rule, with zero web/UI/framework
dependencies. Keeping the domain crate framework-agnostic means it can be
reused unchanged by a CLI, a web server, or a WASM bundle, and it means the
core business rules stay testable in plain `cargo test`, with no browser or
server required.

**`web/`** consumer of `domain`: an axum-based HTTP server
exposing a JSON API and a small server-rendered HTML page. It depends on
`domain` by path and is the only crate that knows about serde, HTTP status
codes, or HTML.

## Running

```bash
cargo test            # runs all tests across all workspace members
cargo run -p web       # starts the HTTP server on http://127.0.0.1:3000
```

Endpoints exposed by `web`:

| Method | Path              | Purpose                                            |
| ------ | ----------------- | -------------------------------------------------- |
| GET    | `/health`         | Liveness check                                     |
| GET    | `/`               | HTML form for generating test people               |
| POST   | `/generate`       | HTML form submission → results table               |
| POST   | `/api/v1/persons` | JSON API: generate one or more synthetic `Person`s |

Example JSON request:

```bash
curl -X POST localhost:3000/api/v1/persons \
  -H 'content-type: application/json' \
  -d '{"gender": "female", "min_date": "1990-01-01", "max_date": "1999-12-31", "seed": 42, "count": 3}'
```

Host/port are configurable via `HOST` and `PORT` environment variables
(defaults: `127.0.0.1:3000`).

## Tooling

- `rustfmt`, run on save via Zed + rust-analyzer.
- `clippy`, surfaced as inline diagnostics via rust-analyzer.

## Roadmap / not yet built

- **Additional identifiers** - REGON, IBAN, following the same
  validated-newtype pattern established by `Pesel` and `Nip`.
- **Address, phone, email fields on `Person`.** Each deserves its own
  small generator (postal codes and phone number formats have their own
  structure) rather than being bolted on all at once.
- **CI pipeline.** No GitHub Actions workflow yet (`cargo test`, `cargo
clippy`, `cargo fmt --check`). Worth adding now that there's a `web` crate
  whose deploy it should also gate.
- **Rate limiting**, if `web` is ever deployed somewhere publicly reachable
  rather than run locally.
