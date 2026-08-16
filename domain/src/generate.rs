use crate::{DateOfBirth, Gender, Pesel};
use chrono::{Datelike, NaiveDate};
use rand::RngExt;

const PESEL_MIN_YEAR: i32 = 1800;
const PESEL_MAX_YEAR: i32 = 2299;

#[derive(Debug, Clone, Default)]
pub struct PeselConstraints {
    pub gender: Option<Gender>,
    pub date_range: Option<(DateOfBirth, DateOfBirth)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerationError {
    InvalidDateRange { min: DateOfBirth, max: DateOfBirth },
    DateRangeOutsidePeselEncodableYears { min: DateOfBirth, max: DateOfBirth },
}

impl std::fmt::Display for GenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenerationError::InvalidDateRange { min, max } => {
                write!(f, "invalid date range: min ({min}) is after max ({max})")
            }
            GenerationError::DateRangeOutsidePeselEncodableYears { min, max } => {
                write!(
                    f,
                    "date range {min}..={max} falls outside the years PESEL can encode ({PESEL_MIN_YEAR}-{PESEL_MAX_YEAR})"
                )
            }
        }
    }
}

impl std::error::Error for GenerationError {}

pub fn generate_pesel(
    rng: &mut impl RngExt,
    constraints: &PeselConstraints,
) -> Result<Pesel, GenerationError> {
    let (min, max) = constraints.date_range.unwrap_or_else(default_date_range);

    if min > max {
        return Err(GenerationError::InvalidDateRange { min, max });
    }
    if min.year() < PESEL_MIN_YEAR || max.year() > PESEL_MAX_YEAR {
        return Err(GenerationError::DateRangeOutsidePeselEncodableYears { min, max });
    }

    let dob = random_date_in_range(rng, min, max);

    let gender = constraints.gender.unwrap_or_else(|| {
        if rng.random_bool(0.5) {
            Gender::Male
        } else {
            Gender::Female
        }
    });

    let serial: u16 = rng.random_range(0..=999);

    Ok(Pesel::from_parts(dob, gender, serial)
        .expect("range and serial were validated above; from_parts cannot fail here"))
}

fn random_date_in_range(rng: &mut impl RngExt, min: DateOfBirth, max: DateOfBirth) -> DateOfBirth {
    let min_days = to_naive_date(min).num_days_from_ce();
    let max_days = to_naive_date(max).num_days_from_ce();

    let chosen_days = rng.random_range(min_days..=max_days);
    let date = NaiveDate::from_num_days_from_ce_opt(chosen_days)
        .expect("day count came from a valid date's own day count");

    DateOfBirth::new(date.year(), date.month(), date.day())
        .expect("a NaiveDate is always a valid calendar date")
}

fn to_naive_date(dob: DateOfBirth) -> NaiveDate {
    NaiveDate::from_ymd_opt(dob.year(), dob.month(), dob.day())
        .expect("DateOfBirth always wraps a valid calendar date")
}

pub fn default_date_range() -> (DateOfBirth, DateOfBirth) {
    let today = chrono::Local::now().date_naive();
    let min = today - chrono::Duration::days(90 * 365);
    let max = today - chrono::Duration::days(18 * 365);
    (
        DateOfBirth::new(min.year(), min.month(), min.day())
            .expect("subtracting days from a valid date stays valid"),
        DateOfBirth::new(max.year(), max.month(), max.day())
            .expect("subtracting days from a valid date stays valid"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn narrow_constraints() -> PeselConstraints {
        PeselConstraints {
            gender: None,
            date_range: Some((
                DateOfBirth::new(1990, 1, 1).unwrap(),
                DateOfBirth::new(1999, 12, 31).unwrap(),
            )),
        }
    }

    #[test]
    fn same_seed_produces_identical_sequences() {
        let constraints = narrow_constraints();

        let mut rng_a = StdRng::seed_from_u64(42);
        let mut rng_b = StdRng::seed_from_u64(42);

        let sequence_a: Vec<String> = (0..20)
            .map(|_| {
                generate_pesel(&mut rng_a, &constraints)
                    .unwrap()
                    .to_string()
            })
            .collect();
        let sequence_b: Vec<String> = (0..20)
            .map(|_| {
                generate_pesel(&mut rng_b, &constraints)
                    .unwrap()
                    .to_string()
            })
            .collect();

        assert_eq!(sequence_a, sequence_b);
    }

    #[test]
    fn different_seeds_produce_different_sequences() {
        let constraints = narrow_constraints();

        let mut rng_a = StdRng::seed_from_u64(1);
        let mut rng_b = StdRng::seed_from_u64(2);

        let sequence_a: Vec<String> = (0..20)
            .map(|_| {
                generate_pesel(&mut rng_a, &constraints)
                    .unwrap()
                    .to_string()
            })
            .collect();
        let sequence_b: Vec<String> = (0..20)
            .map(|_| {
                generate_pesel(&mut rng_b, &constraints)
                    .unwrap()
                    .to_string()
            })
            .collect();

        assert_ne!(sequence_a, sequence_b);
    }

    #[test]
    fn generated_pesels_always_round_trip_through_parse() {
        let constraints = narrow_constraints();
        let mut rng = StdRng::seed_from_u64(7);

        for _ in 0..200 {
            let generated = generate_pesel(&mut rng, &constraints).unwrap();
            let reparsed = Pesel::parse(&generated.to_string())
                .expect("a generated Pesel must always be independently parseable");
            assert_eq!(reparsed, generated);
        }
    }

    #[test]
    fn respects_date_range_constraint() {
        let min = DateOfBirth::new(1990, 1, 1).unwrap();
        let max = DateOfBirth::new(1990, 3, 31).unwrap();
        let constraints = PeselConstraints {
            gender: None,
            date_range: Some((min, max)),
        };
        let mut rng = StdRng::seed_from_u64(3);

        for _ in 0..200 {
            let generated = generate_pesel(&mut rng, &constraints).unwrap();
            let dob = generated.date_of_birth();
            assert!(dob >= min && dob <= max, "{dob} outside [{min}, {max}]");
        }
    }

    #[test]
    fn respects_gender_constraint() {
        let constraints = PeselConstraints {
            gender: Some(Gender::Female),
            date_range: Some((
                DateOfBirth::new(1990, 1, 1).unwrap(),
                DateOfBirth::new(1990, 12, 31).unwrap(),
            )),
        };
        let mut rng = StdRng::seed_from_u64(11);

        for _ in 0..50 {
            let generated = generate_pesel(&mut rng, &constraints).unwrap();
            assert_eq!(generated.gender(), Gender::Female);
        }
    }

    #[test]
    fn rejects_inverted_date_range() {
        let min = DateOfBirth::new(2000, 1, 1).unwrap();
        let max = DateOfBirth::new(1990, 1, 1).unwrap();
        let constraints = PeselConstraints {
            gender: None,
            date_range: Some((min, max)),
        };
        let mut rng = StdRng::seed_from_u64(0);

        assert_eq!(
            generate_pesel(&mut rng, &constraints).unwrap_err(),
            GenerationError::InvalidDateRange { min, max }
        );
    }

    #[test]
    fn rejects_date_range_outside_pesel_encodable_years() {
        let min = DateOfBirth::new(1700, 1, 1).unwrap();
        let max = DateOfBirth::new(1990, 1, 1).unwrap();
        let constraints = PeselConstraints {
            gender: None,
            date_range: Some((min, max)),
        };
        let mut rng = StdRng::seed_from_u64(0);

        assert_eq!(
            generate_pesel(&mut rng, &constraints).unwrap_err(),
            GenerationError::DateRangeOutsidePeselEncodableYears { min, max }
        );
    }

    #[test]
    fn handles_century_boundary_range_without_encoding_errors() {
        let min = DateOfBirth::new(1999, 12, 1).unwrap();
        let max = DateOfBirth::new(2000, 1, 31).unwrap();
        let constraints = PeselConstraints {
            gender: None,
            date_range: Some((min, max)),
        };
        let mut rng = StdRng::seed_from_u64(99);

        for _ in 0..200 {
            let generated = generate_pesel(&mut rng, &constraints).unwrap();
            let dob = generated.date_of_birth();
            assert!(dob >= min && dob <= max);
            assert_eq!(
                Pesel::parse(&generated.to_string())
                    .unwrap()
                    .date_of_birth(),
                dob
            );
        }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    proptest! {
        #[test]
        fn generated_pesels_always_satisfy_their_constraints(
            seed in any::<u64>(),
            year_a in 1900i32..2100,
            year_b in 1900i32..2100,
            gender_is_male in any::<bool>(),
        ) {
            let (min_year, max_year) = if year_a <= year_b { (year_a, year_b) } else { (year_b, year_a) };
            let min = DateOfBirth::new(min_year, 1, 1).unwrap();
            let max = DateOfBirth::new(max_year, 12, 31).unwrap();
            let gender = if gender_is_male { Gender::Male } else { Gender::Female };

            let constraints = PeselConstraints { gender: Some(gender), date_range: Some((min, max)) };
            let mut rng = StdRng::seed_from_u64(seed);

            let generated = generate_pesel(&mut rng, &constraints).unwrap();

            prop_assert_eq!(generated.gender(), gender);
            let dob = generated.date_of_birth();
            prop_assert!(dob >= min && dob <= max);

            let reparsed = Pesel::parse(&generated.to_string()).unwrap();
            prop_assert_eq!(reparsed, generated);
        }
    }
}
