use crate::generate::{self, GenerationError, PeselConstraints};
use crate::{DateOfBirth, Gender, Pesel};
use rand::RngExt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
    pub gender: Gender,
    pub date_of_birth: DateOfBirth,
    pub pesel: Pesel,
}

#[derive(Debug, Clone, Default)]
pub struct PersonConstraints {
    pub gender: Option<Gender>,
    pub date_range: Option<(DateOfBirth, DateOfBirth)>,
}

pub fn generate_person(
    rng: &mut impl RngExt,
    constraints: &PersonConstraints,
) -> Result<Person, GenerationError> {
    let pesel_constraints = PeselConstraints {
        gender: constraints.gender,
        date_range: constraints.date_range,
    };
    let pesel = generate::generate_pesel(rng, &pesel_constraints)?;

    let gender = pesel.gender();
    let date_of_birth = pesel.date_of_birth();
    let first_name = pick_first_name(rng, gender).to_string();
    let last_name = pick_last_name(rng, gender);

    Ok(Person {
        first_name,
        last_name,
        gender,
        date_of_birth,
        pesel,
    })
}

const MALE_FIRST_NAMES: &[&str] = &[
    "Jan",
    "Piotr",
    "Krzysztof",
    "Andrzej",
    "Tomasz",
    "Paweł",
    "Michał",
    "Marcin",
    "Grzegorz",
    "Tadeusz",
    "Jerzy",
    "Dariusz",
    "Marek",
    "Łukasz",
    "Wojciech",
    "Adam",
    "Kamil",
    "Rafał",
    "Stanisław",
    "Zbigniew",
];

const FEMALE_FIRST_NAMES: &[&str] = &[
    "Anna",
    "Maria",
    "Katarzyna",
    "Małgorzata",
    "Agnieszka",
    "Barbara",
    "Ewa",
    "Elżbieta",
    "Krystyna",
    "Magdalena",
    "Joanna",
    "Danuta",
    "Teresa",
    "Zofia",
    "Beata",
    "Monika",
    "Alicja",
    "Halina",
    "Irena",
    "Justyna",
];

const INVARIANT_SURNAMES: &[&str] = &[
    "Nowak",
    "Kowalczyk",
    "Wójcik",
    "Zając",
    "Woźniak",
    "Kaczmarek",
];

const SURNAME_STEMS: &[&str] = &[
    "Kowal", "Wiśniew", "Zieliń", "Wilcz", "Krawcz", "Wieczor", "Głowa", "Baran", "Sikor",
    "Ostrow", "Kamiń", "Lewandow", "Szymań", "Piotrow",
];

fn pick_first_name(rng: &mut impl RngExt, gender: Gender) -> &'static str {
    let names = match gender {
        Gender::Male => MALE_FIRST_NAMES,
        Gender::Female => FEMALE_FIRST_NAMES,
    };
    names[rng.random_range(0..names.len())]
}

fn pick_last_name(rng: &mut impl RngExt, gender: Gender) -> String {
    if rng.random_bool(0.5) {
        INVARIANT_SURNAMES[rng.random_range(0..INVARIANT_SURNAMES.len())].to_string()
    } else {
        let stem = SURNAME_STEMS[rng.random_range(0..SURNAME_STEMS.len())];
        let suffix = match gender {
            Gender::Male => "ski",
            Gender::Female => "ska",
        };
        format!("{stem}{suffix}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn narrow_constraints() -> PersonConstraints {
        PersonConstraints {
            gender: None,
            date_range: Some((
                DateOfBirth::new(1970, 1, 1).unwrap(),
                DateOfBirth::new(2005, 12, 31).unwrap(),
            )),
        }
    }

    #[test]
    fn person_gender_always_matches_its_pesel() {
        let constraints = narrow_constraints();
        let mut rng = StdRng::seed_from_u64(5);

        for _ in 0..200 {
            let person = generate_person(&mut rng, &constraints).unwrap();
            assert_eq!(person.gender, person.pesel.gender());
        }
    }

    #[test]
    fn person_date_of_birth_always_matches_its_pesel() {
        let constraints = narrow_constraints();
        let mut rng = StdRng::seed_from_u64(6);

        for _ in 0..200 {
            let person = generate_person(&mut rng, &constraints).unwrap();
            assert_eq!(person.date_of_birth, person.pesel.date_of_birth());
        }
    }

    #[test]
    fn first_name_is_drawn_from_the_matching_gender_list() {
        let constraints = narrow_constraints();
        let mut rng = StdRng::seed_from_u64(9);

        for _ in 0..200 {
            let person = generate_person(&mut rng, &constraints).unwrap();
            let expected_list = match person.gender {
                Gender::Male => MALE_FIRST_NAMES,
                Gender::Female => FEMALE_FIRST_NAMES,
            };
            assert!(
                expected_list.contains(&person.first_name.as_str()),
                "{} not in the {:?} name list",
                person.first_name,
                person.gender
            );
        }
    }

    #[test]
    fn inflected_surnames_use_the_correct_gender_suffix() {
        let constraints = narrow_constraints();
        let mut rng = StdRng::seed_from_u64(13);

        for _ in 0..200 {
            let person = generate_person(&mut rng, &constraints).unwrap();
            if INVARIANT_SURNAMES.contains(&person.last_name.as_str()) {
                continue;
            }
            match person.gender {
                Gender::Male => assert!(
                    person.last_name.ends_with("ski"),
                    "{} should end in 'ski'",
                    person.last_name
                ),
                Gender::Female => assert!(
                    person.last_name.ends_with("ska"),
                    "{} should end in 'ska'",
                    person.last_name
                ),
            }
        }
    }

    #[test]
    fn same_seed_produces_identical_people() {
        let constraints = narrow_constraints();

        let mut rng_a = StdRng::seed_from_u64(21);
        let mut rng_b = StdRng::seed_from_u64(21);

        let people_a: Vec<Person> = (0..20)
            .map(|_| generate_person(&mut rng_a, &constraints).unwrap())
            .collect();
        let people_b: Vec<Person> = (0..20)
            .map(|_| generate_person(&mut rng_b, &constraints).unwrap())
            .collect();

        assert_eq!(people_a, people_b);
    }

    #[test]
    fn respects_gender_constraint() {
        let constraints = PersonConstraints {
            gender: Some(Gender::Female),
            date_range: Some((
                DateOfBirth::new(1990, 1, 1).unwrap(),
                DateOfBirth::new(1990, 12, 31).unwrap(),
            )),
        };
        let mut rng = StdRng::seed_from_u64(17);

        for _ in 0..50 {
            let person = generate_person(&mut rng, &constraints).unwrap();
            assert_eq!(person.gender, Gender::Female);
            assert!(FEMALE_FIRST_NAMES.contains(&person.first_name.as_str()));
        }
    }

    #[test]
    fn respects_date_range_constraint() {
        let min = DateOfBirth::new(1985, 6, 1).unwrap();
        let max = DateOfBirth::new(1985, 6, 30).unwrap();
        let constraints = PersonConstraints {
            gender: None,
            date_range: Some((min, max)),
        };
        let mut rng = StdRng::seed_from_u64(23);

        for _ in 0..50 {
            let person = generate_person(&mut rng, &constraints).unwrap();
            assert!(person.date_of_birth >= min && person.date_of_birth <= max);
        }
    }

    #[test]
    fn propagates_generation_errors_from_invalid_constraints() {
        let min = DateOfBirth::new(2000, 1, 1).unwrap();
        let max = DateOfBirth::new(1990, 1, 1).unwrap();
        let constraints = PersonConstraints {
            gender: None,
            date_range: Some((min, max)),
        };
        let mut rng = StdRng::seed_from_u64(0);

        assert_eq!(
            generate_person(&mut rng, &constraints).unwrap_err(),
            GenerationError::InvalidDateRange { min, max }
        );
    }

    #[test]
    fn ski_surnames_are_never_assigned_unchanged_to_a_female_person() {
        let constraints = PersonConstraints {
            gender: Some(Gender::Female),
            date_range: None,
        };
        let mut rng = StdRng::seed_from_u64(31);

        for _ in 0..500 {
            let person = generate_person(&mut rng, &constraints).unwrap();
            assert!(
                !person.last_name.ends_with("ski"),
                "female person got a masculine -ski surname: {}",
                person.last_name
            );
        }
    }
}
