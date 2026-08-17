use crate::dto::{GenerateForm, PersonDto, SYNTHETIC_DATA_DISCLAIMER};

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
    let person_results_html = match ctx.person_results {
        Some((people, seed)) => result_section(people, seed),
        None => String::new(),
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="description" content="Synthetic Polish test-person and PESEL/NIP generator for QA engineers.">
<link rel="icon" href="data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%22.9em%22 font-size=%2290%22>🧪</text></svg>">
<title>pl-testdata - Polish test-person generator</title>
<style>{CSS}</style>
</head>
<body>
<header>
  <h1>pl-testdata</h1>
  <p class="disclaimer" role="note">⚠️ {SYNTHETIC_DATA_DISCLAIMER}</p>
</header>
<main>
{error_html}
{form_html}
{person_results_html}
<script>
document.querySelectorAll('.copy').forEach((btn) => {{
  btn.addEventListener('click', () => navigator.clipboard.writeText(btn.dataset.copy));
}});
const copyAllBtn = document.getElementById('copy-all');
if (copyAllBtn) {{
  copyAllBtn.addEventListener('click', () => {{
    const json = document.getElementById('results-json').textContent;
    navigator.clipboard.writeText(json);
  }});
}}
</script>
</body>
</html>"#
    )
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

    <label for="gender">Gender</label>
    <select id="gender" name="gender">
      <option value="" {any_sel}>Any</option>
      <option value="male" {male_sel}>Male</option>
      <option value="female" {female_sel}>Female</option>
    </select>

    <label for="min_date">Born after</label>
    <input type="date" id="min_date" name="min_date" value="{min_date}">

    <label for="max_date">Born before</label>
    <input type="date" id="max_date" name="max_date" value="{max_date}">

    <label for="seed">Seed (optional - reuse it to reproduce the same output)</label>
    <input type="number" id="seed" name="seed" min="0" value="{seed}">

    <label for="count">How many</label>
    <input type="number" id="count" name="count" min="1" max="50" value="{count_value}">

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
            format!(
                r#"<tr>
  <td>{}</td><td>{}</td><td>{}</td><td>{}</td>
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
        r#"<p class="seed">Seed used: <code>{seed}</code> - resubmit with this seed to reproduce these rows.</p>
<button type="button" id="copy-all" class="copy-all">Copy all as JSON</button>
<script type="application/json" id="results-json">{json_escaped}</script>
<div class="table-wrap">
<table>
<thead><tr><th>First name</th><th>Last name</th><th>Gender</th><th>Date of birth</th><th>PESEL</th><th>NIP</th></tr></thead>
<tbody>{rows}</tbody>
</table>
</div>"#
    )
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const CSS: &str = "\
body{font-family:system-ui,sans-serif;max-width:720px;margin:2rem auto;padding:0 1rem;color:#1a1a1a}
.disclaimer{background:#fff3cd;border:1px solid #ffe69c;padding:.5rem 1rem;border-radius:.25rem;font-weight:600}
.error{background:#fde2e2;border:1px solid #f5b5b5;padding:.5rem 1rem;border-radius:.25rem;font-weight:600;color:#7a1f1f}
fieldset{border:1px solid #ccc;border-radius:.5rem;padding:1rem;display:grid;gap:.5rem;max-width:320px;margin:0 auto}
label{font-size:.85rem;font-weight:600}
.table-wrap{overflow-x:auto;margin-top:1rem}
table{width:100%;border-collapse:collapse}
th,td{border-bottom:1px solid #ddd;padding:.5rem;text-align:left;white-space:nowrap}
.seed{color:#555;font-size:.9rem;margin-top:1rem}
.copy-all{margin-top:.5rem}
button{cursor:pointer}
@media (max-width: 480px){
  fieldset{max-width:100%}
}";
