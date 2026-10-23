// SPDX-License-Identifier: MIT
//! One test per rule in `docs/design.md`, named for the rule it falsifies.

use bands::{Band, Error, Limit, NonFinite, Plan, Severity};

const RANGE: Band = Band::Range {
    lolo: 10.5,
    lo: 11.0,
    hi: 14.5,
    hihi: 15.0,
};

fn range() -> Plan {
    Plan::builder(RANGE).build().unwrap()
}

#[test]
fn range_band_classifies_every_region() {
    let plan = range();
    assert_eq!(plan.eval(10.0), Severity::TripLow);
    assert_eq!(plan.eval(10.8), Severity::WarnLow);
    assert_eq!(plan.eval(12.6), Severity::Ok);
    assert_eq!(plan.eval(14.8), Severity::WarnHigh);
    assert_eq!(plan.eval(15.2), Severity::TripHigh);
}

#[test]
fn upper_band_never_reports_low() {
    let plan = Plan::builder(Band::Upper {
        hi: 85.0,
        hihi: 95.0,
    })
    .build()
    .unwrap();
    assert_eq!(plan.eval(-1.0e6), Severity::Ok);
    assert_eq!(plan.eval(90.0), Severity::WarnHigh);
    assert_eq!(plan.eval(100.0), Severity::TripHigh);
}

#[test]
fn lower_band_never_reports_high() {
    let plan = Plan::builder(Band::Lower { lolo: 1.0, lo: 2.0 })
        .build()
        .unwrap();
    assert_eq!(plan.eval(1.0e6), Severity::Ok);
    assert_eq!(plan.eval(1.5), Severity::WarnLow);
    assert_eq!(plan.eval(0.5), Severity::TripLow);
}

#[test]
fn warn_band_only_warns() {
    let plan = Plan::builder(Band::Warn(50.0)).build().unwrap();
    assert_eq!(plan.eval(49.0), Severity::Ok);
    assert_eq!(plan.eval(1.0e6), Severity::WarnHigh);
}

#[test]
fn trip_band_only_trips() {
    let plan = Plan::builder(Band::Trip(50.0)).build().unwrap();
    assert_eq!(plan.eval(49.0), Severity::Ok);
    assert_eq!(plan.eval(50.5), Severity::TripHigh);
}

#[test]
fn band_limit_reports_only_the_limits_a_band_uses() {
    assert_eq!(RANGE.limit(Limit::Lolo), Some(10.5));
    assert_eq!(RANGE.limit(Limit::Lo), Some(11.0));
    assert_eq!(RANGE.limit(Limit::Hi), Some(14.5));
    assert_eq!(RANGE.limit(Limit::Hihi), Some(15.0));
    let upper = Band::Upper { hi: 1.0, hihi: 2.0 };
    assert_eq!(upper.limit(Limit::Lolo), None);
    assert_eq!(upper.limit(Limit::Lo), None);
    assert_eq!(upper.limit(Limit::Hi), Some(1.0));
    assert_eq!(upper.limit(Limit::Hihi), Some(2.0));
    let lower = Band::Lower { lolo: 1.0, lo: 2.0 };
    assert_eq!(lower.limit(Limit::Lolo), Some(1.0));
    assert_eq!(lower.limit(Limit::Lo), Some(2.0));
    assert_eq!(lower.limit(Limit::Hi), None);
    assert_eq!(Band::Warn(3.0).limit(Limit::Hi), Some(3.0));
    assert_eq!(Band::Warn(3.0).limit(Limit::Hihi), None);
    assert_eq!(Band::Trip(4.0).limit(Limit::Hihi), Some(4.0));
    assert_eq!(Band::Trip(4.0).limit(Limit::Lolo), None);
}

#[test]
fn a_sample_exactly_at_a_limit_is_not_past_it() {
    let plan = Plan::builder(RANGE).epsilon(0.0).build().unwrap();
    assert_eq!(plan.eval(10.5), Severity::WarnLow);
    assert_eq!(plan.eval(11.0), Severity::Ok);
    assert_eq!(plan.eval(14.5), Severity::Ok);
    assert_eq!(plan.eval(15.0), Severity::WarnHigh);
}

#[test]
fn epsilon_widens_each_limit_into_a_dead_band() {
    let plan = Plan::builder(RANGE).epsilon(0.1).build().unwrap();
    assert_eq!(plan.eval(14.55), Severity::Ok);
    assert_eq!(plan.eval(14.65), Severity::WarnHigh);
    assert_eq!(plan.eval(10.95), Severity::Ok);
    assert_eq!(plan.eval(10.85), Severity::WarnLow);
}

#[test]
fn trip_is_reported_before_warning_on_the_same_side() {
    let plan = Plan::builder(Band::Range {
        lolo: 1.0,
        lo: 1.0,
        hi: 2.0,
        hihi: 2.0,
    })
    .build()
    .unwrap();
    assert_eq!(plan.eval(0.0), Severity::TripLow);
    assert_eq!(plan.eval(3.0), Severity::TripHigh);
}

#[test]
fn non_finite_samples_follow_the_policy() {
    for (policy, expected) in [
        (NonFinite::Trip, Severity::TripHigh),
        (NonFinite::Warn, Severity::WarnHigh),
        (NonFinite::Ok, Severity::Ok),
        (NonFinite::Invalid, Severity::Invalid),
    ] {
        let plan = Plan::builder(RANGE).policy(policy).build().unwrap();
        for sample in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(plan.eval(sample), expected, "{policy:?} {sample}");
            assert_eq!(plan.eval_from(Severity::TripLow, sample), expected);
        }
    }
}

#[test]
fn non_finite_policy_is_on_the_high_side_even_for_a_lower_band() {
    let plan = Plan::builder(Band::Lower { lolo: 1.0, lo: 2.0 })
        .build()
        .unwrap();
    assert_eq!(plan.eval(f32::NAN), Severity::TripHigh);
}

#[test]
fn default_policy_is_trip() {
    assert_eq!(range().policy(), NonFinite::Trip);
    assert_eq!(NonFinite::default(), NonFinite::Trip);
}

#[test]
fn builder_defaults_and_accessors() {
    let plan = range();
    assert_eq!(plan.band(), RANGE);
    assert_eq!(plan.epsilon(), Plan::DEFAULT_EPSILON);
    assert_eq!(plan.hysteresis(), 0.0);
    let custom = Plan::builder(RANGE)
        .epsilon(0.5)
        .hysteresis(0.25)
        .policy(NonFinite::Ok)
        .build()
        .unwrap();
    assert_eq!(custom.epsilon(), 0.5);
    assert_eq!(custom.hysteresis(), 0.25);
    assert_eq!(custom.policy(), NonFinite::Ok);
}

#[test]
fn build_rejects_a_bad_epsilon_first() {
    for epsilon in [-1.0, f32::NAN, f32::INFINITY] {
        assert_eq!(
            Plan::builder(Band::Warn(f32::NAN)).epsilon(epsilon).build(),
            Err(Error::Epsilon)
        );
    }
}

#[test]
fn build_rejects_a_bad_hysteresis() {
    for hysteresis in [-1.0, f32::NAN, f32::NEG_INFINITY] {
        assert_eq!(
            Plan::builder(RANGE).hysteresis(hysteresis).build(),
            Err(Error::Hysteresis)
        );
    }
}

#[test]
fn build_rejects_a_non_finite_limit_and_names_it() {
    let bad = Band::Range {
        lolo: 1.0,
        lo: f32::NAN,
        hi: 3.0,
        hihi: 4.0,
    };
    assert_eq!(Plan::builder(bad).build(), Err(Error::NotFinite(Limit::Lo)));
    assert_eq!(
        Plan::builder(Band::Trip(f32::INFINITY)).build(),
        Err(Error::NotFinite(Limit::Hihi))
    );
    assert_eq!(
        Plan::builder(Band::Lower {
            lolo: f32::NEG_INFINITY,
            lo: 0.0
        })
        .build(),
        Err(Error::NotFinite(Limit::Lolo))
    );
}

#[test]
fn build_rejects_a_limit_beyond_max_limit_and_names_it() {
    let bad = Band::Upper {
        hi: 1.0,
        hihi: Plan::MAX_LIMIT * 2.0,
    };
    assert_eq!(
        Plan::builder(bad).build(),
        Err(Error::OutOfRange(Limit::Hihi))
    );
    let bad = Band::Lower {
        lolo: -Plan::MAX_LIMIT * 2.0,
        lo: 0.0,
    };
    assert_eq!(
        Plan::builder(bad).build(),
        Err(Error::OutOfRange(Limit::Lolo))
    );
    let edge = Band::Upper {
        hi: -Plan::MAX_LIMIT,
        hihi: Plan::MAX_LIMIT,
    };
    assert!(Plan::builder(edge).build().is_ok());
}

#[test]
fn build_rejects_limits_out_of_order_for_every_band() {
    let bad = [
        Band::Range {
            lolo: 2.0,
            lo: 1.0,
            hi: 3.0,
            hihi: 4.0,
        },
        Band::Range {
            lolo: 1.0,
            lo: 3.0,
            hi: 2.0,
            hihi: 4.0,
        },
        Band::Range {
            lolo: 1.0,
            lo: 2.0,
            hi: 4.0,
            hihi: 3.0,
        },
        Band::Upper { hi: 2.0, hihi: 1.0 },
        Band::Lower { lolo: 2.0, lo: 1.0 },
    ];
    for band in bad {
        assert_eq!(Plan::builder(band).build(), Err(Error::Order), "{band:?}");
    }
}

#[test]
fn build_accepts_equal_neighbouring_limits() {
    assert!(
        Plan::builder(Band::Range {
            lolo: 1.0,
            lo: 1.0,
            hi: 1.0,
            hihi: 1.0
        })
        .build()
        .is_ok()
    );
}

#[test]
fn hysteresis_moves_only_the_crossed_limit_inwards() {
    let plan = Plan::builder(RANGE)
        .epsilon(0.0)
        .hysteresis(0.2)
        .build()
        .unwrap();
    // In TripHigh, hihi becomes 14.8.
    assert_eq!(plan.eval_from(Severity::TripHigh, 14.9), Severity::TripHigh);
    assert_eq!(plan.eval_from(Severity::TripHigh, 14.7), Severity::WarnHigh);
    // In WarnHigh, hi becomes 14.3, and hihi is back at 15.0.
    assert_eq!(plan.eval_from(Severity::WarnHigh, 14.4), Severity::WarnHigh);
    assert_eq!(plan.eval_from(Severity::WarnHigh, 14.2), Severity::Ok);
    assert_eq!(plan.eval_from(Severity::WarnHigh, 14.9), Severity::WarnHigh);
    // In WarnLow, lo becomes 11.2.
    assert_eq!(plan.eval_from(Severity::WarnLow, 11.1), Severity::WarnLow);
    assert_eq!(plan.eval_from(Severity::WarnLow, 11.3), Severity::Ok);
    // In TripLow, lolo becomes 10.7.
    assert_eq!(plan.eval_from(Severity::TripLow, 10.6), Severity::TripLow);
    assert_eq!(plan.eval_from(Severity::TripLow, 10.8), Severity::WarnLow);
    // Ok and Invalid adjust nothing.
    assert_eq!(plan.eval_from(Severity::Ok, 14.9), Severity::WarnHigh);
    assert_eq!(plan.eval_from(Severity::Invalid, 14.9), Severity::WarnHigh);
}

#[test]
fn hysteresis_applies_to_every_band_variant() {
    let upper = Plan::builder(Band::Upper { hi: 1.0, hihi: 2.0 })
        .epsilon(0.0)
        .hysteresis(0.5)
        .build()
        .unwrap();
    assert_eq!(
        upper.eval_from(Severity::WarnHigh, 0.75),
        Severity::WarnHigh
    );
    assert_eq!(
        upper.eval_from(Severity::TripHigh, 1.75),
        Severity::TripHigh
    );
    let lower = Plan::builder(Band::Lower { lolo: 1.0, lo: 2.0 })
        .epsilon(0.0)
        .hysteresis(0.5)
        .build()
        .unwrap();
    assert_eq!(lower.eval_from(Severity::WarnLow, 2.25), Severity::WarnLow);
    assert_eq!(lower.eval_from(Severity::TripLow, 1.25), Severity::TripLow);
    let warn = Plan::builder(Band::Warn(1.0))
        .epsilon(0.0)
        .hysteresis(0.5)
        .build()
        .unwrap();
    assert_eq!(warn.eval_from(Severity::WarnHigh, 0.75), Severity::WarnHigh);
    let trip = Plan::builder(Band::Trip(1.0))
        .epsilon(0.0)
        .hysteresis(0.5)
        .build()
        .unwrap();
    assert_eq!(trip.eval_from(Severity::TripHigh, 0.75), Severity::TripHigh);
}

#[test]
fn hysteresis_on_a_limit_the_band_lacks_adjusts_nothing() {
    let upper = Plan::builder(Band::Upper { hi: 1.0, hihi: 2.0 })
        .epsilon(0.0)
        .hysteresis(0.5)
        .build()
        .unwrap();
    assert_eq!(upper.eval_from(Severity::TripLow, 1.5), Severity::WarnHigh);
    assert_eq!(upper.eval_from(Severity::WarnLow, 0.5), Severity::Ok);
    let lower = Plan::builder(Band::Lower { lolo: 1.0, lo: 2.0 })
        .epsilon(0.0)
        .hysteresis(0.5)
        .build()
        .unwrap();
    assert_eq!(lower.eval_from(Severity::TripHigh, 1.5), Severity::WarnLow);
    assert_eq!(lower.eval_from(Severity::WarnHigh, 2.5), Severity::Ok);
    let warn = Plan::builder(Band::Warn(1.0))
        .epsilon(0.0)
        .hysteresis(0.5)
        .build()
        .unwrap();
    assert_eq!(warn.eval_from(Severity::TripHigh, 0.75), Severity::Ok);
    let trip = Plan::builder(Band::Trip(1.0))
        .epsilon(0.0)
        .hysteresis(0.5)
        .build()
        .unwrap();
    assert_eq!(trip.eval_from(Severity::WarnHigh, 0.75), Severity::Ok);
}

#[test]
fn hysteresis_larger_than_the_gap_lets_comparison_order_decide() {
    // hihi moves to 13.0, below hi at 14.5: a sample of 14.0 is above the
    // adjusted hihi, and the trip comparison comes first.
    let plan = Plan::builder(RANGE)
        .epsilon(0.0)
        .hysteresis(2.0)
        .build()
        .unwrap();
    assert_eq!(plan.eval_from(Severity::TripHigh, 14.0), Severity::TripHigh);
}

#[test]
fn severity_predicates_and_names() {
    let table = [
        (Severity::Ok, false, false, false, false, "OK"),
        (Severity::WarnLow, true, false, true, false, "WARN_LOW"),
        (Severity::WarnHigh, true, false, false, true, "WARN_HIGH"),
        (Severity::TripLow, false, true, true, false, "TRIP_LOW"),
        (Severity::TripHigh, false, true, false, true, "TRIP_HIGH"),
        (Severity::Invalid, false, false, false, false, "INVALID"),
    ];
    for (sev, warn, trip, low, high, name) in table {
        assert_eq!(sev.is_warn(), warn, "{sev:?}");
        assert_eq!(sev.is_trip(), trip, "{sev:?}");
        assert_eq!(sev.is_low(), low, "{sev:?}");
        assert_eq!(sev.is_high(), high, "{sev:?}");
        assert_eq!(sev.name(), name);
        assert_eq!(sev.to_string(), name);
    }
}

#[test]
fn errors_display_and_implement_error() {
    let errors: [(Error, &str); 5] = [
        (Error::NotFinite(Limit::Lo), "Lo limit is not finite"),
        (
            Error::OutOfRange(Limit::Hihi),
            "Hihi limit exceeds the maximum magnitude",
        ),
        (Error::Order, "limits are not in ascending order"),
        (Error::Epsilon, "epsilon is not finite and non-negative"),
        (
            Error::Hysteresis,
            "hysteresis is not finite and non-negative",
        ),
    ];
    for (error, text) in errors {
        assert_eq!(error.to_string(), text);
        let dynamic: &dyn core::error::Error = &error;
        assert!(dynamic.source().is_none());
    }
}

#[test]
fn types_are_plain_data() {
    let plan = range();
    let copy = plan;
    assert_eq!(plan, copy);
    assert!(format!("{plan:?}").contains("Plan"));
    assert!(format!("{:?}", Plan::builder(RANGE)).contains("Builder"));
}
