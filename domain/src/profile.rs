use crate::address::{self, Address};
use crate::email::{self, EmailAddress};
use crate::generate::GenerationError;
use crate::nip::{self, Nip};
use crate::person::{self, Person, PersonConstraints};
use crate::phone::{self, PhoneNumber};
use rand::RngExt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestProfile {
    pub person: Person,
    pub nip: Nip,
    pub address: Address,
    pub phone: PhoneNumber,
    pub email: EmailAddress,
}

pub fn generate_test_profile(
    rng: &mut impl RngExt,
    constraints: &PersonConstraints,
) -> Result<TestProfile, GenerationError> {
    let person = person::generate_person(rng, constraints)?;
    let nip = nip::generate_nip(rng);
    let address = address::generate_address(rng);
    let phone = phone::generate_phone_number(rng);
    let email = email::generate_email(rng, &person.first_name, &person.last_name);

    Ok(TestProfile {
        person,
        nip,
        address,
        phone,
        email,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn same_seed_produces_identical_profiles() {
        let constraints = PersonConstraints::default();

        let mut rng_a = StdRng::seed_from_u64(42);
        let mut rng_b = StdRng::seed_from_u64(42);

        let profiles_a: Vec<TestProfile> = (0..20)
            .map(|_| generate_test_profile(&mut rng_a, &constraints).unwrap())
            .collect();
        let profiles_b: Vec<TestProfile> = (0..20)
            .map(|_| generate_test_profile(&mut rng_b, &constraints).unwrap())
            .collect();

        assert_eq!(profiles_a, profiles_b);
    }

    #[test]
    fn propagates_generation_errors_from_the_underlying_person_constraints() {
        use crate::DateOfBirth;

        let min = DateOfBirth::new(2000, 1, 1).unwrap();
        let max = DateOfBirth::new(1990, 1, 1).unwrap();
        let constraints = PersonConstraints {
            gender: None,
            date_range: Some((min, max)),
        };
        let mut rng = StdRng::seed_from_u64(0);

        assert_eq!(
            generate_test_profile(&mut rng, &constraints).unwrap_err(),
            GenerationError::InvalidDateRange { min, max }
        );
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::{DateOfBirth, Gender};
    use proptest::prelude::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    proptest! {
        #[test]
        fn bundled_profile_still_satisfies_person_constraints(
            seed in any::<u64>(),
            year_a in 1900i32..2100,
            year_b in 1900i32..2100,
            gender_is_male in any::<bool>(),
        ) {
            let (min_year, max_year) = if year_a <= year_b { (year_a, year_b) } else { (year_b, year_a) };
            let min = DateOfBirth::new(min_year, 1, 1).unwrap();
            let max = DateOfBirth::new(max_year, 12, 31).unwrap();
            let gender = if gender_is_male { Gender::Male } else { Gender::Female };

            let constraints = PersonConstraints { gender: Some(gender), date_range: Some((min, max)) };
            let mut rng = StdRng::seed_from_u64(seed);

            let profile = generate_test_profile(&mut rng, &constraints).unwrap();

            prop_assert_eq!(profile.person.gender, gender);
            prop_assert!(profile.person.date_of_birth >= min && profile.person.date_of_birth <= max);
            prop_assert_eq!(profile.person.gender, profile.person.pesel.gender());
        }
    }
}
