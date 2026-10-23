# bands

[![CI](https://github.com/aajll/bands/actions/workflows/ci.yml/badge.svg)](https://github.com/aajll/bands/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

**Classifies a sample as OK, warning or trip against validated LOLO, LO, HI and HIHI limits, for `no_std` control code.**

## Quick start

Add the crate as a git dependency; see the [integration guide](docs/integration.md) for the tag to pin and the other ways to depend on it.

```rust
use bands::{Band, Plan, Severity};

// A 12 V supply monitor: under-voltage trip and warning, over-voltage
// warning and trip. Validated once, at start-up.
let plan = Plan::builder(Band::Range { lolo: 10.5, lo: 11.0, hi: 14.5, hihi: 15.0 })
    .build()
    .expect("limits ascend and are finite");

let classified: Vec<Severity> = [12.6, 14.8, 15.2, 10.0].map(|v| plan.eval(v)).to_vec();
assert_eq!(
    classified,
    [Severity::Ok, Severity::WarnHigh, Severity::TripHigh, Severity::TripLow]
);
assert_eq!(classified[2].to_string(), "TRIP_HIGH");
```

## Description

A plan is built once from a band of limits and then evaluated against each sample. Building validates every limit, so evaluation has no error path at all.

- Five bands: full range (LOLO, LO, HI, HIHI), upper only, lower only, a single warning limit, and a single trip limit. Each carries exactly the limits it uses, so a limit cannot be missing or ignored
- A policy chooses what a NaN or infinite sample produces: trip, warning, OK or `Invalid`
- An epsilon dead-band at each limit, and hysteresis from the previous severity, stop a noisy signal flickering between levels
- Evaluation is a pure function of the plan and the sample: no state, no allocation, safe from any context that can read the plan
- `no_std` with no dependencies and no `unsafe`; tests cover every line and region, with property tests for monotonicity and the dead-band

Filtering, debouncing, latching, rate-of-change limits and timing are out of scope. A project that needs them builds them around the severity.

## Validated toolchains

| Toolchain | Target | Validation |
| --- | --- | --- |
| Rust stable and 1.85 | `x86_64` Linux | Contract and property tests, clippy pedantic, coverage gate |
| Rust stable | `thumbv7em-none-eabihf` | `no_std` library build |

See [cross builds](docs/integration.md#cross-builds).

## Documentation

- [Integration guide](docs/integration.md) — adding the dependency, minimum Rust version, and cross builds
- [API reference](https://docs.rs/bands) — rustdoc for every public item, or `cargo doc --open`
- [Design](docs/design.md) — bands, validation, evaluation order, non-finite samples, hysteresis, decisions, and verification
- [Changelog](CHANGELOG.md) — release history

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before you open a pull request. Report security issues as described in [SECURITY.md](.github/SECURITY.md).

## License

MIT. See [LICENSE](./LICENSE).
