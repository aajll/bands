# Contributing to bands

`bands` classifies samples against validated limits. A change must keep every configuration error in `build`, keep `eval` and `eval_from` pure functions with no error path, keep the comparison order and the non-finite policy as documented, and stay `no_std`, dependency-free and `unsafe`-free. See [docs/design.md](docs/design.md) for the rules the tests enforce.

## Getting started

These are the commands CI runs. Run them locally before opening a pull request.

```sh
# Format, lint and document with warnings denied
cargo fmt --check
cargo clippy --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

# Tests on stable and on the minimum supported Rust version
cargo test
cargo +1.85 test

# Prove the library stays no_std
rustup target add thumbv7em-none-eabihf
cargo build --target thumbv7em-none-eabihf

# Coverage gate: 100% line and region coverage of src/
cargo install cargo-llvm-cov --locked
rustup component add llvm-tools-preview
cargo llvm-cov --all-targets --fail-under-lines 100 --fail-under-regions 100

# Dependency policy and advisories
cargo install cargo-deny cargo-audit --locked
cargo deny check
cargo audit
```

## Source style

- `rustfmt` with the default configuration; CI checks `cargo fmt --check`.
- Clippy `pedantic` is enabled through the `[lints]` table in `Cargo.toml` and CI denies warnings. An `#[allow]` carries a comment explaining why.
- Every source file starts with an SPDX licence tag.

## Language rules

- `#![no_std]` with no `alloc` and no runtime dependencies. A dev-dependency is acceptable when it earns its place in the tests.
- `unsafe_code` is forbidden through `[lints.rust]`.
- Public items have rustdoc with an example where the behaviour is not obvious, and `missing_docs` is a warning that CI turns into an error.
- Invalid configurations are made unrepresentable by type where possible, as `Band` does for missing and extra limits, and rejected once in `Builder::build` otherwise. Evaluation never returns an error.
- Accessors and builder setters are `const fn` so a `Builder` can be set up in a `const` context.

## Tests and coverage

- `src/lib.rs` holds the implementation; unit tests belong there only for private helpers.
- `tests/contract.rs` holds one test per rule in `docs/design.md`, named for the rule it falsifies. A behaviour change updates the design document and the test together.
- `tests/properties.rs` holds `proptest` invariants over arbitrary valid plans and samples.
- The README and the guides under `docs/` are compiled and run as doctests through `include_str!` in `src/lib.rs`, so an example that stops compiling fails `cargo test`. Mark a fenced block that is not runnable Rust with its real language (`toml`, `sh`, `text`).
- Coverage is gated at 100% line and region coverage of `src/`. A new branch comes with the test that reaches it.

## Documentation

| File | Purpose |
| --- | --- |
| `README.md` | Landing page: quick start, description, validated toolchains, and links. Keep it short. |
| `docs/integration.md` | Adding the dependency, the minimum Rust version, build and cross builds. |
| rustdoc (`cargo doc`) | Reference for every public item. Changes with the interface. |
| `docs/design.md` | Bands, validation, evaluation order, non-finite samples, hysteresis, decisions, portability, cost and verification. |
| `CHANGELOG.md` | User-visible changes, under `[Unreleased]` until a release. |

Update rustdoc together with the interface. Update `docs/design.md` when behaviour changes, and record a design choice with its alternatives under its Decisions section. Write prose in British English, one line per paragraph, with no hard wrapping.

## Commits

Use Conventional Commits:

- `feat: ...` new feature
- `fix: ...` bug fix
- `docs: ...` documentation only
- `test: ...` test-only changes
- `chore: ...` build, CI, release work
- `refactor: ...` a change that neither fixes a bug nor adds a feature

Keep the subject under about 70 characters. Use the body to explain why the change is needed.

## Pull requests

- Open an issue first for non-trivial changes so the design can be agreed before implementation.
- Keep pull requests focused: one feature or one fix.
- All CI checks must pass.
