# pl-testdata

Polish test-data generator - PESEL numbers, NIP numbers, and full test-person profiles.

## Try it

A live instance is deployed at: https://pl-testdata.onrender.com

<img width="1031" height="848" alt="Screenshot from 2026-09-03 22-19-33" src="https://github.com/user-attachments/assets/74d5548c-e075-4745-adb4-d4c087d7d4eb" />

Note: the free-tier instance sleeps after ~15 minutes of inactivity - the
first request after idle time may take a few seconds to wake it up.

## Deployment

The app is deployed on [Render](https://render.com) as a web service, built
directly from source via Render's native Rust runtime (`cargo run --release`).

A `Dockerfile` is also included at the repo root for anyone who wants to run
the app in a container:

```bash
docker build -t pl-testdata .
docker run -p 3000:3000 -e PORT=3000 -e HOST=0.0.0.0 pl-testdata
```

CI (`.github/workflows/ci.yml`) runs `cargo fmt --check`, `cargo clippy`, and
`cargo test --workspace` on every push and pull request.

## Architecture

```
pl-testdata/
├── Cargo.toml               # workspace manifest, lists member crates
├── domain/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # Gender, DateOfBirth, Pesel - types, validation, tests
│       ├── generate.rs      # generate_pesel - seeded, constrained random Pesel generation
│       ├── nip.rs           # Nip, NipError, generate_nip - validated NIP construction/parsing/generation
│       ├── address.rs       # Address, PostalCode, PostalCodeError, generate_address
│       ├── person.rs        # Person, generate_person - a name paired with a coherent Pesel
│       ├── phone.rs         # PhoneNumber, PhoneNumberError, generate_phone_number
│       ├── email.rs         # EmailAddress, ReservedDomain, generate_email
│       ├── document.rs      # IdDocumentNumber, IdDocumentNumberError, generate_id_document_number
│       ├── iban.rs          # Iban, IbanError, generate_iban - ISO 7064 MOD 97-10 checksum
│       └── profile.rs       # TestProfile, generate_test_profile - assembles one full profile per seeded RNG
└── web/
    ├── Cargo.toml
    ├── assets/
    │   ├── page.html        # overall HTML shell: header, error slot, form, results, footer
    │   ├── form.html        # the generation-options form (gender, count, date range, seed)
    │   ├── results.html     # results table shell: seed banner, copy-all button, JSON payload
    │   ├── result_card.html # a single field-grid card template, one per generated person
    │   └── styles.css       # served as-is via GET /styles.css, no build step
    ├── src/
    │   ├── lib.rs           # module declarations, so the binary and integration tests share one crate
    │   ├── main.rs          # binary entry point: tracing setup + axum::serve
    │   ├── routes.rs        # GET /health, GET /, POST /generate, POST /api/v1/persons
    │   ├── dto.rs           # PersonDto, GenerateRequest/GenerateForm - the domain <-> HTTP boundary
    │   ├── error.rs         # ApiError - maps domain error types to HTTP 400s
    │   └── templates.rs     # dependency-free HTML rendering (form + results table)
    └── tests/
        └── api.rs           # integration tests against the router, including a seeded-determinism test
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

Or against the live instance:

```bash
curl -X POST https://pl-testdata.onrender.com/api/v1/persons \
  -H 'content-type: application/json' \
  -d '{"gender": "female", "min_date": "1990-01-01", "max_date": "1999-12-31", "seed": 42, "count": 3}'
```

Host/port are configurable via `HOST` and `PORT` environment variables
(defaults: `127.0.0.1:3000`).

## Tooling

- `rustfmt`, run on save via Zed + rust-analyzer.
- `clippy`, surfaced as inline diagnostics via rust-analyzer.
