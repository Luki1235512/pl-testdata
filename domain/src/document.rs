use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdDocumentNumber {
    letters: [u8; 3],
    digits: [u8; 6],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdDocumentNumberError {
    WrongLength { actual: usize },
    NonLetterCharacter { character: char },
    NonDigitCharacter { character: char },
    ChecksumMismatch { expected: u8, actual: u8 },
}

impl fmt::Display for IdDocumentNumberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdDocumentNumberError::WrongLength { actual } => {
                write!(
                    f,
                    "ID document number must be exactly 9 characters (3 letters + 6 digits), got {actual}"
                )
            }
            IdDocumentNumberError::NonLetterCharacter { character } => {
                write!(
                    f,
                    "the first 3 characters must be uppercase letters A-Z, found '{character}'"
                )
            }
            IdDocumentNumberError::NonDigitCharacter { character } => {
                write!(
                    f,
                    "the last 6 characters must be digits, found '{character}'"
                )
            }
            IdDocumentNumberError::ChecksumMismatch { expected, actual } => {
                write!(f, "checksum mismatch: expected {expected}, got {actual}")
            }
        }
    }
}

impl std::error::Error for IdDocumentNumberError {}

const LETTER_WEIGHTS: [u32; 3] = [7, 3, 1];
const REST_DIGIT_WEIGHTS: [u32; 5] = [7, 3, 1, 7, 3];

fn checksum_digit(letters: &[u8; 3], rest_digits: &[u8; 5]) -> u8 {
    let letter_sum: u32 = letters
        .iter()
        .zip(LETTER_WEIGHTS.iter())
        .map(|(&l, &w)| (l as u32 + 10) * w)
        .sum();

    let digit_sum: u32 = rest_digits
        .iter()
        .zip(REST_DIGIT_WEIGHTS.iter())
        .map(|(&d, &w)| (d as u32) * w)
        .sum();

    ((letter_sum + digit_sum) % 10) as u8
}

impl IdDocumentNumber {
    pub fn from_parts(letters: [u8; 3], rest_digits: [u8; 5]) -> Self {
        debug_assert!(letters.iter().all(|&l| l <= 25));
        debug_assert!(rest_digits.iter().all(|&d| d <= 9));

        let check = checksum_digit(&letters, &rest_digits);
        let mut digits = [0u8; 6];
        digits[0] = check;
        digits[1..6].copy_from_slice(&rest_digits);

        IdDocumentNumber { letters, digits }
    }

    pub fn parse(s: &str) -> Result<Self, IdDocumentNumberError> {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() != 9 {
            return Err(IdDocumentNumberError::WrongLength {
                actual: chars.len(),
            });
        }

        let mut letters = [0u8; 3];
        for (i, &c) in chars[0..3].iter().enumerate() {
            if !c.is_ascii_uppercase() {
                return Err(IdDocumentNumberError::NonLetterCharacter { character: c });
            }
            letters[i] = c as u8 - b'A';
        }

        let mut digits = [0u8; 6];
        for (i, &c) in chars[3..9].iter().enumerate() {
            let d = c
                .to_digit(10)
                .ok_or(IdDocumentNumberError::NonDigitCharacter { character: c })?;
            digits[i] = d as u8;
        }

        let rest: [u8; 5] = digits[1..6].try_into().unwrap();
        let expected = checksum_digit(&letters, &rest);
        if expected != digits[0] {
            return Err(IdDocumentNumberError::ChecksumMismatch {
                expected,
                actual: digits[0],
            });
        }

        Ok(IdDocumentNumber { letters, digits })
    }

    pub fn as_str(&self) -> String {
        let letter_part: String = self.letters.iter().map(|&l| (b'A' + l) as char).collect();
        let digit_part: String = self.digits.iter().map(|&d| (b'0' + d) as char).collect();
        format!("{letter_part}{digit_part}")
    }
}

impl fmt::Display for IdDocumentNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub fn generate_id_document_number(rng: &mut impl rand::RngExt) -> IdDocumentNumber {
    let letters: [u8; 3] = std::array::from_fn(|_| rng.random_range(0..=25));
    let rest_digits: [u8; 5] = std::array::from_fn(|_| rng.random_range(0..=9));
    IdDocumentNumber::from_parts(letters, rest_digits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_parts_roundtrips_through_parse() {
        let letters = [0u8, 1, 2]; // "ABC"
        let rest = [1, 2, 3, 4, 5];
        let built = IdDocumentNumber::from_parts(letters, rest);

        let reparsed = IdDocumentNumber::parse(&built.to_string()).unwrap();
        assert_eq!(reparsed, built);
    }

    #[test]
    fn as_str_formats_letters_uppercase_and_digits_after() {
        let built = IdDocumentNumber::from_parts([0, 1, 2], [1, 2, 3, 4, 5]);
        let s = built.to_string();
        assert_eq!(s.len(), 9);
        assert!(s[0..3].chars().all(|c| c.is_ascii_uppercase()));
        assert!(s[3..9].chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn rejects_wrong_length() {
        assert_eq!(
            IdDocumentNumber::parse("ABC123").unwrap_err(),
            IdDocumentNumberError::WrongLength { actual: 6 }
        );
    }

    #[test]
    fn rejects_lowercase_letter_section() {
        // checksum will not even be reached; letter validity is checked first
        let err = IdDocumentNumber::parse("abc123456").unwrap_err();
        assert_eq!(
            err,
            IdDocumentNumberError::NonLetterCharacter { character: 'a' }
        );
    }

    #[test]
    fn rejects_non_digit_in_digit_section() {
        let valid = IdDocumentNumber::from_parts([0, 1, 2], [1, 2, 3, 4, 5]).to_string();
        let mut bytes = valid.into_bytes();
        bytes[5] = b'X'; // corrupt a digit position
        let corrupted = String::from_utf8(bytes).unwrap();

        let err = IdDocumentNumber::parse(&corrupted).unwrap_err();
        assert_eq!(
            err,
            IdDocumentNumberError::NonDigitCharacter { character: 'X' }
        );
    }

    #[test]
    fn rejects_bad_checksum() {
        let valid = IdDocumentNumber::from_parts([0, 1, 2], [1, 2, 3, 4, 5]).to_string();
        let mut bytes = valid.into_bytes();
        let last = bytes.len() - 1;
        bytes[last] = if bytes[last] == b'9' {
            b'8'
        } else {
            bytes[last] + 1
        };
        let corrupted = String::from_utf8(bytes).unwrap();

        assert!(matches!(
            IdDocumentNumber::parse(&corrupted),
            Err(IdDocumentNumberError::ChecksumMismatch { .. })
        ));
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
            .map(|_| generate_id_document_number(&mut rng_a).to_string())
            .collect();
        let seq_b: Vec<String> = (0..50)
            .map(|_| generate_id_document_number(&mut rng_b).to_string())
            .collect();

        assert_eq!(seq_a, seq_b);
    }

    #[test]
    fn different_seeds_produce_different_sequences() {
        let mut rng_a = StdRng::seed_from_u64(1);
        let mut rng_b = StdRng::seed_from_u64(2);

        let seq_a: Vec<String> = (0..50)
            .map(|_| generate_id_document_number(&mut rng_a).to_string())
            .collect();
        let seq_b: Vec<String> = (0..50)
            .map(|_| generate_id_document_number(&mut rng_b).to_string())
            .collect();

        assert_ne!(seq_a, seq_b);
    }

    #[test]
    fn generated_numbers_always_round_trip_through_parse() {
        let mut rng = StdRng::seed_from_u64(7);

        for _ in 0..500 {
            let generated = generate_id_document_number(&mut rng);
            let reparsed = IdDocumentNumber::parse(&generated.to_string())
                .expect("a generated IdDocumentNumber must always be independently parseable");
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
        fn generated_document_numbers_always_round_trip(seed in any::<u64>()) {
            let mut rng = StdRng::seed_from_u64(seed);
            let generated = generate_id_document_number(&mut rng);

            let s = generated.to_string();
            let reparsed = IdDocumentNumber::parse(&s).unwrap();
            prop_assert_eq!(reparsed, generated);
        }
    }
}
