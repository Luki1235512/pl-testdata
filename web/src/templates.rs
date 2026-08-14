use crate::dto::{PersonDto, SYNTHETIC_DATA_DISCLAIMER};

pub fn page(results: Option<(&[PersonDto], u64)>) -> String {
    let results_html = match results {
        Some((people, seed)) => result_section(people, seed),
        None => String::new(),
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>pl-testdata - Polish test-person generator</title>
<style>{CSS}</style>
</head>
<body>
<header>
  <h1>pl-testdata</h1>
  <p class="disclaimer" role="note">⚠️ {SYNTHETIC_DATA_DISCLAIMER}</p>
</header>
<main>
{FORM}
{results_html}
</main>
</body>
</html>"#
    )
}

const FORM: &str = r#"<form method="post" action="/generate">
  <fieldset>
    <legend>Generation options</legend>

    <label for="gender">Gender</label>
    <select id="gender" name="gender">
      <option value="">Any</option>
      <option value="male">Male</option>
      <option value="female">Female</option>
    </select>

    <label for="min_date">Born after</label>
    <input type="date" id="min_date" name="min_date">

    <label for="max_date">Born before</label>
    <input type="date" id="max_date" name="max_date">

    <label for="seed">Seed (optional - reuse it to reproduce the same output)</label>
    <input type="number" id="seed" name="seed" min="0">

    <label for="count">How many</label>
    <input type="number" id="count" name="count" min="1" max="50" value="1">

    <button type="submit">Generate</button>
  </fieldset>
</form>"#;

fn result_section(people: &[PersonDto], seed: u64) -> String {
    let rows: String = people
        .iter()
        .map(|p| {
            format!(
                r#"<tr>
  <td>{}</td><td>{}</td><td>{}</td><td>{}</td>
  <td><code>{}</code> <button type="button" class="copy" data-copy="{}">Copy</button></td>
</tr>"#,
                escape(&p.first_name),
                escape(&p.last_name),
                escape(&p.gender),
                escape(&p.date_of_birth),
                escape(&p.pesel),
                escape(&p.pesel),
            )
        })
        .collect();

    format!(
        r#"<p class="seed">Seed used: <code>{seed}</code> - resubmit with this seed to reproduce these rows.</p>
<table>
<thead><tr><th>First name</th><th>Last name</th><th>Gender</th><th>Date of birth</th><th>PESEL</th></tr></thead>
<tbody>{rows}</tbody>
</table>
<script>
document.querySelectorAll('.copy').forEach((btn) => {{
  btn.addEventListener('click', () => navigator.clipboard.writeText(btn.dataset.copy));
}});
</script>"#
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
fieldset{border:1px solid #ccc;border-radius:.5rem;padding:1rem;display:grid;gap:.5rem;max-width:320px}
label{font-size:.85rem;font-weight:600}
table{width:100%;border-collapse:collapse;margin-top:1rem}
th,td{border-bottom:1px solid #ddd;padding:.5rem;text-align:left}
.seed{color:#555;font-size:.9rem}
button{cursor:pointer}";
