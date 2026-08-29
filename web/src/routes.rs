use axum::extract::Form;
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use domain::DateOfBirth;
use domain::address::generate_address;
use domain::generate::default_date_range;
use domain::nip::generate_nip;
use domain::person::{PersonConstraints, generate_person};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

use crate::dto::{
    GenerateForm, GenerateRequest, GenerateResponse, PersonDto, SYNTHETIC_DATA_DISCLAIMER,
};
use crate::error::ApiError;
use crate::templates;

const MAX_COUNT: u8 = 50;
const STYLES_CSS: &str = include_str!("../assets/styles.css");

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/", get(index))
        .route("/styles.css", get(styles))
        .route("/generate", post(html_generate))
        .route("/api/v1/persons", post(api_generate))
}

async fn health() -> &'static str {
    "ok"
}

async fn index() -> Html<String> {
    Html(templates::page(templates::PageContext {
        person_results: None,
        submitted_form: None,
        error: None,
    }))
}

async fn styles() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        STYLES_CSS,
    )
}

async fn html_generate(Form(form): Form<GenerateForm>) -> Response {
    let request = match form.clone().into_request() {
        Ok(request) => request,
        Err(form_err) => {
            let api_err: ApiError = form_err.into();
            return (
                StatusCode::BAD_REQUEST,
                Html(templates::page(templates::PageContext {
                    person_results: None,
                    submitted_form: Some(&form),
                    error: Some(api_err.message()),
                })),
            )
                .into_response();
        }
    };

    match generate_people(&request) {
        Ok((dtos, resolved_seed)) => Html(templates::page(templates::PageContext {
            person_results: Some((&dtos, resolved_seed)),
            submitted_form: Some(&form),
            error: None,
        }))
        .into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Html(templates::page(templates::PageContext {
                person_results: None,
                submitted_form: Some(&form),
                error: Some(err.message()),
            })),
        )
            .into_response(),
    }
}

async fn api_generate(
    Json(request): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, ApiError> {
    let (people, resolved_seed) = generate_people(&request)?;
    Ok(Json(GenerateResponse {
        disclaimer: SYNTHETIC_DATA_DISCLAIMER,
        resolved_seed,
        people,
    }))
}

fn generate_people(request: &GenerateRequest) -> Result<(Vec<PersonDto>, u64), ApiError> {
    let count = request.count.unwrap_or(1);
    if count == 0 || count > MAX_COUNT {
        return Err(ApiError::InvalidCount {
            count,
            max: MAX_COUNT,
        });
    }

    let (default_min, default_max) = default_date_range();
    let date_range = match (&request.min_date, &request.max_date) {
        (Some(min), Some(max)) => Some((
            parse_iso_date("min_date", min)?,
            parse_iso_date("max_date", max)?,
        )),
        (Some(min), None) => Some((parse_iso_date("min_date", min)?, default_max)),
        (None, Some(max)) => Some((default_min, parse_iso_date("max_date", max)?)),
        (None, None) => None,
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
        .map(|_| {
            let person = generate_person(&mut rng, &constraints)?;
            let nip = generate_nip(&mut rng);
            let address = generate_address(&mut rng);
            Ok(PersonDto::new(person, nip, address))
        })
        .collect::<Result<Vec<PersonDto>, ApiError>>()?;

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
