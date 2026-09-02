use domain::Gender;
use domain::profile::TestProfile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, PartialEq)]
pub struct PersonDto {
    pub first_name: String,
    pub last_name: String,
    pub gender: String,
    pub date_of_birth: String,
    pub pesel: String,
    pub nip: String,
    pub city: String,
    pub postal_code: String,
    pub phone: String,
    pub email: String,
    pub id_document: String,
    pub iban: String,
}

impl PersonDto {
    pub fn new(profile: TestProfile) -> Self {
        let TestProfile {
            person,
            nip,
            address,
            phone,
            email,
            id_document,
            iban,
        } = profile;
        PersonDto {
            first_name: person.first_name,
            last_name: person.last_name,
            gender: person.gender.to_string(),
            date_of_birth: person.date_of_birth.to_string(),
            pesel: person.pesel.to_string(),
            nip: nip.to_string(),
            city: address.city,
            postal_code: address.postal_code.to_string(),
            phone: phone.to_string(),
            email: email.to_string(),
            id_document: id_document.to_string(),
            iban: iban.to_string(),
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

#[derive(Debug, Deserialize, Default, Clone)]
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
            min_date: parse_iso_date_field("min_date", &self.min_date)?,
            max_date: parse_iso_date_field("max_date", &self.max_date)?,
            seed,
            count,
        })
    }
}

fn parse_iso_date_field(field: &'static str, raw: &str) -> Result<Option<String>, FormParseError> {
    let invalid = || FormParseError {
        field,
        value: raw.to_string(),
    };

    match non_empty(raw) {
        None => Ok(None),
        Some(s) => {
            let parts: Vec<&str> = s.split('-').collect();
            let [y, m, d] = parts.as_slice() else {
                return Err(invalid());
            };
            if y.len() != 4 || m.len() != 2 || d.len() != 2 {
                return Err(invalid());
            }
            if !y.chars().all(|c| c.is_ascii_digit())
                || !m.chars().all(|c| c.is_ascii_digit())
                || !d.chars().all(|c| c.is_ascii_digit())
            {
                return Err(invalid());
            }
            Ok(Some(s.to_string()))
        }
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
    use domain::address::Address;
    use domain::email::{EmailAddress, ReservedDomain};
    use domain::person::Person;
    use domain::phone::PhoneNumber;
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
        let nip = domain::nip::Nip::from_digits([1, 2, 3, 4, 5, 6, 3, 2, 1]).unwrap();
        let address = Address {
            city: "Kraków".to_string(),
            postal_code: domain::address::PostalCode::from_digits([3, 0, 0, 0, 1]),
        };
        let phone = PhoneNumber::from_digits([5, 0, 1, 2, 3, 4, 5, 6, 7]).unwrap();
        let email = EmailAddress::new("anna.nowak", ReservedDomain::ExampleCom);
        let id_document =
            domain::document::IdDocumentNumber::from_parts([0, 1, 2], [1, 2, 3, 4, 5]);
        let iban = domain::iban::Iban::from_parts(
            [1, 0, 9, 0, 1, 0, 1, 4],
            [0, 0, 0, 0, 0, 7, 1, 2, 1, 9, 8, 1, 2, 8, 7, 4],
        );

        let profile = TestProfile {
            person,
            nip,
            address,
            phone,
            email,
            id_document,
            iban,
        };
        let dto = PersonDto::new(profile);

        assert_eq!(dto.first_name, "Anna");
        assert_eq!(dto.gender, "Female");
        assert_eq!(dto.date_of_birth, "1990-06-15");
        assert_eq!(dto.pesel, pesel.to_string());
        assert_eq!(dto.nip, "1234563218");
        assert_eq!(dto.city, "Kraków");
        assert_eq!(dto.postal_code, "30-001");
        assert_eq!(dto.phone, "+48 501 234 567");
        assert_eq!(dto.email, "anna.nowak@example.com");
        assert_eq!(dto.id_document, "ABC412345");
        assert_eq!(dto.iban, iban.to_string());
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

    #[test]
    fn form_passes_through_iso_dates_unchanged() {
        let form = GenerateForm {
            min_date: "1990-06-01".to_string(),
            max_date: "1999-12-31".to_string(),
            ..Default::default()
        };
        let request = form.into_request().unwrap();
        assert_eq!(request.min_date.as_deref(), Some("1990-06-01"));
        assert_eq!(request.max_date.as_deref(), Some("1999-12-31"));
    }

    #[test]
    fn form_rejects_a_malformed_iso_date() {
        let form = GenerateForm {
            min_date: "01/06/1990".to_string(),
            ..Default::default()
        };
        let err = form.into_request().unwrap_err();
        assert_eq!(err.field, "min_date");
    }
}
