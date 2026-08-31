use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhoneNumber([u8; 9]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhoneNumberError {
    WrongLength { actual: usize },
    NonDigitCharacter { character: char },
    ReservedPrefix { prefix: String },
}

impl fmt::Display for PhoneNumberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PhoneNumberError::WrongLength { actual } => {
                write!(f, "phone number must be exactly 9 digits, got {actual}")
            }
            PhoneNumberError::NonDigitCharacter { character } => {
                write!(
                    f,
                    "phone number must contain only digits, fount '{character}'"
                )
            }
            PhoneNumberError::ReservedPrefix { prefix } => {
                write!(
                    f,
                    "'{prefix}' is a reserved prefix (emergency, service, or premium-rate) and cannot be used for a synthetic mobile number"
                )
            }
        }
    }
}

impl std::error::Error for PhoneNumberError {}

const MOBILE_PREFIXES: &[&str] = &[
    "45", "50", "51", "53", "57", "60", "66", "69", "72", "73", "78", "79", "88",
];

const RESERVED_PREFIXES: &[&str] = &["70", "80", "81", "99", "11"];

impl PhoneNumber {
    pub fn from_digits(digits: [u8; 9]) -> Result<Self, PhoneNumberError> {
        let prefix: String = digits[0..2].iter().map(|d| (b'0' + d) as char).collect();
        if RESERVED_PREFIXES.contains(&prefix.as_str()) {
            return Err(PhoneNumberError::ReservedPrefix { prefix });
        }
        Ok(PhoneNumber(digits))
    }

    pub fn parse(s: &str) -> Result<Self, PhoneNumberError> {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() != 9 {
            return Err(PhoneNumberError::WrongLength {
                actual: chars.len(),
            });
        }

        let mut digits = [0u8; 9];
        for (i, &c) in chars.iter().enumerate() {
            digits[i] = c
                .to_digit(10)
                .ok_or(PhoneNumberError::NonDigitCharacter { character: c })?
                as u8;
        }

        PhoneNumber::from_digits(digits)
    }

    pub fn as_str(&self) -> String {
        let digits: String = self.0.iter().map(|d| (b'0' + d) as char).collect();
        format!("+48 {} {} {}", &digits[0..3], &digits[3..6], &digits[6..9])
    }
}

impl fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub fn generate_phone_number(rng: &mut impl rand::RngExt) -> PhoneNumber {
    loop {
        let prefix = MOBILE_PREFIXES[rng.random_range(0..MOBILE_PREFIXES.len())];
        let mut digits = [0u8; 9];
        digits[0] = prefix.as_bytes()[0] - b'0';
        digits[1] = prefix.as_bytes()[1] - b'0';
        for slot in digits.iter_mut().skip(2) {
            *slot = rng.random_range(0..=9);
        }

        if let Ok(number) = PhoneNumber::from_digits(digits) {
            return number;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_digits_formats_with_plus_48_and_grouping() {
        let number = PhoneNumber::from_digits([5, 0, 1, 2, 3, 4, 5, 6, 7]).unwrap();
        assert_eq!(number.to_string(), "+48 501 234 567");
    }

    #[test]
    fn rejects_reserved_prefix() {
        let err = PhoneNumber::from_digits([9, 9, 1, 2, 3, 4, 5, 6, 7]).unwrap_err();
        assert_eq!(
            err,
            PhoneNumberError::ReservedPrefix {
                prefix: "99".to_string()
            }
        );
    }

    #[test]
    fn parse_rejects_wrong_length() {
        assert_eq!(
            PhoneNumber::parse("12345").unwrap_err(),
            PhoneNumberError::WrongLength { actual: 5 }
        );
    }

    #[test]
    fn parse_rejects_non_digit_character() {
        let err = PhoneNumber::parse("50123456X").unwrap_err();
        assert_eq!(err, PhoneNumberError::NonDigitCharacter { character: 'X' });
    }

    #[test]
    fn parse_round_trips_through_display_digits() {
        let number = PhoneNumber::from_digits([5, 1, 9, 8, 7, 6, 5, 4, 3]).unwrap();
        let digits_only: String = number
            .to_string()
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();
        let reparsed = PhoneNumber::parse(&digits_only[2..]).unwrap();
        assert_eq!(reparsed, number);
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
            .map(|_| generate_phone_number(&mut rng_a).to_string())
            .collect();
        let seq_b: Vec<String> = (0..50)
            .map(|_| generate_phone_number(&mut rng_b).to_string())
            .collect();

        assert_eq!(seq_a, seq_b);
    }

    #[test]
    fn generated_numbers_never_use_a_reserved_prefix() {
        let mut rng = StdRng::seed_from_u64(3);

        for _ in 0..1000 {
            let number = generate_phone_number(&mut rng).to_string();
            let digits: String = number.chars().filter(|c| c.is_ascii_digit()).collect();
            let prefix = &digits[2..4];
            assert!(
                !RESERVED_PREFIXES.contains(&prefix),
                "{number} uses reserved prefix {prefix}"
            );
        }
    }

    #[test]
    fn generated_numbers_always_round_trip_through_parse() {
        let mut rng = StdRng::seed_from_u64(8);

        for _ in 0..500 {
            let generated = generate_phone_number(&mut rng);
            let digits: String = generated
                .to_string()
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect();
            let reparsed = PhoneNumber::parse(&digits[2..]).unwrap();
            assert_eq!(reparsed, generated);
        }
    }
}
