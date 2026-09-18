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
                faq_heading: "Najczęściej zadawane pytania",
                faq_items: &[
                    (
                        "Czy wygenerowany numer PESEL jest prawdziwy?",
                        "Nie. Numer PESEL, NIP i pozostałe dane są w pełni syntetyczne - mają poprawną strukturę i sumę kontrolną, ale nie odpowiadają żadnej rzeczywistej osobie ani nie są zarejestrowane w żadnym systemie państwowym.",
                    ),
                    (
                        "Jak liczona jest suma kontrolna numeru PESEL?",
                        "PESEL koduje datę urodzenia (z przesunięciem miesiąca dla stuleci innych niż 1900-1999), płeć w dziesiątej cyfrze oraz jednocyfrową sumę kontrolną wyliczaną z wag 1,3,7,9,1,3,7,9,1 nałożonych na pierwsze dziewięć cyfr.",
                    ),
                    (
                        "Czy mogę odtworzyć te same dane testowe później?",
                        "Tak - podaj tę samą wartość seed przy kolejnym generowaniu, a wynik będzie identyczny co do bajtu. To przydatne przy powtarzalnych testach automatycznych i regresyjnych.",
                    ),
                    (
                        "Do czego mogę użyć tego generatora?",
                        "Do wypełniania formularzy rejestracyjnych, testowania walidacji PESEL/NIP po stronie frontendu i backendu, generowania danych do testów E2E, obciążeniowych oraz fixture'ów w testach jednostkowych i integracyjnych.",
                    ),
                    (
                        "Czy dane mogą przez przypadek pokrywać się z prawdziwą osobą?",
                        "Teoretycznie tak, tak jak każda losowo wygenerowana liczba może pokrywać się z istniejącym numerem - dane nie są jednak pobierane z żadnego rejestru osób ani w żaden sposób z nim powiązane.",
                    ),
                    (
                        "Czy generator udostępnia API?",
                        "Tak, pod adresem /api/v1/persons w formacie JSON, z tymi samymi parametrami co formularz (płeć, zakres dat urodzenia, seed, liczba rekordów), co pozwala włączyć generator bezpośrednio do skryptów testowych i pipeline'ów CI/CD.",
                    ),
                ],
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
                faq_heading: "Frequently asked questions",
                faq_items: &[
                    (
                        "Is the generated PESEL number real?",
                        "No. The PESEL, NIP, and every other field are fully synthetic - they have a structurally valid format and checksum, but they are not tied to any real person and are not registered in any government system.",
                    ),
                    (
                        "How is the PESEL checksum calculated?",
                        "A PESEL encodes the date of birth (with a month offset for centuries other than 1900-1999), gender in the tenth digit, and a single check digit computed from the weights 1,3,7,9,1,3,7,9,1 applied to the first nine digits.",
                    ),
                    (
                        "Can I reproduce the same test data later?",
                        "Yes - pass the same seed value on your next request and the output will be byte-for-byte identical. This is useful for repeatable automated tests and regression fixtures.",
                    ),
                    (
                        "What can I use this generator for?",
                        "Filling out registration forms, testing PESEL/NIP validation on the frontend and backend, generating data for end-to-end and load tests, and building fixtures for unit and integration tests.",
                    ),
                    (
                        "Could a generated value accidentally match a real person?",
                        "In principle, yes - the same way any randomly generated number could coincide with a real one. The values are not drawn from, or linked to, any registry of real people.",
                    ),
                    (
                        "Is there an API?",
                        "Yes, at /api/v1/persons over JSON, with the same parameters as the form (gender, date-of-birth range, seed, record count), so you can wire the generator directly into test scripts and CI/CD pipelines.",
                    ),
                ],
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
    pub faq_heading: &'static str,
    pub faq_items: &'static [(&'static str, &'static str)],
}
