use crate::dto::{GenerateForm, PersonDto};

const PAGE_TEMPLATE: &str = include_str!("../assets/page.html");
const FORM_TEMPLATE: &str = include_str!("../assets/form.html");
const RESULTS_TEMPLATE: &str = include_str!("../assets/results.html");
const RESULT_ROW_TEMPLATE: &str = include_str!("../assets/result_row.html");

pub struct PageContext<'a> {
    pub person_results: Option<(&'a [PersonDto], u64)>,
    pub submitted_form: Option<&'a GenerateForm>,
    pub error: Option<String>,
}

pub fn page(ctx: PageContext) -> String {
    let error_html = match &ctx.error {
        Some(msg) => format!(r#"<p class="error" role="alert">⚠️ {}</p>"#, escape(msg)),
        None => String::new(),
    };
    let form_html = render_form(ctx.submitted_form);
    let results_html = match ctx.person_results {
        Some((people, seed)) => result_section(people, seed),
        None => String::new(),
    };

    PAGE_TEMPLATE
        .replace("{{ERROR_HTML}}", &error_html)
        .replace("{{FORM_HTML}}", &form_html)
        .replace("{{RESULTS_HTML}}", &results_html)
}

fn render_form(submitted: Option<&GenerateForm>) -> String {
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

fn result_section(people: &[PersonDto], seed: u64) -> String {
    let rows: String = people.iter().map(render_row).collect();

    let json_payload = serde_json::to_string(people).unwrap_or_else(|_| "[]".to_string());
    let json_escaped = json_payload
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    RESULTS_TEMPLATE
        .replace("{{SEED}}", &seed.to_string())
        .replace("{{JSON_PAYLOAD}}", &json_escaped)
        .replace("{{ROWS}}", &rows)
}

fn render_row(p: &PersonDto) -> String {
    let badge_class = if p.gender == "Male" {
        "badge-male"
    } else {
        "badge-female"
    };

    RESULT_ROW_TEMPLATE
        .replace("{{FIRST_NAME}}", &escape(&p.first_name))
        .replace("{{LAST_NAME}}", &escape(&p.last_name))
        .replace("{{BADGE_CLASS}}", badge_class)
        .replace("{{GENDER}}", &escape(&p.gender))
        .replace("{{DATE_OF_BIRTH}}", &escape(&p.date_of_birth))
        .replace("{{PESEL}}", &escape(&p.pesel))
        .replace("{{NIP}}", &escape(&p.nip))
        .replace("{{CITY}}", &escape(&p.city))
        .replace("{{POSTAL_CODE}}", &escape(&p.postal_code))
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
