use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female
}

impl fmt::Display for Gender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Gender::Male => write!(f, "Male"),
            Gender::Female => write!(f, "Female"),
        }
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
