use crate::dto::{GenerateForm, PersonDto};

const PAGE_TEMPLATE: &str = include_str!("../assets/page.html");

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

    format!(
        r#"<form method="post" action="/generate">
  <fieldset>
    <legend>Generation options</legend>

    <div class="field">
      <label for="gender">Gender</label>
      <select id="gender" name="gender">
        <option value="" {any_sel}>Any</option>
        <option value="male" {male_sel}>Male</option>
        <option value="female" {female_sel}>Female</option>
      </select>
    </div>

    <div class="field">
      <label for="count">How many</label>
      <input type="number" id="count" name="count" min="1" max="50" value="{count_value}">
    </div>

    <div class="field">
      <label for="min_date">Born after</label>
      <input type="date" id="min_date" name="min_date" value="{min_date}">
    </div>

    <div class="field">
      <label for="max_date">Born before</label>
      <input type="date" id="max_date" name="max_date" value="{max_date}">
    </div>

    <div class="field span-2">
      <label for="seed">Seed (optional — reuse it to reproduce the same output)</label>
      <input type="number" id="seed" name="seed" min="0" value="{seed}">
    </div>

    <button type="submit">Generate</button>
  </fieldset>
</form>"#,
        any_sel = if gender.is_empty() { "selected" } else { "" },
        male_sel = if gender == "male" { "selected" } else { "" },
        female_sel = if gender == "female" { "selected" } else { "" },
        min_date = escape(min_date),
        max_date = escape(max_date),
        seed = escape(seed),
        count_value = escape(count_value),
    )
}

fn result_section(people: &[PersonDto], seed: u64) -> String {
    let rows: String = people
        .iter()
        .map(|p| {
            let badge_class = if p.gender == "Male" {
                "badge-male"
            } else {
                "badge-female"
            };
            format!(
                r#"<tr>
  <td>{}</td><td>{}</td><td><span class="badge {badge_class}">{}</span></td><td>{}</td>
  <td><code>{}</code> <button type="button" class="copy" data-copy="{}">Copy</button></td>
  <td><code>{}</code> <button type="button" class="copy" data-copy="{}">Copy</button></td>
</tr>"#,
                escape(&p.first_name),
                escape(&p.last_name),
                escape(&p.gender),
                escape(&p.date_of_birth),
                escape(&p.pesel),
                escape(&p.pesel),
                escape(&p.nip),
                escape(&p.nip),
            )
        })
        .collect();

    let json_payload = serde_json::to_string(people).unwrap_or_else(|_| "[]".to_string());
    let json_escaped = json_payload
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    format!(
        r#"<section class="card">
<p class="seed">Seed used: <code>{seed}</code> — resubmit with this seed to reproduce these rows.</p>
<button type="button" id="copy-all" class="copy-all">Copy all as JSON</button>
<script type="application/json" id="results-json">{json_escaped}</script>
<div class="table-wrap">
<table>
<thead><tr><th>First name</th><th>Last name</th><th>Gender</th><th>Date of birth</th><th>PESEL</th><th>NIP</th></tr></thead>
<tbody>{rows}</tbody>
</table>
</div>
</section>"#
    )
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
