#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Pl,
    En,
}

impl Lang {
    pub fn path_segment(self) -> &'static str {
        match self {
            Lang::Pl => "pl",
            Lang::En => "en",
        }
    }

    pub fn other(self) -> Lang {
        match self {
            Lang::Pl => Lang::En,
            Lang::En => Lang::Pl,
        }
    }

    pub fn detect(accept_language: Option<&str>) -> Lang {
        let Some(value) = accept_language else {
            return Lang::En;
        };

        let first_tag = value
            .split(',')
            .next()
            .unwrap_or("")
            .split(';')
            .next()
            .unwrap_or("")
            .trim();

        if first_tag.eq_ignore_ascii_case("pl") || first_tag.to_ascii_lowercase().starts_with("pl-")
        {
            Lang::Pl
        } else {
            Lang::En
        }
    }

    pub fn messages(self) -> Messages {
        match self {
            Lang::Pl => Messages {
                html_lang: "pl",
                title: "Generator PESEL i danych testowych",
                tagline: "Generuj syntetyczne numery PESEL, NIP, adresy i pełne profile testowych osób do testów QA i walidacji formularzy.",
                gender_label: "Płeć",
                gender_any: "Dowolna",
                gender_male: "Mężczyzna",
                gender_female: "Kobieta",
                count_label: "Ile wygenerować",
                min_date_label: "Urodzony po",
                max_date_label: "Urodzony przed",
                seed_label: "Seed (opcjonalne - użyj ponownie, aby powtórzyć te same wyniki)",
                generate_button: "Generuj",
                disclaimer: "Wygenerowane dane są fikcyjne i nie odnoszą się do żadnej rzeczywistej osoby. Wyłącznie do celów testowych.",
                switch_label: "EN",
                field_first_name: "Imię",
                field_last_name: "Nazwisko",
                field_date_of_birth: "Data urodzenia",
                field_city: "Miasto",
                field_postal_code: "Kod pocztowy",
                field_phone: "Telefon",
                field_id_document: "Dokument tożsamości",
                field_email: "E-mail",
                seed_used_prefix: "Użyty seed:",
                seed_used_suffix: "- użyj go ponownie, aby powtórzyć te wyniki.",
                copy_all_label: "Kopiuj wszystko jako JSON",
            },
            Lang::En => Messages {
                html_lang: "en",
                title: "Polish Test Data Generator",
                tagline: "Generate synthetic PESEL numbers, NIP numbers, addresses, and full test-person profiles for QA, automated testing, and form validation.",
                gender_label: "Gender",
                gender_any: "Any",
                gender_male: "Male",
                gender_female: "Female",
                count_label: "How many",
                min_date_label: "Born after",
                max_date_label: "Born before",
                seed_label: "Seed (optional - reuse it to reproduce the same output)",
                generate_button: "Generate",
                disclaimer: "Generated values are fictitious and not linked to real people. For QA and testing use only.",
                switch_label: "PL",
                field_first_name: "First name",
                field_last_name: "Last name",
                field_date_of_birth: "Date of birth",
                field_city: "City",
                field_postal_code: "Postal code",
                field_phone: "Phone",
                field_id_document: "ID document",
                field_email: "Email",
                seed_used_prefix: "Seed used:",
                seed_used_suffix: "- resubmit with this seed to reproduce these rows.",
                copy_all_label: "Copy all as JSON",
            },
        }
    }
}

pub struct Messages {
    pub html_lang: &'static str,
    pub title: &'static str,
    pub tagline: &'static str,
    pub gender_label: &'static str,
    pub gender_any: &'static str,
    pub gender_male: &'static str,
    pub gender_female: &'static str,
    pub count_label: &'static str,
    pub min_date_label: &'static str,
    pub max_date_label: &'static str,
    pub seed_label: &'static str,
    pub generate_button: &'static str,
    pub disclaimer: &'static str,
    pub switch_label: &'static str,
    pub field_first_name: &'static str,
    pub field_last_name: &'static str,
    pub field_date_of_birth: &'static str,
    pub field_city: &'static str,
    pub field_postal_code: &'static str,
    pub field_phone: &'static str,
    pub field_id_document: &'static str,
    pub field_email: &'static str,
    pub seed_used_prefix: &'static str,
    pub seed_used_suffix: &'static str,
    pub copy_all_label: &'static str,
}
