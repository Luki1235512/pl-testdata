use chrono::{Datelike, NaiveDate};
use std::fmt;

pub mod generate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
}

impl fmt::Display for Gender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Gender::Male => write!(f, "Male"),
            Gender::Female => write!(f, "Female"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateOfBirth(NaiveDate);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateOfBirthError {
    InvalidCalendarDate { year: i32, month: u32, day: u32 },
}

impl fmt::Display for DateOfBirthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DateOfBirthError::InvalidCalendarDate { year, month, day } => {
                write!(
                    f,
                    "{year:04}-{month:02}-{day:02} is not a valid calendar date"
                )
            }
        }
    }
}

impl std::error::Error for DateOfBirthError {}

impl DateOfBirth {
    pub fn new(year: i32, month: u32, day: u32) -> Result<Self, DateOfBirthError> {
        NaiveDate::from_ymd_opt(year, month, day)
            .map(DateOfBirth)
            .ok_or(DateOfBirthError::InvalidCalendarDate { year, month, day })
    }

    pub fn year(&self) -> i32 {
        self.0.year()
    }

    pub fn month(&self) -> u32 {
        self.0.month()
    }

    pub fn day(&self) -> u32 {
        self.0.day()
    }
}

impl fmt::Display for DateOfBirth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pesel([u8; 11]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeselError {
    WrongLength { actual: usize },
    NonDigitCharacter { character: char },
    ChecksumMismatch { expected: u8, actual: u8 },
    InvalidEncodedMonth { encoded_month: u8 },
    InvalidEncodedDate { year: i32, month: u32, day: u32 },
    YearOutOfRange { year: i32 },
    SerialOutOfRange { serial: u16 },
}

impl fmt::Display for PeselError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PeselError::WrongLength { actual } => {
                write!(f, "PESEL must be exactly 11 digits, got {actual}")
            }
            PeselError::NonDigitCharacter { character } => {
                write!(f, "PESEL must contain only digits, found '{character}'")
            }
            PeselError::ChecksumMismatch { expected, actual } => {
                write!(f, "checksum mismatch: expected {expected}, got {actual}")
            }
            PeselError::InvalidEncodedMonth { encoded_month } => {
                write!(f, "{encoded_month:02} is not a valid PESEL-encoded month")
            }
            PeselError::InvalidEncodedDate { year, month, day } => {
                write!(
                    f,
                    "{year:04}-{month:02}-{day:02} is not a valid calendar date"
                )
            }
            PeselError::YearOutOfRange { year } => {
                write!(
                    f,
                    "year {year} is outside the range PESEL can encode (1800-2299)"
                )
            }
            PeselError::SerialOutOfRange { serial } => {
                write!(f, "serial must be in 0..=999, got {serial}")
            }
        }
    }
}

impl std::error::Error for PeselError {}

const CHECKSUM_WEIGHTS: [u32; 10] = [1, 3, 7, 9, 1, 3, 7, 9, 1, 3];

fn checksum_digit(first_ten: &[u8; 10]) -> u8 {
    let sum: u32 = first_ten
        .iter()
        .zip(CHECKSUM_WEIGHTS.iter())
        .map(|(&digit, &weight)| digit as u32 * weight)
        .sum();
    ((10 - (sum % 10)) % 10) as u8
}

fn encode_month(year: i32, month: u32) -> Result<u32, PeselError> {
    let offer = match year {
        1800..=1899 => 80,
        1900..=1999 => 0,
        2000..=2099 => 20,
        2100..=2199 => 40,
        2200..=2299 => 60,
        _ => return Err(PeselError::YearOutOfRange { year }),
    };
    Ok(month + offer)
}

fn decode_month(encoded_month: u8) -> Result<(i32, u32), PeselError> {
    let m = encoded_month as u32;
    match m {
        1..=12 => Ok((1900, m)),
        21..=32 => Ok((2000, m - 20)),
        41..=52 => Ok((2100, m - 40)),
        61..=72 => Ok((2200, m - 60)),
        81..=92 => Ok((1800, m - 80)),
        _ => Err(PeselError::InvalidEncodedMonth { encoded_month }),
    }
}

impl Pesel {
    pub fn from_parts(date: DateOfBirth, gender: Gender, serial: u16) -> Result<Self, PeselError> {
        if serial > 999 {
            return Err(PeselError::SerialOutOfRange { serial });
        }

        let encoded_month = encode_month(date.year(), date.month())?;
        let two_digit_year = date.year().rem_euclid(100) as u32;

        let mut digits = [0u8; 11];
        digits[0] = (two_digit_year / 10) as u8;
        digits[1] = (two_digit_year % 10) as u8;
        digits[2] = (encoded_month / 10) as u8;
        digits[3] = (encoded_month % 10) as u8;
        digits[4] = (date.day() / 10) as u8;
        digits[5] = (date.day() % 10) as u8;
        digits[6] = ((serial / 100) % 10) as u8;
        digits[7] = ((serial / 10) % 10) as u8;
        digits[8] = (serial % 10) as u8;
        digits[9] = match gender {
            Gender::Male => 1,
            Gender::Female => 0,
        };

        let first_ten: [u8; 10] = digits[0..10].try_into().unwrap();
        digits[10] = checksum_digit(&first_ten);

        Ok(Pesel(digits))
    }

    pub fn parse(s: &str) -> Result<Self, PeselError> {
        let char_count = s.chars().count();
        if char_count != 11 {
            return Err(PeselError::WrongLength { actual: char_count });
        }

        let mut digits = [0u8; 11];
        for (i, c) in s.chars().enumerate() {
            let d = c
                .to_digit(10)
                .ok_or(PeselError::NonDigitCharacter { character: c })?;
            digits[i] = d as u8;
        }

        let first_ten: [u8; 10] = digits[0..10].try_into().unwrap();
        let expected = checksum_digit(&first_ten);
        if expected != digits[10] {
            return Err(PeselError::ChecksumMismatch {
                expected,
                actual: digits[10],
            });
        }

        let encoded_month = digits[2] * 10 + digits[3];
        let (century_base, month) = decode_month(encoded_month)?;
        let year = century_base + (digits[0] as i32 * 10 + digits[1] as i32);
        let day = (digits[4] * 10 + digits[5]) as u32;
        DateOfBirth::new(year, month, day).map_err(|_| PeselError::InvalidEncodedDate {
            year,
            month,
            day,
        })?;

        Ok(Pesel(digits))
    }

    pub fn date_of_birth(&self) -> DateOfBirth {
        let encoded_month = self.0[2] * 10 + self.0[3];
        let (century_base, month) =
            decode_month(encoded_month).expect("a constructed Pesel always has a valid month");
        let year = century_base + (self.0[0] as i32 * 10 + self.0[1] as i32);
        let day = (self.0[4] * 10 + self.0[5]) as u32;
        DateOfBirth::new(year, month, day).expect("a constructed Pesel always encodes a valid date")
    }

    pub fn gender(&self) -> Gender {
        if self.0[9] % 2 == 1 {
            Gender::Male
        } else {
            Gender::Female
        }
    }

    pub fn as_str(&self) -> String {
        self.0.iter().map(|d| (b'0' + d) as char).collect()
    }
}

impl fmt::Display for Pesel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn male_display_as_male() {
        assert_eq!(Gender::Male.to_string(), "Male");
    }

    #[test]
    fn female_display_as_female() {
        assert_eq!(Gender::Female.to_string(), "Female");
    }

    #[test]
    fn genders_are_equal_to_themselves() {
        assert_eq!(Gender::Male, Gender::Male);
        assert_eq!(Gender::Female, Gender::Female);
    }
}

#[cfg(test)]
mod date_of_birth_tests {
    use super::*;

    #[test]
    fn accepts_an_ordinary_valid_date() {
        let dob = DateOfBirth::new(1990, 6, 15).unwrap();
        assert_eq!(dob.to_string(), "1990-06-15");
    }

    #[test]
    fn accepts_leap_day_in_a_leap_year() {
        assert!(DateOfBirth::new(2000, 2, 29).is_ok());
        assert!(DateOfBirth::new(2004, 2, 29).is_ok());
    }

    #[test]
    fn rejects_leap_day_in_a_non_leap_year() {
        assert!(DateOfBirth::new(1900, 2, 29).is_err());
        assert!(DateOfBirth::new(2001, 2, 29).is_err());
    }

    #[test]
    fn rejects_month_thirteen() {
        let err = DateOfBirth::new(1990, 13, 1).unwrap_err();
        assert_eq!(
            err,
            DateOfBirthError::InvalidCalendarDate {
                year: 1990,
                month: 13,
                day: 1
            }
        );
    }

    #[test]
    fn rejects_day_zero_and_day_thirty_two() {
        assert!(DateOfBirth::new(1990, 1, 0).is_err());
        assert!(DateOfBirth::new(1990, 1, 32).is_err());
    }

    #[test]
    fn rejects_april_thirty_first() {
        assert!(DateOfBirth::new(1990, 4, 31).is_err());
    }
}

#[cfg(test)]
mod pesel_tests {
    use super::*;

    #[test]
    fn parses_a_known_valid_pesel() {
        let pesel = Pesel::parse("44051401359").unwrap();
        assert_eq!(pesel.date_of_birth().to_string(), "1944-05-14");
        assert_eq!(pesel.gender(), Gender::Male);
    }

    #[test]
    fn from_parts_roundtrips_through_parse() {
        let dob = DateOfBirth::new(1990, 6, 15).unwrap();
        let built = Pesel::from_parts(dob, Gender::Female, 42).unwrap();

        let reparsed = Pesel::parse(&built.to_string()).unwrap();
        assert_eq!(reparsed.date_of_birth(), dob);
        assert_eq!(reparsed.gender(), Gender::Female);
    }

    #[test]
    fn encodes_1900s_century_with_no_month_offset() {
        let dob = DateOfBirth::new(1944, 5, 14).unwrap();
        let pesel = Pesel::from_parts(dob, Gender::Male, 135).unwrap();
        assert_eq!(&pesel.to_string()[2..4], "05");
    }

    #[test]
    fn encodes_2000s_century_with_plus_twenty_month_offset() {
        let dob = DateOfBirth::new(2005, 3, 14).unwrap();
        let pesel = Pesel::from_parts(dob, Gender::Male, 1).unwrap();
        assert_eq!(&pesel.to_string()[2..4], "23");
    }

    #[test]
    fn encodes_1800s_century_with_plus_eighty_month_offset() {
        let dob = DateOfBirth::new(1888, 1, 1).unwrap();
        let pesel = Pesel::from_parts(dob, Gender::Female, 1).unwrap();
        assert_eq!(&pesel.to_string()[2..4], "81");
        assert_eq!(pesel.date_of_birth().to_string(), "1888-01-01");
    }

    #[test]
    fn gender_digit_parity_matches_gender() {
        let dob = DateOfBirth::new(1990, 1, 1).unwrap();
        let male = Pesel::from_parts(dob, Gender::Male, 1).unwrap();
        let female = Pesel::from_parts(dob, Gender::Female, 1).unwrap();
        assert_eq!(male.gender(), Gender::Male);
        assert_eq!(female.gender(), Gender::Female);
    }

    #[test]
    fn rejects_wrong_length() {
        assert_eq!(
            Pesel::parse("123").unwrap_err(),
            PeselError::WrongLength { actual: 3 }
        );
    }

    #[test]
    fn rejects_non_digit_characters() {
        let err = Pesel::parse("4405140135X").unwrap_err();
        assert_eq!(err, PeselError::NonDigitCharacter { character: 'X' });
    }

    #[test]
    fn rejects_bad_checksum() {
        let err = Pesel::parse("44051401350").unwrap_err();
        assert_eq!(
            err,
            PeselError::ChecksumMismatch {
                expected: 9,
                actual: 0
            }
        );
    }

    #[test]
    fn rejects_invalid_encoded_month() {
        let mut digits = [4u8, 4, 9, 9, 1, 4, 0, 1, 3, 5, 0];
        let checksum = checksum_digit(&digits[0..10].try_into().unwrap());
        digits[10] = checksum;
        let s: String = digits.iter().map(|d| (b'0' + d) as char).collect();

        let err = Pesel::parse(&s).unwrap_err();
        assert_eq!(err, PeselError::InvalidEncodedMonth { encoded_month: 99 });
    }

    #[test]
    fn from_parts_rejects_serial_over_999() {
        let dob = DateOfBirth::new(1990, 1, 1).unwrap();
        let err = Pesel::from_parts(dob, Gender::Male, 1000).unwrap_err();
        assert_eq!(err, PeselError::SerialOutOfRange { serial: 1000 });
    }
}
