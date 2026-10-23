# Integration guide

How to add `bands` to a Rust project. The [API reference](https://docs.rs/bands) describes every public item and [design.md](design.md) explains the behaviour.

## Requirements

- Rust 1.85 or later (edition 2024). The minimum is recorded as `rust-version` in `Cargo.toml` and checked in CI.
- No runtime dependencies. The crate is `#![no_std]` and does not use `alloc`.

## Git dependency

Until the crate is published, depend on a release tag:

```toml
[dependencies]
bands = { git = "https://github.com/aajll/bands", tag = "v0.1.0" }
```

Pinning a tag rather than a branch keeps builds reproducible and makes an upgrade a deliberate change.

## Published crate

Once published, use the version from crates.io:

```toml
[dependencies]
bands = "0.1"
```

## Use

Build the plan where the configuration is loaded, so a bad limit is a start-up error rather than a silent `OK` at run time. Keep the previous severity per signal and pass it back for hysteresis:

```rust
use bands::{Band, NonFinite, Plan, Severity};

struct TemperatureMonitor {
    plan: Plan,
    last: Severity,
}

impl TemperatureMonitor {
    fn new() -> Result<Self, bands::Error> {
        let plan = Plan::builder(Band::Upper { hi: 85.0, hihi: 95.0 })
            .policy(NonFinite::Trip)
            .hysteresis(2.0)
            .build()?;
        Ok(Self { plan, last: Severity::Ok })
    }

    fn sample(&mut self, celsius: f32) -> Severity {
        self.last = self.plan.eval_from(self.last, celsius);
        self.last
    }
}

let mut monitor = TemperatureMonitor::new().unwrap();
assert_eq!(monitor.sample(96.0), Severity::TripHigh);
assert_eq!(monitor.sample(94.0), Severity::TripHigh); // inside the 2 degree margin
assert_eq!(monitor.sample(92.0), Severity::WarnHigh);
```

A disabled monitor is an `Option<Plan>` that is `None`; the crate has no band that always reports `OK`.

## Build and test

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo doc --no-deps
```

See [CONTRIBUTING.md](../CONTRIBUTING.md) for the coverage, dependency and audit gates CI runs.

## Cross builds

CI builds the library for `thumbv7em-none-eabihf` to prove it stays `no_std`:

```sh
rustup target add thumbv7em-none-eabihf
cargo build --target thumbv7em-none-eabihf
```

Any target with `core` works. Tests run on the host.
