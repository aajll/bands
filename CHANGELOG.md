# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `Band`, an enum of the five limit sets (range, upper, lower, single warning, single trip), each carrying exactly the limits it uses.
- `Plan`, built and validated once through `Plan::builder`, with `eval` for a plain classification and `eval_from` for classification with hysteresis from the previous severity.
- `NonFinite`, the policy for NaN and infinite samples, and `Severity` with predicates and the log names `OK`, `WARN_LOW`, `WARN_HIGH`, `TRIP_LOW`, `TRIP_HIGH` and `INVALID`.
- `Error`, naming the limit that failed validation where one is involved.
- `no_std` with no dependencies and `unsafe` forbidden.
- Contract and property tests, with CI gates on formatting, clippy pedantic, rustdoc, a `thumbv7em-none-eabihf` build, `cargo deny`, `cargo audit` and 100% line and region coverage.
