# Design

`bands` classifies a sample against a set of limits. A `Band` names which limits apply and carries their values; a `Plan` is a band whose limits, epsilon and hysteresis have been validated once. Evaluating a plan against a sample is a pure function with no error path: at most four comparisons, low side first, with a configured answer for a sample that is NaN or infinite.

## Goals

- Every configuration error is found when the plan is built, so evaluation has no error path.
- A failed sensor gives a defined, configured result rather than whatever a NaN comparison happens to produce.
- Evaluation is a pure function of the plan and the sample: no allocation, no global state, constant time.
- Invalid configurations are unrepresentable where the type system can do it, and rejected by `build` where it cannot.

Filtering, debouncing, latching, rate-of-change limits and timing are out of scope. A project that needs them builds them around the severity.

## Bands

A `Band` is an enum whose variants carry exactly the limits they use:

| Band | Limits | Severities it can produce |
| --- | --- | --- |
| `Range` | `lolo`, `lo`, `hi`, `hihi` | all four |
| `Upper` | `hi`, `hihi` | `WarnHigh`, `TripHigh` |
| `Lower` | `lolo`, `lo` | `WarnLow`, `TripLow` |
| `Warn(hi)` | `hi` | `WarnHigh` |
| `Trip(hihi)` | `hihi` | `TripHigh` |

Because a band cannot be missing a limit or carry one it ignores, the classes of error "required limit missing" and "extra limit for this type" do not exist. `Band::limit(Limit)` answers whether a band uses a limit and with what value.

## Validation

`Builder::build` checks, in order: the epsilon and the hysteresis are finite and non-negative; each limit the band uses is finite; each has a magnitude of at most `Plan::MAX_LIMIT`; and the limits the band uses ascend, `lolo <= lo <= hi <= hihi`, with equal neighbours allowed. It returns the first failure as an `Error` that names the limit where one is involved.

`Plan::MAX_LIMIT` bounds the limits, not the samples. Its value of 4,000,000 keeps every limit where the spacing between adjacent `f32` values is at most 0.25.

## Evaluation

A finite sample goes through at most four comparisons, low side first:

1. `sample < lolo - epsilon`: `TripLow`
2. `sample < lo - epsilon`: `WarnLow`
3. `sample > hihi + epsilon`: `TripHigh`
4. `sample > hi + epsilon`: `WarnHigh`

Each band runs only the comparisons for the limits it uses. Because a trip is tested before the warning on its side, a sample past both reports the trip. The comparisons are strict, so a sample exactly at a limit is not past it, and `epsilon` widens that boundary into a dead-band. With validated ordering the comparison order never changes the answer for a plain `eval`; it matters only when hysteresis moves a limit past its neighbour.

## Non-finite samples

A NaN compares false with everything, so without a check a NaN sample would pass all four comparisons and report `Ok`, and an infinite one would report a trip on one side. `eval` therefore tests `is_finite` first and hands a NaN or infinite sample to the plan's `NonFinite` policy:

| Policy | Result |
| --- | --- |
| `Trip` (default) | `TripHigh` |
| `Warn` | `WarnHigh` |
| `Ok` | `Ok` |
| `Invalid` | `Invalid` |

The trip and warning results are on the high side for every band, including `Lower`: they mean "act as if this limit tripped", not "the value is high". A caller that needs to tell a lost sensor from a real excursion uses `Invalid` and handles it itself.

## Hysteresis

`Plan::eval_from(prev, sample)` moves the one limit that `prev` crossed inwards by the plan's hysteresis, then classifies the sample against the adjusted band. The plan itself is never modified and the caller holds the previous severity, so the plan stays stateless and one plan can serve several signals.

Moving only the limit that was crossed keeps the adjustment local. A signal in `TripHigh` returns to `WarnHigh` once it falls below `hihi - hysteresis`; on the next call `prev` is `WarnHigh`, and the margin applies to `hi`. A `prev` that names a limit the band does not use, such as `TripLow` against an `Upper` band, adjusts nothing. A hysteresis larger than the gap between two limits moves a limit past its neighbour for that call only, and the fixed comparison order then decides the result. With a hysteresis of zero, `eval_from` equals `eval`.

## Floating-point limits

Limits, samples, epsilon and hysteresis are all `f32`, and each comparison adds or subtracts in `f32`. An epsilon smaller than half the spacing between `f32` values at a limit rounds away: `limit + epsilon` equals `limit`, and the dead-band is gone. The spacing is 2^-13 between 1024 and 2048 and doubles at every power of two, reaching 2^-8 (about 0.0039) at 32768, so the default epsilon of 1e-3 has no effect for limits of magnitude 32768 and above. Scale the epsilon with the limits, or keep limits in units that make them small. The same rounding applies to hysteresis.

## Decisions

### The band carries its limits

The C reference stores four limits in every configuration, marks an unused one with NaN, and offers a strict flag that rejects a finite value in an unused limit, because a type set wrongly is otherwise silently dropped. An enum with the limits as variant fields makes both the missing and the extra case impossible to write, so the strict flag, the reorder flag and three of the C status codes have no counterpart. A configuration loaded from a file or a panel is converted to a `Band` at the boundary, where a missing field is already an error of the loader.

### No reordering

The C reference can sort limits that arrive out of order. The consumer that has such a source sorts its own input before building the band; doing it in the library would silently accept a configuration whose author got the limits wrong, which is the failure the strict mode exists to catch.

### One policy enum

Three C flags select the non-finite result with a precedence rule. A single `NonFinite` enum has one value, so there is nothing to rank. The default is `Trip`, the most protective choice, matching the C default configuration.

### Hysteresis is part of the plan

The C reference takes the hysteresis on each evaluation call and returns `INVALID` when it is negative or not finite. Validating it in `build` with the limits removes the last error path from evaluation and keeps the margin next to the limits it belongs with. A signal that needs a different margin from the same limits builds a second plan; a `Plan` is `Copy` and a few words long.

### No disabled band

The C reference has a `NONE` type that reports `OK` for every sample. A monitor that is switched off is an `Option<Plan>` holding `None`, which the consumer cannot forget to check.

### Severity names match the C reference

`Severity::name` and its `Display` give `OK`, `WARN_LOW`, `WARN_HIGH`, `TRIP_LOW`, `TRIP_HIGH` and `INVALID`, so logs from a Rust consumer and a C consumer read the same.

## Portability

The crate is `#![no_std]`, uses only `core`, has no dependencies and forbids `unsafe`. It builds for any Rust target with `core`; CI builds it for `thumbv7em-none-eabihf`.

## Cost

`eval` is a finiteness test and at most four compares, each with one addition. `eval_from` adds one copy of the band and one addition. A `Plan` is a `Band` of up to four `f32` values plus two `f32` and a policy byte.

## Verification

- `tests/contract.rs` holds one test per rule in this document, named for the rule it falsifies, including every `Error` variant.
- `tests/properties.rs` holds property tests that, for any valid `Range` plan, the severity is monotone in the sample, a sample within epsilon of a limit is never past it, and `eval_from` with zero hysteresis equals `eval`.
- Doc examples in the README and the guides compile and run as doctests.
- CI gates on `cargo fmt`, clippy pedantic with warnings denied, `cargo doc` with warnings denied, the stable and minimum toolchains, a `no_std` target build, `cargo deny`, `cargo audit` and 100% line and region coverage of `src/`.
