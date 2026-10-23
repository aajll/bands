// SPDX-License-Identifier: MIT
//! Invariants over arbitrary valid plans and samples.

use bands::{Band, Plan, Severity};
use proptest::prelude::*;

/// Position on the number line: lower is worse on the low side, higher is
/// worse on the high side.
fn rank(sev: Severity) -> i8 {
    match sev {
        Severity::TripLow => -2,
        Severity::WarnLow => -1,
        Severity::Ok => 0,
        Severity::WarnHigh => 1,
        Severity::TripHigh => 2,
        Severity::Invalid => unreachable!("finite samples never give Invalid"),
    }
}

fn ascending_range() -> impl Strategy<Value = Band> {
    (
        -1000.0f32..1000.0,
        0.0f32..100.0,
        0.0f32..100.0,
        0.0f32..100.0,
    )
        .prop_map(|(lolo, a, b, c)| Band::Range {
            lolo,
            lo: lolo + a,
            hi: lolo + a + b,
            hihi: lolo + a + b + c,
        })
}

proptest! {
    #[test]
    fn severity_is_monotone_in_the_sample(
        band in ascending_range(),
        epsilon in 0.0f32..10.0,
        a in -2000.0f32..2000.0,
        b in -2000.0f32..2000.0,
    ) {
        let plan = Plan::builder(band).epsilon(epsilon).build().unwrap();
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        prop_assert!(rank(plan.eval(lo)) <= rank(plan.eval(hi)));
    }

    #[test]
    fn a_sample_within_epsilon_of_a_limit_is_not_past_it(
        band in ascending_range(),
        epsilon in 0.5f32..10.0,
        offset in -0.5f32..0.5,
    ) {
        let plan = Plan::builder(band).epsilon(epsilon).build().unwrap();
        let Band::Range { lolo, lo, hi, hihi } = band else { unreachable!() };
        prop_assert_ne!(plan.eval(hihi + offset), Severity::TripHigh);
        prop_assert!(!plan.eval(hi + offset).is_high());
        prop_assert!(!plan.eval(lo + offset).is_low());
        prop_assert_ne!(plan.eval(lolo + offset), Severity::TripLow);
    }

    #[test]
    fn zero_hysteresis_makes_eval_from_equal_eval(
        band in ascending_range(),
        sample in -3000.0f32..3000.0,
        prev in prop::sample::select(&[
            Severity::Ok, Severity::WarnLow, Severity::WarnHigh,
            Severity::TripLow, Severity::TripHigh, Severity::Invalid,
        ]),
    ) {
        let plan = Plan::builder(band).build().unwrap();
        prop_assert_eq!(plan.eval_from(prev, sample), plan.eval(sample));
    }

    #[test]
    fn hysteresis_only_ever_holds_the_previous_severity_longer(
        band in ascending_range(),
        hysteresis in 0.0f32..5.0,
        sample in -3000.0f32..3000.0,
    ) {
        let plan = Plan::builder(band).hysteresis(hysteresis).build().unwrap();
        let plain = plan.eval(sample);
        // From a high severity, the adjusted result is at least as high as
        // the plain one; from a low severity, at least as low.
        for prev in [Severity::WarnHigh, Severity::TripHigh] {
            prop_assert!(rank(plan.eval_from(prev, sample)) >= rank(plain));
        }
        for prev in [Severity::WarnLow, Severity::TripLow] {
            prop_assert!(rank(plan.eval_from(prev, sample)) <= rank(plain));
        }
    }
}
