use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostalCode([u8; 5]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostalCodeError {
    WrongLength { actual: usize },
    NonDigitCharacter { character: char },
    MissingSeparator,
}

impl fmt::Display for PostalCodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PostalCodeError::WrongLength { actual } => {
                write!(
                    f,
                    "postal code must be exactly 6 characters (NN-NNN), got {actual}"
                )
            }
            PostalCodeError::NonDigitCharacter { character } => {
                write!(
                    f,
                    "postal code must contain only digits and a dash, found '{character}'"
                )
            }
            PostalCodeError::MissingSeparator => {
                write!(
                    f,
                    "postal code must have a dash between the 2nd and 3rd digit, e.g '00-001'"
                )
            }
        }
    }
}

impl std::error::Error for PostalCodeError {}

impl PostalCode {
    pub fn from_digits(digits: [u8; 5]) -> Self {
        PostalCode(digits)
    }

    pub fn parse(s: &str) -> Result<Self, PostalCodeError> {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() != 6 {
            return Err(PostalCodeError::WrongLength {
                actual: chars.len(),
            });
        }
        if chars[2] != '-' {
            return Err(PostalCodeError::MissingSeparator);
        }

        let mut digits = [0u8; 5];
        let mut digit_index = 0;
        for (i, &c) in chars.iter().enumerate() {
            if i == 2 {
                continue;
            }
            let d = c
                .to_digit(10)
                .ok_or(PostalCodeError::NonDigitCharacter { character: c })?;
            digits[digit_index] = d as u8;
            digit_index += 1;
        }

        Ok(PostalCode(digits))
    }

    pub fn as_str(&self) -> String {
        let digits: String = self.0.iter().map(|d| (b'0' + d) as char).collect();
        format!("{}-{}", &digits[0..2], &digits[2..5])
    }
}

impl fmt::Display for PostalCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
    pub city: String,
    pub postal_code: PostalCode,
}

const POLISH_CITIES: &[&str] = &[
    "Warszawa",
    "Kraków",
    "Łódź",
    "Wrocław",
    "Poznań",
    "Gdańsk",
    "Olsztyn",
    "Szczecin",
    "Bydgoszcz",
    "Lublin",
    "Białystok",
    "Katowice",
    "Gdynia",
    "Częstochowa",
    "Radom",
    "Sosnowiec",
    "Toruń",
    "Kielce",
    "Rzeszów",
    "Gliwice",
    "Zabrze",
];

pub fn generate_address(rng: &mut impl rand::RngExt) -> Address {
    let city = POLISH_CITIES[rng.random_range(0..POLISH_CITIES.len())].to_string();
    let digits: [u8; 5] = std::array::from_fn(|_| rng.random_range(0..=9));
    Address {
        city,
        postal_code: PostalCode::from_digits(digits),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_digits_displays_in_nn_nnn_form() {
        let code = PostalCode::from_digits([3, 0, 0, 0, 1]);
        assert_eq!(code.to_string(), "30-001");
    }

    #[test]
    fn parse_round_trips_through_display() {
        let code = PostalCode::parse("00-001").unwrap();
        assert_eq!(code.to_string(), "00-001");
    }

    #[test]
    fn rejects_wrong_length() {
        assert_eq!(
            PostalCode::parse("123").unwrap_err(),
            PostalCodeError::WrongLength { actual: 3 }
        );
    }

    #[test]
    fn rejects_missing_separator() {
        assert_eq!(
            PostalCode::parse("001234").unwrap_err(),
            PostalCodeError::MissingSeparator
        )
    }

    #[test]
    fn rejects_non_digit_character() {
        let err = PostalCode::parse("AB-001").unwrap_err();
        assert_eq!(err, PostalCodeError::NonDigitCharacter { character: 'A' });
    }
}

#[cfg(test)]
mod generate_tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn same_seed_produces_identical_sequences() {
        let mut rng_a = StdRng::seed_from_u64(42);
        let mut rng_b = StdRng::seed_from_u64(42);

        let seq_a: Vec<String> = (0..50)
            .map(|_| generate_address(&mut rng_a).postal_code.to_string())
            .collect();
        let seq_b: Vec<String> = (0..50)
            .map(|_| generate_address(&mut rng_b).postal_code.to_string())
            .collect();

        assert_eq!(seq_a, seq_b);
    }

    #[test]
    fn different_seeds_produce_different_sequences() {
        let mut rng_a = StdRng::seed_from_u64(1);
        let mut rng_b = StdRng::seed_from_u64(2);

        let seq_a: Vec<String> = (0..50)
            .map(|_| generate_address(&mut rng_a).postal_code.to_string())
            .collect();
        let seq_b: Vec<String> = (0..50)
            .map(|_| generate_address(&mut rng_b).postal_code.to_string())
            .collect();

        assert_ne!(seq_a, seq_b);
    }

    #[test]
    fn city_is_always_from_the_known_list() {
        let mut rng = StdRng::seed_from_u64(9);

        for _ in 0..500 {
            let address = generate_address(&mut rng);
            assert!(
                POLISH_CITIES.contains(&address.city.as_str()),
                "{} not in the known city list",
                address.city
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
        fn generated_postal_codes_always_round_trip_through_parse(seed in any::<u64>()) {
            let mut rng = StdRng::seed_from_u64(seed);
            let address = generate_address(&mut rng);

            let s = address.postal_code.to_string();
            let reparsed = PostalCode::parse(&s).unwrap();
            prop_assert_eq!(reparsed, address.postal_code);
        }
    }
}
