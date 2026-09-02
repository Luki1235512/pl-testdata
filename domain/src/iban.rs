use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Iban {
    check_digits: [u8; 2],
    bank_branch: [u8; 8],
    account: [u8; 16],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IbanError {
    WrongLength { actual: usize },
    MissingCountryCode { found: String },
    NonDigitCharacter { character: char },
    ChecksumMismatch { expected: [u8; 2], actual: [u8; 2] },
}

impl fmt::Display for IbanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IbanError::WrongLength { actual } => write!(
                f,
                "Polish IBAN must be exactly 28 characters (PL + 26 digits), got {actual}"
            ),
            IbanError::MissingCountryCode { found } => {
                write!(f, "IBAN must start with country code 'PL', found '{found}'")
            }
            IbanError::NonDigitCharacter { character } => {
                write!(
                    f,
                    "IBAN digit section must contain only digits, found '{character}'"
                )
            }
            IbanError::ChecksumMismatch { expected, actual } => write!(
                f,
                "checksum mismatch: expected {:02}, got {:02}",
                expected[0] * 10 + expected[1],
                actual[0] * 10 + actual[1]
            ),
        }
    }
}

impl std::error::Error for IbanError {}

fn mod97_fold(digits: impl Iterator<Item = u8>) -> u32 {
    digits.fold(0u32, |rem, d| (rem * 10 + d as u32) % 97)
}

fn letter_digits(c: char) -> [u8; 2] {
    let value = c as u8 - b'A' + 10;
    [value / 10, value % 10]
}

fn compute_check_digits(bank_branch: &[u8; 8], account: &[u8; 16]) -> [u8; 2] {
    let rearranged = bank_branch
        .iter()
        .copied()
        .chain(account.iter().copied())
        .chain(letter_digits('P'))
        .chain(letter_digits('L'))
        .chain([0, 0]);

    let remainder = mod97_fold(rearranged);
    let check = 98 - remainder;
    [(check / 10) as u8, (check % 10) as u8]
}

impl Iban {
    pub fn from_parts(bank_branch: [u8; 8], account: [u8; 16]) -> Self {
        debug_assert!(bank_branch.iter().all(|&d| d <= 9));
        debug_assert!(account.iter().all(|&d| d <= 9));

        let check_digits = compute_check_digits(&bank_branch, &account);
        Iban {
            check_digits,
            bank_branch,
            account,
        }
    }

    pub fn parse(s: &str) -> Result<Self, IbanError> {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() != 28 {
            return Err(IbanError::WrongLength {
                actual: chars.len(),
            });
        }

        let country: String = chars[0..2].iter().collect();
        if country != "PL" {
            return Err(IbanError::MissingCountryCode { found: country });
        }

        let mut digits = [0u8; 26];
        for (i, &c) in chars[2..28].iter().enumerate() {
            digits[i] =
                c.to_digit(10)
                    .ok_or(IbanError::NonDigitCharacter { character: c })? as u8;
        }

        let check_digits = [digits[0], digits[1]];
        let bank_branch: [u8; 8] = digits[2..10].try_into().unwrap();
        let account: [u8; 16] = digits[10..26].try_into().unwrap();

        let expected = compute_check_digits(&bank_branch, &account);
        if expected != check_digits {
            return Err(IbanError::ChecksumMismatch {
                expected,
                actual: check_digits,
            });
        }

        Ok(Iban {
            check_digits,
            bank_branch,
            account,
        })
    }

    pub fn as_str(&self) -> String {
        let check: String = self
            .check_digits
            .iter()
            .map(|d| (b'0' + d) as char)
            .collect();
        let bank_branch: String = self
            .bank_branch
            .iter()
            .map(|d| (b'0' + d) as char)
            .collect();
        let account: String = self.account.iter().map(|d| (b'0' + d) as char).collect();
        format!("PL{check}{bank_branch}{account}")
    }
}

impl fmt::Display for Iban {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub fn generate_iban(rng: &mut impl rand::RngExt) -> Iban {
    let bank_branch: [u8; 8] = std::array::from_fn(|_| rng.random_range(0..=9));
    let account: [u8; 16] = std::array::from_fn(|_| rng.random_range(0..=9));
    Iban::from_parts(bank_branch, account)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_parts_roundtrips_through_parse() {
        let bank_branch = [1, 0, 9, 0, 1, 0, 1, 4];
        let account = [0, 0, 0, 0, 0, 7, 1, 2, 1, 9, 8, 1, 2, 8, 7, 4];
        let built = Iban::from_parts(bank_branch, account);

        let reparsed = Iban::parse(&built.to_string()).unwrap();
        assert_eq!(reparsed, built);
    }

    #[test]
    fn as_str_starts_with_pl_and_is_28_characters() {
        let built = Iban::from_parts([1, 0, 9, 0, 1, 0, 1, 4], [0; 16]);
        let s = built.to_string();
        assert_eq!(s.len(), 28);
        assert!(s.starts_with("PL"));
        assert!(s[2..].chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn rejects_wrong_length() {
        assert_eq!(
            Iban::parse("PL1234").unwrap_err(),
            IbanError::WrongLength { actual: 6 }
        );
    }

    #[test]
    fn rejects_wrong_country_code() {
        let built = Iban::from_parts([1, 0, 9, 0, 1, 0, 1, 4], [0; 16]);
        let s = built.to_string();
        let swapped = format!("DE{}", &s[2..]);
        let err = Iban::parse(&swapped).unwrap_err();
        assert_eq!(
            err,
            IbanError::MissingCountryCode {
                found: "DE".to_string()
            }
        );
    }

    #[test]
    fn rejects_non_digit_character() {
        let built = Iban::from_parts([1, 0, 9, 0, 1, 0, 1, 4], [0; 16]);
        let mut s = built.to_string();
        s.replace_range(5..6, "X");
        let err = Iban::parse(&s).unwrap_err();
        assert_eq!(err, IbanError::NonDigitCharacter { character: 'X' });
    }

    #[test]
    fn rejects_bad_checksum() {
        let built = Iban::from_parts([1, 0, 9, 0, 1, 0, 1, 4], [0; 16]);
        let mut s = built.to_string();
        let corrupted_digit = if &s[2..3] == "9" { "8" } else { "9" };
        s.replace_range(2..3, corrupted_digit);

        assert!(matches!(
            Iban::parse(&s),
            Err(IbanError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn mod97_fold_matches_hand_computed_small_cases() {
        assert_eq!(mod97_fold([9, 8, 1, 2].into_iter()), 9812 % 97);
        assert_eq!(mod97_fold([1, 0, 0].into_iter()), 3);
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
            .map(|_| generate_iban(&mut rng_a).to_string())
            .collect();
        let seq_b: Vec<String> = (0..50)
            .map(|_| generate_iban(&mut rng_b).to_string())
            .collect();

        assert_eq!(seq_a, seq_b);
    }

    #[test]
    fn generated_ibans_always_round_trip_through_parse() {
        let mut rng = StdRng::seed_from_u64(7);

        for _ in 0..500 {
            let generated = generate_iban(&mut rng);
            let reparsed = Iban::parse(&generated.to_string())
                .expect("a generated Iban must always be independently parseable");
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
        fn generated_ibans_always_round_trip(seed in any::<u64>()) {
            let mut rng = StdRng::seed_from_u64(seed);
            let generated = generate_iban(&mut rng);

            let s = generated.to_string();
            let reparsed = Iban::parse(&s).unwrap();
            prop_assert_eq!(reparsed, generated);
        }
    }
}
