use chrono::{Datelike, NaiveDate};
use std::fmt;

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
