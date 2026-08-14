use axum::extract::Form;
use axum::response::Html;
use axum::routing::{get, post};
use axum::{Json, Router};
use domain::DateOfBirth;
use domain::person::{Person, PersonConstraints, generate_person};
use rand::RngExt;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::dto::{
    GenerateForm, GenerateRequest, GenerateResponse, PersonDto, SYNTHETIC_DATA_DISCLAIMER,
};
use crate::error::ApiError;
use crate::templates;

const MAX_COUNT: u8 = 50;

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/", get(index))
        .route("/generate", post(html_generate))
        .route("/api/v1/persons", post(api_generate))
}

async fn health() -> &'static str {
    "ok"
}

async fn index() -> Html<String> {
    Html(templates::page(None))
}

async fn html_generate(Form(form): Form<GenerateForm>) -> Result<Html<String>, ApiError> {
    let request = form.into_request()?;
    let (people, resolved_seed) = generate_people(&request)?;
    let dtos: Vec<PersonDto> = people.into_iter().map(PersonDto::from).collect();
    Ok(Html(templates::page(Some((&dtos, resolved_seed)))))
}

async fn api_generate(
    Json(request): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, ApiError> {
    let (people, resolved_seed) = generate_people(&request)?;
    let people = people.into_iter().map(PersonDto::from).collect();
    Ok(Json(GenerateResponse {
        disclaimer: SYNTHETIC_DATA_DISCLAIMER,
        resolved_seed,
        people,
    }))
}

fn generate_people(request: &GenerateRequest) -> Result<(Vec<Person>, u64), ApiError> {
    let count = request.count.unwrap_or(1);
    if count == 0 || count > MAX_COUNT {
        return Err(ApiError::InvalidCount {
            count,
            max: MAX_COUNT,
        });
    }

    let date_range = match (&request.min_date, &request.max_date) {
        (Some(min), Some(max)) => Some((
            parse_iso_date("min_date", min)?,
            parse_iso_date("max_date", max)?,
        )),
        (None, None) => None,
        (Some(_), None) => {
            return Err(ApiError::InvalidField {
                field: "max_date",
                value: "missing".into(),
            });
        }
        (None, Some(_)) => {
            return Err(ApiError::InvalidField {
                field: "min_date",
                value: "missing".into(),
            });
        }
    };

    let constraints = PersonConstraints {
        gender: request.gender.map(Into::into),
        date_range,
    };

    let resolved_seed = request
        .seed
        .unwrap_or_else(|| rand::rng().random_range(0..=u64::MAX));
    let mut rng = StdRng::seed_from_u64(resolved_seed);

    let people = (0..count)
        .map(|_| generate_person(&mut rng, &constraints))
        .collect::<Result<Vec<Person>, _>>()?;

    Ok((people, resolved_seed))
}

fn parse_iso_date(field: &'static str, value: &str) -> Result<DateOfBirth, ApiError> {
    let invalid = || ApiError::InvalidField {
        field,
        value: value.to_string(),
    };

    let parts: Vec<&str> = value.split('-').collect();
    let [y, m, d] = parts.as_slice() else {
        return Err(invalid());
    };

    let year = y.parse::<i32>().map_err(|_| invalid())?;
    let month = m.parse::<u32>().map_err(|_| invalid())?;
    let day = d.parse::<u32>().map_err(|_| invalid())?;

    DateOfBirth::new(year, month, day).map_err(|_| invalid())
}
