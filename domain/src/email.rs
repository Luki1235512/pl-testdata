use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReservedDomain {
    ExampleCom,
    ExampleOrg,
    ExampleNet,
}

impl ReservedDomain {
    const ALL: [ReservedDomain; 3] = [
        ReservedDomain::ExampleCom,
        ReservedDomain::ExampleOrg,
        ReservedDomain::ExampleNet,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            ReservedDomain::ExampleCom => "example.com",
            ReservedDomain::ExampleOrg => "example.org",
            ReservedDomain::ExampleNet => "example.net",
        }
    }
}

impl fmt::Display for ReservedDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailAddress {
    local_part: String,
    domain: ReservedDomain,
}

impl EmailAddress {
    pub fn new(local_part: impl Into<String>, domain: ReservedDomain) -> Self {
        EmailAddress {
            local_part: sanitize_local_part(&local_part.into()),
            domain,
        }
    }

    pub fn as_str(&self) -> String {
        format!("{}@{}", self.local_part, self.domain.as_str())
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub fn generate_email(
    rng: &mut impl rand::RngExt,
    first_name: &str,
    last_name: &str,
) -> EmailAddress {
    let disambiguator = rng.random_range(10..=999);
    let local = format!(
        "{}.{}{disambiguator}",
        ascii_slug(first_name),
        ascii_slug(last_name)
    );
    let domain = ReservedDomain::ALL[rng.random_range(0..ReservedDomain::ALL.len())];
    EmailAddress::new(local, domain)
}

fn ascii_slug(s: &str) -> String {
    sanitize_local_part(s)
}

fn sanitize_local_part(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(transliterate_polish_char)
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.')
        .map(|c| c.to_ascii_lowercase())
        .collect();

    if cleaned.is_empty() {
        "user".to_string()
    } else {
        cleaned
    }
}

fn transliterate_polish_char(c: char) -> char {
    match c {
        'ą' | 'Ą' => 'a',
        'ć' | 'Ć' => 'c',
        'ę' | 'Ę' => 'e',
        'ł' | 'Ł' => 'l',
        'ń' | 'Ń' => 'n',
        'ó' | 'Ó' => 'o',
        'ś' | 'Ś' => 's',
        'ź' | 'Ź' => 'z',
        'ż' | 'Ż' => 'z',
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_lowercases_and_formats_the_address() {
        let email = EmailAddress::new("Anna.Nowak42", ReservedDomain::ExampleCom);
        assert_eq!(email.to_string(), "anna.nowak42@example.com");
    }

    #[test]
    fn new_transliterates_polish_diacritics() {
        let email = EmailAddress::new("Łukasz.Żółć", ReservedDomain::ExampleOrg);
        assert_eq!(email.to_string(), "lukasz.zolc@example.org");
    }

    #[test]
    fn new_falls_back_to_user_for_an_empty_local_part() {
        let email = EmailAddress::new("", ReservedDomain::ExampleNet);
        assert_eq!(email.to_string(), "user@example.net");
    }
}

#[cfg(test)]
mod generate_tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn generated_addresses_always_use_a_reserved_domain() {
        let mut rng = StdRng::seed_from_u64(4);
        let reserved: Vec<&str> = ReservedDomain::ALL.iter().map(|d| d.as_str()).collect();

        for _ in 0..500 {
            let email = generate_email(&mut rng, "Jan", "Kowalski").to_string();
            let domain = email.split('@').nth(1).expect("address has a domain part");
            assert!(
                reserved.contains(&domain),
                "{email} uses a non-reserved domain '{domain}'"
            );
        }
    }

    #[test]
    fn generated_addresses_are_syntactically_well_formed() {
        let mut rng = StdRng::seed_from_u64(4);

        for _ in 0..200 {
            let email = generate_email(&mut rng, "Ewa", "Wiśniewska").to_string();
            let parts: Vec<&str> = email.split('@').collect();
            assert_eq!(parts.len(), 2, "{email} should have exactly one '@'");
            assert!(!parts[0].is_empty());
        }
    }

    #[test]
    fn generated_local_part_is_derived_from_the_given_name() {
        let mut rng = StdRng::seed_from_u64(4);
        let email = generate_email(&mut rng, "Jan", "Kowalski").to_string();
        assert!(email.starts_with("jan.kowalski"));
    }

    #[test]
    fn same_seed_produces_identical_sequences() {
        let mut rng_a = StdRng::seed_from_u64(42);
        let mut rng_b = StdRng::seed_from_u64(42);

        let seq_a: Vec<String> = (0..30)
            .map(|_| generate_email(&mut rng_a, "Jan", "Kowalski").to_string())
            .collect();
        let seq_b: Vec<String> = (0..30)
            .map(|_| generate_email(&mut rng_b, "Jan", "Kowalski").to_string())
            .collect();

        assert_eq!(seq_a, seq_b);
    }
}
