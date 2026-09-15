use crate::dto::{GenerateForm, PersonDto};
use crate::lang::Lang;

const PAGE_TEMPLATE: &str = include_str!("../assets/page.html");
const FORM_TEMPLATE: &str = include_str!("../assets/form.html");
const RESULTS_TEMPLATE: &str = include_str!("../assets/results.html");
const RESULT_CARD_TEMPLATE: &str = include_str!("../assets/result_card.html");

pub struct PageContext<'a> {
    pub lang: Lang,
    pub person_results: Option<(&'a [PersonDto], u64)>,
    pub submitted_form: Option<&'a GenerateForm>,
    pub error: Option<String>,
}

pub fn page(ctx: PageContext) -> String {
    let m = ctx.lang.messages();
    let other = ctx.lang.other();

    let error_html = match &ctx.error {
        Some(msg) => format!(r#"<p class="error" role="alert">⚠️ {}</p>"#, escape(msg)),
        None => String::new(),
    };
    let form_html = render_form(ctx.lang, ctx.submitted_form);
    let results_html = match ctx.person_results {
        Some((people, seed)) => result_section(ctx.lang, people, seed),
        None => String::new(),
    };

    let hreflang_links = r#"<link rel="alternate" hreflang="pl" href="https://pl-testdata.onrender.com/pl/" /><link rel="alternate" hreflang="en" href="https://pl-testdata.onrender.com/en/" />"#;
    let switch_href = format!("/{}/", other.path_segment());
    let canonical = format!(
        "https://pl-testdata.onrender.com/{}/",
        ctx.lang.path_segment()
    );
    let og_url = canonical.clone();

    PAGE_TEMPLATE
        .replace("{{HTML_LANG}}", m.html_lang)
        .replace("{{TITLE}}", m.title)
        .replace("{{TAGLINE}}", m.tagline)
        .replace("{{DISCLAIMER}}", m.disclaimer)
        .replace("{{HREFLANG_LINKS}}", hreflang_links)
        .replace("{{LANG_SWITCH_HREF}}", &switch_href)
        .replace("{{LANG_SWITCH_LABEL}}", m.switch_label)
        .replace("{{CANONICAL_URL}}", &canonical)
        .replace("{{OG_URL}}", &og_url)
        .replace("{{ERROR_HTML}}", &error_html)
        .replace("{{FORM_HTML}}", &form_html)
        .replace("{{RESULTS_HTML}}", &results_html)
}

fn render_form(lang: Lang, submitted: Option<&GenerateForm>) -> String {
    let m = lang.messages();
    let (gender, min_date, max_date, seed, count) = match submitted {
        Some(f) => (
            f.gender.as_str(),
            f.min_date.as_str(),
            f.max_date.as_str(),
            f.seed.as_str(),
            f.count.as_str(),
        ),
        None => ("", "", "", "", ""),
    };
    let count_value = if count.is_empty() { "1" } else { count };

    FORM_TEMPLATE
        .replace(
            "{{FORM_ACTION}}",
            &format!("/{}/generate", lang.path_segment()),
        )
        .replace("{{LABEL_GENDER}}", m.gender_label)
        .replace("{{OPTION_ANY}}", m.gender_any)
        .replace("{{OPTION_MALE}}", m.gender_male)
        .replace("{{OPTION_FEMALE}}", m.gender_female)
        .replace("{{LABEL_COUNT}}", m.count_label)
        .replace("{{LABEL_MIN_DATE}}", m.min_date_label)
        .replace("{{LABEL_MAX_DATE}}", m.max_date_label)
        .replace("{{LABEL_SEED}}", m.seed_label)
        .replace("{{BUTTON_GENERATE}}", m.generate_button)
        .replace(
            "{{GENDER_ANY_SELECTED}}",
            if gender.is_empty() { "selected" } else { "" },
        )
        .replace(
            "{{GENDER_MALE_SELECTED}}",
            if gender == "male" { "selected" } else { "" },
        )
        .replace(
            "{{GENDER_FEMALE_SELECTED}}",
            if gender == "female" { "selected" } else { "" },
        )
        .replace("{{COUNT_VALUE}}", &escape(count_value))
        .replace("{{MIN_DATE}}", &escape(min_date))
        .replace("{{MAX_DATE}}", &escape(max_date))
        .replace("{{SEED}}", &escape(seed))
}

fn result_section(lang: Lang, people: &[PersonDto], seed: u64) -> String {
    let m = lang.messages();
    let cards: String = people.iter().map(|p| render_card(lang, p)).collect();

    let json_payload = serde_json::to_string(people).unwrap_or_else(|_| "[]".to_string());
    let json_escaped = json_payload
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    RESULTS_TEMPLATE
        .replace("{{SEED_USED_PREFIX}}", m.seed_used_prefix)
        .replace("{{SEED}}", &seed.to_string())
        .replace("{{SEED_USED_SUFFIX}}", m.seed_used_suffix)
        .replace("{{COPY_ALL_LABEL}}", m.copy_all_label)
        .replace("{{JSON_PAYLOAD}}", &json_escaped)
        .replace("{{ROWS}}", &cards)
}

fn render_card(lang: Lang, p: &PersonDto) -> String {
    let m = lang.messages();
    let badge_class = if p.gender == "Male" {
        "badge-male"
    } else {
        "badge-female"
    };
    let gender_display = if p.gender == "Male" {
        m.gender_male
    } else {
        m.gender_female
    };

    RESULT_CARD_TEMPLATE
        .replace("{{FIRST_NAME}}", &escape(&p.first_name))
        .replace("{{LAST_NAME}}", &escape(&p.last_name))
        .replace("{{BADGE_CLASS}}", badge_class)
        .replace("{{GENDER}}", gender_display)
        .replace("{{DATE_OF_BIRTH}}", &escape(&p.date_of_birth))
        .replace("{{PESEL}}", &escape(&p.pesel))
        .replace("{{NIP}}", &escape(&p.nip))
        .replace("{{CITY}}", &escape(&p.city))
        .replace("{{POSTAL_CODE}}", &escape(&p.postal_code))
        .replace("{{PHONE}}", &escape(&p.phone))
        .replace("{{EMAIL}}", &escape(&p.email))
        .replace("{{ID_DOCUMENT}}", &escape(&p.id_document))
        .replace("{{IBAN}}", &escape(&p.iban))
        .replace("{{LABEL_FIRST_NAME}}", m.field_first_name)
        .replace("{{LABEL_LAST_NAME}}", m.field_last_name)
        .replace("{{LABEL_GENDER}}", m.gender_label)
        .replace("{{LABEL_DATE_OF_BIRTH}}", m.field_date_of_birth)
        .replace("{{LABEL_CITY}}", m.field_city)
        .replace("{{LABEL_POSTAL_CODE}}", m.field_postal_code)
        .replace("{{LABEL_PHONE}}", m.field_phone)
        .replace("{{LABEL_ID_DOCUMENT}}", m.field_id_document)
        .replace("{{LABEL_EMAIL}}", m.field_email)
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
