# Security Policy

## Reporting a vulnerability

Use GitHub's private vulnerability reporting on this repository. Expect a response within 7 days. A confirmed issue is fixed in a patch release.

## Scope

`bands` is a small `no_std` library with no dependencies, no `unsafe`, no allocation and no I/O. A plan is a few validated floats, and evaluation is a handful of comparisons that cannot panic or loop on any sample, including NaN and infinities. What a caller does with the severity is the caller's concern.
