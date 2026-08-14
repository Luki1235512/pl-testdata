use domain::Gender;
use domain::person::Person;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, PartialEq)]
pub struct PersonDto {
    pub first_name: String,
    pub last_name: String,
    pub gender: String,
    pub date_of_birth: String,
    pub pesel: String,
}

impl From<Person> for PersonDto {
    fn from(person: Person) -> Self {
        PersonDto {
            first_name: person.first_name,
            last_name: person.last_name,
            gender: person.gender.to_string(),
            date_of_birth: person.date_of_birth.to_string(),
            pesel: person.pesel.to_string(),
        }
    }
}

pub const SYNTHETIC_DATA_DISCLAIMER: &str = "Synthetic test data generated for software testing only. Does not represent a real individual.";

#[derive(Debug, Serialize)]
pub struct GenerateResponse {
    pub disclaimer: &'static str,
    pub resolved_seed: u64,
    pub people: Vec<PersonDto>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GenderDto {
    Male,
    Female,
}

impl From<GenderDto> for Gender {
    fn from(g: GenderDto) -> Self {
        match g {
            GenderDto::Male => Gender::Male,
            GenderDto::Female => Gender::Female,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct GenerateRequest {
    pub gender: Option<GenderDto>,
    pub min_date: Option<String>,
    pub max_date: Option<String>,
    pub seed: Option<u64>,
    pub count: Option<u8>,
}

#[derive(Debug, Deserialize, Default)]
pub struct GenerateForm {
    #[serde(default)]
    pub gender: String,
    #[serde(default)]
    pub min_date: String,
    #[serde(default)]
    pub max_date: String,
    #[serde(default)]
    pub seed: String,
    #[serde(default)]
    pub count: String,
}

#[derive(Debug)]
pub struct FormParseError {
    pub field: &'static str,
    pub value: String,
}

impl GenerateForm {
    pub fn into_request(self) -> Result<GenerateRequest, FormParseError> {
        let gender = match self.gender.as_str() {
            "" => None,
            "male" => Some(GenderDto::Male),
            "female" => Some(GenderDto::Female),
            other => {
                return Err(FormParseError {
                    field: "gender",
                    value: other.to_string(),
                });
            }
        };

        let seed = parse_field("seed", &self.seed)?;
        let count = parse_field("count", &self.count)?;

        Ok(GenerateRequest {
            gender,
            min_date: non_empty(&self.min_date).map(str::to_string),
            max_date: non_empty(&self.max_date).map(str::to_string),
            seed,
            count,
        })
    }
}

fn parse_field<T: std::str::FromStr>(
    field: &'static str,
    raw: &str,
) -> Result<Option<T>, FormParseError> {
    match non_empty(raw) {
        None => Ok(None),
        Some(s) => s.parse().map(Some).map_err(|_| FormParseError {
            field,
            value: s.to_string(),
        }),
    }
}

fn non_empty(s: &str) -> Option<&str> {
    if s.is_empty() { None } else { Some(s) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{DateOfBirth, Pesel};

    #[test]
    fn person_maps_to_dto_with_display_formatted_fields() {
        let dob = DateOfBirth::new(1990, 6, 15).unwrap();
        let pesel = Pesel::from_parts(dob, Gender::Female, 42).unwrap();
        let person = Person {
            first_name: "Anna".to_string(),
            last_name: "Nowak".to_string(),
            gender: Gender::Female,
            date_of_birth: dob,
            pesel,
        };

        let dto = PersonDto::from(person);

        assert_eq!(dto.first_name, "Anna");
        assert_eq!(dto.gender, "Female");
        assert_eq!(dto.date_of_birth, "1990-06-15");
        assert_eq!(dto.pesel, pesel.to_string());
    }

    #[test]
    fn blank_form_becomes_a_fully_unconstrained_request() {
        let form = GenerateForm::default();
        let request = form.into_request().unwrap();

        assert!(request.gender.is_none());
        assert!(request.min_date.is_none());
        assert!(request.seed.is_none());
        assert!(request.count.is_none());
    }

    #[test]
    fn form_rejects_unknown_gender_value() {
        let form = GenerateForm {
            gender: "nonbinary-typo".to_string(),
            ..Default::default()
        };
        let err = form.into_request().unwrap_err();
        assert_eq!(err.field, "gender");
    }
}
