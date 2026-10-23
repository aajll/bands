// SPDX-License-Identifier: MIT
#![doc = include_str!("../README.md")]
#![no_std]

use core::fmt;

/// Which limits a plan compares a sample against, with their values.
///
/// Each variant carries exactly the limits it uses, so a plan cannot be
/// missing a limit or carry one it ignores. Limits follow the process
/// convention: a sample below `lo` is a low warning, below `lolo` a low
/// trip, above `hi` a high warning and above `hihi` a high trip.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Band {
    /// Four limits: `lolo <= lo <= hi <= hihi`.
    Range {
        /// Low trip limit.
        lolo: f32,
        /// Low warning limit.
        lo: f32,
        /// High warning limit.
        hi: f32,
        /// High trip limit.
        hihi: f32,
    },
    /// Two high limits: `hi <= hihi`.
    Upper {
        /// High warning limit.
        hi: f32,
        /// High trip limit.
        hihi: f32,
    },
    /// Two low limits: `lolo <= lo`.
    Lower {
        /// Low trip limit.
        lolo: f32,
        /// Low warning limit.
        lo: f32,
    },
    /// One high limit that produces a warning.
    Warn(f32),
    /// One high limit that produces a trip.
    Trip(f32),
}

/// One of the four limit positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Limit {
    /// Low trip.
    Lolo,
    /// Low warning.
    Lo,
    /// High warning.
    Hi,
    /// High trip.
    Hihi,
}

impl Limit {
    const ASCENDING: [Limit; 4] = [Limit::Lolo, Limit::Lo, Limit::Hi, Limit::Hihi];
}

impl Band {
    /// The value of one limit, or `None` when this band does not use it.
    #[must_use]
    pub const fn limit(self, which: Limit) -> Option<f32> {
        match (self, which) {
            (Band::Range { lolo, .. } | Band::Lower { lolo, .. }, Limit::Lolo) => Some(lolo),
            (Band::Range { lo, .. } | Band::Lower { lo, .. }, Limit::Lo) => Some(lo),
            (Band::Range { hi, .. } | Band::Upper { hi, .. } | Band::Warn(hi), Limit::Hi) => {
                Some(hi)
            }
            (
                Band::Range { hihi, .. } | Band::Upper { hihi, .. } | Band::Trip(hihi),
                Limit::Hihi,
            ) => Some(hihi),
            _ => None,
        }
    }

    /// This band with one limit moved by `delta`, or unchanged when the band
    /// does not use that limit.
    const fn shifted(self, which: Limit, delta: f32) -> Self {
        match (self, which) {
            (Band::Range { lolo, lo, hi, hihi }, Limit::Lolo) => Band::Range {
                lolo: lolo + delta,
                lo,
                hi,
                hihi,
            },
            (Band::Range { lolo, lo, hi, hihi }, Limit::Lo) => Band::Range {
                lolo,
                lo: lo + delta,
                hi,
                hihi,
            },
            (Band::Range { lolo, lo, hi, hihi }, Limit::Hi) => Band::Range {
                lolo,
                lo,
                hi: hi + delta,
                hihi,
            },
            (Band::Range { lolo, lo, hi, hihi }, Limit::Hihi) => Band::Range {
                lolo,
                lo,
                hi,
                hihi: hihi + delta,
            },
            (Band::Upper { hi, hihi }, Limit::Hi) => Band::Upper {
                hi: hi + delta,
                hihi,
            },
            (Band::Upper { hi, hihi }, Limit::Hihi) => Band::Upper {
                hi,
                hihi: hihi + delta,
            },
            (Band::Lower { lolo, lo }, Limit::Lolo) => Band::Lower {
                lolo: lolo + delta,
                lo,
            },
            (Band::Lower { lolo, lo }, Limit::Lo) => Band::Lower {
                lolo,
                lo: lo + delta,
            },
            (Band::Warn(hi), Limit::Hi) => Band::Warn(hi + delta),
            (Band::Trip(hihi), Limit::Hihi) => Band::Trip(hihi + delta),
            (band, _) => band,
        }
    }

    /// Classify a finite sample. Low side first and trip before warning, so a
    /// sample past both limits on one side reports the trip. The comparisons
    /// are strict, so a sample exactly at a limit is not past it, and
    /// `epsilon` widens each limit into a dead-band.
    fn classify(self, epsilon: f32, sample: f32) -> Severity {
        if self
            .limit(Limit::Lolo)
            .is_some_and(|lolo| sample < lolo - epsilon)
        {
            Severity::TripLow
        } else if self
            .limit(Limit::Lo)
            .is_some_and(|lo| sample < lo - epsilon)
        {
            Severity::WarnLow
        } else if self
            .limit(Limit::Hihi)
            .is_some_and(|hihi| sample > hihi + epsilon)
        {
            Severity::TripHigh
        } else if self
            .limit(Limit::Hi)
            .is_some_and(|hi| sample > hi + epsilon)
        {
            Severity::WarnHigh
        } else {
            Severity::Ok
        }
    }
}

/// What a NaN or infinite sample evaluates to.
///
/// The trip and warning results are on the high side for every band,
/// including [`Band::Lower`]: they mean "act as if a limit tripped", not
/// "the value is high".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonFinite {
    /// [`Severity::TripHigh`]: a lost sensor takes protective action. The
    /// default.
    #[default]
    Trip,
    /// [`Severity::WarnHigh`]: operation continues with the operator told.
    Warn,
    /// [`Severity::Ok`]: the sample is ignored. For informational values only.
    Ok,
    /// [`Severity::Invalid`]: the caller tells a lost sensor from a real
    /// excursion itself.
    Invalid,
}

impl NonFinite {
    const fn severity(self) -> Severity {
        match self {
            NonFinite::Trip => Severity::TripHigh,
            NonFinite::Warn => Severity::WarnHigh,
            NonFinite::Ok => Severity::Ok,
            NonFinite::Invalid => Severity::Invalid,
        }
    }
}

/// The result of evaluating a sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// Within the normal range.
    Ok,
    /// Below the low warning limit.
    WarnLow,
    /// Above the high warning limit.
    WarnHigh,
    /// Below the low trip limit.
    TripLow,
    /// Above the high trip limit.
    TripHigh,
    /// A NaN or infinite sample under [`NonFinite::Invalid`].
    Invalid,
}

impl Severity {
    /// Whether this is a low or high warning.
    #[must_use]
    pub const fn is_warn(self) -> bool {
        matches!(self, Severity::WarnLow | Severity::WarnHigh)
    }

    /// Whether this is a low or high trip.
    #[must_use]
    pub const fn is_trip(self) -> bool {
        matches!(self, Severity::TripLow | Severity::TripHigh)
    }

    /// Whether this is a low warning or trip.
    #[must_use]
    pub const fn is_low(self) -> bool {
        matches!(self, Severity::WarnLow | Severity::TripLow)
    }

    /// Whether this is a high warning or trip.
    #[must_use]
    pub const fn is_high(self) -> bool {
        matches!(self, Severity::WarnHigh | Severity::TripHigh)
    }

    /// The upper-case name used in logs: `OK`, `WARN_LOW`, `WARN_HIGH`,
    /// `TRIP_LOW`, `TRIP_HIGH` or `INVALID`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Severity::Ok => "OK",
            Severity::WarnLow => "WARN_LOW",
            Severity::WarnHigh => "WARN_HIGH",
            Severity::TripLow => "TRIP_LOW",
            Severity::TripHigh => "TRIP_HIGH",
            Severity::Invalid => "INVALID",
        }
    }

    /// The limit a sample crossed to reach this severity, if any.
    const fn crossed(self) -> Option<Limit> {
        match self {
            Severity::TripLow => Some(Limit::Lolo),
            Severity::WarnLow => Some(Limit::Lo),
            Severity::WarnHigh => Some(Limit::Hi),
            Severity::TripHigh => Some(Limit::Hihi),
            Severity::Ok | Severity::Invalid => None,
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Why a plan could not be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Error {
    /// A limit is NaN or infinite.
    NotFinite(Limit),
    /// A limit's magnitude exceeds [`Plan::MAX_LIMIT`].
    OutOfRange(Limit),
    /// The limits the band uses do not ascend (`lolo <= lo <= hi <= hihi`).
    Order,
    /// The epsilon is NaN, infinite or negative.
    Epsilon,
    /// The hysteresis is NaN, infinite or negative.
    Hysteresis,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotFinite(limit) => write!(f, "{limit:?} limit is not finite"),
            Error::OutOfRange(limit) => write!(f, "{limit:?} limit exceeds the maximum magnitude"),
            Error::Order => f.write_str("limits are not in ascending order"),
            Error::Epsilon => f.write_str("epsilon is not finite and non-negative"),
            Error::Hysteresis => f.write_str("hysteresis is not finite and non-negative"),
        }
    }
}

impl core::error::Error for Error {}

/// Builds a [`Plan`], validating everything once.
#[derive(Debug, Clone, Copy, PartialEq)]
#[must_use = "a builder does nothing until `build` is called"]
pub struct Builder {
    band: Band,
    policy: NonFinite,
    epsilon: f32,
    hysteresis: f32,
}

impl Builder {
    /// What a NaN or infinite sample evaluates to. Defaults to
    /// [`NonFinite::Trip`].
    pub const fn policy(mut self, policy: NonFinite) -> Self {
        self.policy = policy;
        self
    }

    /// The dead-band at each limit. Defaults to [`Plan::DEFAULT_EPSILON`].
    pub const fn epsilon(mut self, epsilon: f32) -> Self {
        self.epsilon = epsilon;
        self
    }

    /// The margin [`Plan::eval_from`] moves a crossed limit inwards by.
    /// Defaults to zero.
    pub const fn hysteresis(mut self, hysteresis: f32) -> Self {
        self.hysteresis = hysteresis;
        self
    }

    /// Validate and build the plan.
    ///
    /// # Errors
    ///
    /// Returns the first of: a non-finite epsilon or hysteresis, or a
    /// negative one; a limit that is not finite; a limit whose magnitude
    /// exceeds [`Plan::MAX_LIMIT`]; limits out of order.
    pub fn build(self) -> Result<Plan, Error> {
        if !(self.epsilon.is_finite() && self.epsilon >= 0.0) {
            return Err(Error::Epsilon);
        }
        if !(self.hysteresis.is_finite() && self.hysteresis >= 0.0) {
            return Err(Error::Hysteresis);
        }
        let mut previous = f32::NEG_INFINITY;
        for which in Limit::ASCENDING {
            let Some(value) = self.band.limit(which) else {
                continue;
            };
            if !value.is_finite() {
                return Err(Error::NotFinite(which));
            }
            if value.abs() > Plan::MAX_LIMIT {
                return Err(Error::OutOfRange(which));
            }
            if value < previous {
                return Err(Error::Order);
            }
            previous = value;
        }
        Ok(Plan {
            band: self.band,
            policy: self.policy,
            epsilon: self.epsilon,
            hysteresis: self.hysteresis,
        })
    }
}

/// A validated set of limits that classifies samples.
///
/// Build one with [`Plan::builder`]. Evaluation is a pure function of the
/// plan and the sample with no error path, so one plan can serve any number
/// of signals from any context that can read it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plan {
    band: Band,
    policy: NonFinite,
    epsilon: f32,
    hysteresis: f32,
}

impl Plan {
    /// The largest magnitude a limit may have. Below it the spacing between
    /// adjacent `f32` values is at most 0.25, so limits keep a usable
    /// resolution.
    pub const MAX_LIMIT: f32 = 4_000_000.0;

    /// The dead-band a builder starts with.
    pub const DEFAULT_EPSILON: f32 = 1.0e-3;

    /// Start building a plan for `band` with the default policy, epsilon and
    /// hysteresis.
    pub const fn builder(band: Band) -> Builder {
        Builder {
            band,
            policy: NonFinite::Trip,
            epsilon: Self::DEFAULT_EPSILON,
            hysteresis: 0.0,
        }
    }

    /// Classify a sample.
    ///
    /// A NaN or infinite sample evaluates to the plan's [`NonFinite`] policy
    /// before any limit is examined.
    #[must_use]
    pub fn eval(&self, sample: f32) -> Severity {
        if !sample.is_finite() {
            return self.policy.severity();
        }
        self.band.classify(self.epsilon, sample)
    }

    /// Classify a sample given the previous severity of the same signal,
    /// with the limit that `prev` crossed moved inwards by the plan's
    /// hysteresis.
    ///
    /// A signal in [`Severity::TripHigh`] returns to [`Severity::WarnHigh`]
    /// once it falls below `hihi - hysteresis`; on the next call `prev` is
    /// `WarnHigh` and the margin applies to `hi`. Moving only the crossed
    /// limit keeps the adjustment local. A hysteresis larger than the gap
    /// between two limits moves a limit past its neighbour for that call
    /// only, and the fixed comparison order then decides the result.
    #[must_use]
    pub fn eval_from(&self, prev: Severity, sample: f32) -> Severity {
        if !sample.is_finite() {
            return self.policy.severity();
        }
        let band = match prev.crossed() {
            Some(limit @ (Limit::Hi | Limit::Hihi)) => self.band.shifted(limit, -self.hysteresis),
            Some(limit @ (Limit::Lolo | Limit::Lo)) => self.band.shifted(limit, self.hysteresis),
            None => self.band,
        };
        band.classify(self.epsilon, sample)
    }

    /// The limits.
    #[must_use]
    pub const fn band(&self) -> Band {
        self.band
    }

    /// What a NaN or infinite sample evaluates to.
    #[must_use]
    pub const fn policy(&self) -> NonFinite {
        self.policy
    }

    /// The dead-band at each limit.
    #[must_use]
    pub const fn epsilon(&self) -> f32 {
        self.epsilon
    }

    /// The margin [`eval_from`](Self::eval_from) applies.
    #[must_use]
    pub const fn hysteresis(&self) -> f32 {
        self.hysteresis
    }
}

#[cfg(doctest)]
#[doc = include_str!("../docs/integration.md")]
mod doc_integration {}

#[cfg(doctest)]
#[doc = include_str!("../docs/design.md")]
mod doc_design {}
