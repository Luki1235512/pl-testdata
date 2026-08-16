use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nip([u8; 10]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NipError {
    WrongLength { actual: usize },
    NonDigitCharacter { character: char },
    ChecksumMismatch { expected: u8, actual: u8 },
    UnencodableDigits { first_nine: [u8; 9] },
}

impl fmt::Display for NipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NipError::WrongLength { actual } => {
                write!(f, "NIP must be exactly 10 digits, got {actual}")
            }
            NipError::NonDigitCharacter { character } => {
                write!(f, "NIP must contain only digits, found '{character}'")
            }
            NipError::ChecksumMismatch { expected, actual } => {
                write!(f, "checksum mismatch: expected {expected}, got {actual}")
            }
            NipError::UnencodableDigits { first_nine } => {
                let s: String = first_nine.iter().map(|d| (b'0' + d) as char).collect();
                write!(
                    f,
                    "prefix {s} has no valid NIP checksum digit (weighted sum is congruent to 10 mod 11)"
                )
            }
        }
    }
}

impl std::error::Error for NipError {}

const NIP_WEIGHTS: [u32; 9] = [6, 5, 7, 2, 3, 4, 5, 6, 7];

fn checksum_digit(first_nine: &[u8; 9]) -> Option<u8> {
    let sum: u32 = first_nine
        .iter()
        .zip(NIP_WEIGHTS.iter())
        .map(|(&digit, &weight)| digit as u32 * weight)
        .sum();
    let remainder = sum % 11;
    if remainder == 10 {
        None
    } else {
        Some(remainder as u8)
    }
}

impl Nip {
    pub fn from_digits(first_nine: [u8; 9]) -> Result<Self, NipError> {
        let checksum =
            checksum_digit(&first_nine).ok_or(NipError::UnencodableDigits { first_nine })?;

        let mut digits = [0u8; 10];
        digits[0..9].copy_from_slice(&first_nine);
        digits[9] = checksum;

        Ok(Nip(digits))
    }

    pub fn parse(s: &str) -> Result<Self, NipError> {
        let char_count = s.chars().count();
        if char_count != 10 {
            return Err(NipError::WrongLength { actual: char_count });
        }

        let mut digits = [0u8; 10];
        for (i, c) in s.chars().enumerate() {
            let d = c
                .to_digit(10)
                .ok_or(NipError::NonDigitCharacter { character: c })?;
            digits[i] = d as u8;
        }

        let first_nine: [u8; 9] = digits[0..9].try_into().unwrap();
        let expected =
            checksum_digit(&first_nine).ok_or(NipError::UnencodableDigits { first_nine })?;

        if expected != digits[9] {
            return Err(NipError::ChecksumMismatch {
                expected,
                actual: digits[9],
            });
        }

        Ok(Nip(digits))
    }

    pub fn as_str(&self) -> String {
        self.0.iter().map(|d| (b'0' + d) as char).collect()
    }
}

impl fmt::Display for Nip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub fn generate_nip(rng: &mut impl rand::RngExt) -> Nip {
    loop {
        let digits: [u8; 9] = std::array::from_fn(|_| rng.random_range(0..=9));
        if let Ok(nip) = Nip::from_digits(digits) {
            return nip;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_is_none_for_a_known_unencodable_prefix() {
        let unencodable = (0u32..2000)
            .map(|n| {
                let s = format!("{n:09}");
                let digits: [u8; 9] = std::array::from_fn(|i| s.as_bytes()[i] - b'0');
                digits
            })
            .find(|digits| checksum_digit(digits).is_none())
            .expect("an unencodable prefix exists within the first 2000 candidates");

        assert_eq!(checksum_digit(&unencodable), None);
    }

    #[test]
    fn checksum_is_some_for_a_known_encodable_prefix() {
        let prefix = [1, 2, 3, 4, 5, 6, 3, 2, 1];
        assert_eq!(checksum_digit(&prefix), Some(8));
    }

    #[test]
    fn from_digits_roundtrips_through_parse() {
        let prefix = [1, 2, 3, 4, 5, 6, 3, 2, 1];
        let nip = Nip::from_digits(prefix).unwrap();
        assert_eq!(nip.to_string(), "1234563218");

        let reparsed = Nip::parse(&nip.to_string()).unwrap();
        assert_eq!(reparsed, nip);
    }

    #[test]
    fn from_digits_rejects_an_unencodable_prefix() {
        let unencodable = (0u32..1_000_000_000)
            .map(|n| {
                let s = format!("{n:09}");
                let digits: [u8; 9] = std::array::from_fn(|i| s.as_bytes()[i] - b'0');
                digits
            })
            .find(|digits| checksum_digit(digits).is_none())
            .unwrap();

        let err = Nip::from_digits(unencodable).unwrap_err();
        assert_eq!(
            err,
            NipError::UnencodableDigits {
                first_nine: unencodable
            }
        );
    }

    #[test]
    fn rejects_wrong_length() {
        assert_eq!(
            Nip::parse("123").unwrap_err(),
            NipError::WrongLength { actual: 3 }
        );
    }

    #[test]
    fn rejects_non_digit_characters() {
        let err = Nip::parse("123456321X").unwrap_err();
        assert_eq!(err, NipError::NonDigitCharacter { character: 'X' });
    }

    #[test]
    fn rejects_bad_checksum() {
        let err = Nip::parse("1234563219").unwrap_err();
        assert_eq!(
            err,
            NipError::ChecksumMismatch {
                expected: 8,
                actual: 9
            }
        );
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
            .map(|_| generate_nip(&mut rng_a).to_string())
            .collect();
        let seq_b: Vec<String> = (0..50)
            .map(|_| generate_nip(&mut rng_b).to_string())
            .collect();

        assert_eq!(seq_a, seq_b);
    }

    #[test]
    fn different_seeds_produce_different_sequences() {
        let mut rng_a = StdRng::seed_from_u64(1);
        let mut rng_b = StdRng::seed_from_u64(2);

        let seq_a: Vec<String> = (0..50)
            .map(|_| generate_nip(&mut rng_a).to_string())
            .collect();
        let seq_b: Vec<String> = (0..50)
            .map(|_| generate_nip(&mut rng_b).to_string())
            .collect();

        assert_ne!(seq_a, seq_b);
    }

    #[test]
    fn generated_nips_always_round_trip_through_parse() {
        let mut rng = StdRng::seed_from_u64(7);

        for _ in 0..500 {
            let generated = generate_nip(&mut rng);
            let reparsed = Nip::parse(&generated.to_string())
                .expect("a generated Nip must always be independently parseable");
            assert_eq!(reparsed, generated);
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
        fn generated_nips_always_have_a_valid_checksum(seed in any::<u64>()) {
            let mut rng = StdRng::seed_from_u64(seed);
            let nip = generate_nip(&mut rng);

            let s = nip.to_string();
            let digits: [u8; 10] = std::array::from_fn(|i| s.as_bytes()[i] - b'0');
            let first_nine: [u8; 9] = digits[0..9].try_into().unwrap();

            let expected = checksum_digit(&first_nine).expect("generator never returns an unencodable prefix");
            prop_assert_eq!(digits[9], expected);

            let reparsed = Nip::parse(&s).unwrap();
            prop_assert_eq!(reparsed, nip);
        }
    }
}
