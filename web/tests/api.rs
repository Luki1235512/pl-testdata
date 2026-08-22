use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;
use web::routes::router;

async fn body_json(response: Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn json_post(uri: &str, payload: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap()
}

#[tokio::test]
async fn health_returns_ok() {
    let response = router()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn index_page_carries_the_synthetic_data_footer_notice() {
    let response = router()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("fictitious and not linked to real people"));
}

#[tokio::test]
async fn same_seed_produces_byte_identical_people_across_two_requests() {
    let payload = json!({ "seed": 42, "count": 5 });

    let response_a = router()
        .oneshot(json_post("/api/v1/persons", payload.clone()))
        .await
        .unwrap();
    let response_b = router()
        .oneshot(json_post("/api/v1/persons", payload))
        .await
        .unwrap();

    assert_eq!(response_a.status(), StatusCode::OK);
    assert_eq!(response_b.status(), StatusCode::OK);

    let json_a = body_json(response_a).await;
    let json_b = body_json(response_b).await;

    assert_eq!(json_a["resolved_seed"], 42);
    assert_eq!(json_a["people"], json_b["people"]);
}

#[tokio::test]
async fn omitted_seed_is_still_reported_back_for_reproducibility() {
    let response = router()
        .oneshot(json_post("/api/v1/persons", json!({})))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert!(json["resolved_seed"].is_u64());
}

#[tokio::test]
async fn invalid_date_range_returns_400_with_the_domain_error_reason() {
    let payload = json!({ "min_date": "2000-01-01", "max_date": "1990-01-01" });

    let response = router()
        .oneshot(json_post("/api/v1/persons", payload))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("invalid date range")
    );
}

#[tokio::test]
async fn malformed_date_returns_400_naming_the_offending_field() {
    let payload = json!({ "min_date": "not-a-date", "max_date": "2000-01-01" });

    let response = router()
        .oneshot(json_post("/api/v1/persons", payload))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert!(json["error"].as_str().unwrap().contains("min_date"));
}

#[tokio::test]
async fn gender_constraint_is_respected_for_every_generated_person() {
    let payload = json!({ "gender": "female", "seed": 7, "count": 20 });

    let response = router()
        .oneshot(json_post("/api/v1/persons", payload))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    for person in json["people"].as_array().unwrap() {
        assert_eq!(person["gender"], "Female");
    }
}

#[tokio::test]
async fn count_above_the_cap_is_rejected() {
    let response = router()
        .oneshot(json_post("/api/v1/persons", json!({ "count": 255 })))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn count_of_zero_is_rejected() {
    let response = router()
        .oneshot(json_post("/api/v1/persons", json!({ "count": 0 })))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn html_form_submission_renders_a_results_table() {
    let body = "gender=&min_date=&max_date=&seed=99&count=3";
    let request = Request::builder()
        .method("POST")
        .uri("/generate")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();

    let response = router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("<table>"));
    assert!(html.contains("Seed used: <code>99</code>"));
    assert!(html.contains("<th>NIP</th>"));
}

#[tokio::test]
async fn every_generated_person_carries_a_valid_nip() {
    let payload = json!({ "seed": 5, "count": 10 });

    let response = router()
        .oneshot(json_post("/api/v1/persons", payload))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    for person in json["people"].as_array().unwrap() {
        let nip = person["nip"]
            .as_str()
            .expect("nip should always be present");
        assert_eq!(nip.len(), 10);
        assert!(nip.chars().all(|c| c.is_ascii_digit()));
    }
}

#[tokio::test]
async fn only_min_date_no_longer_errors_and_uses_a_default_upper_bound() {
    let body = "gender=&min_date=2000-01-01&max_date=&seed=1&count=5";
    let request = Request::builder()
        .method("POST")
        .uri("/generate")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();

    let response = router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("<table>"));
}

#[tokio::test]
async fn only_max_date_no_longer_errors_and_uses_a_default_lower_bound() {
    let body = "gender=&min_date=&max_date=2005-12-31&seed=1&count=5";
    let request = Request::builder()
        .method("POST")
        .uri("/generate")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();

    let response = router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn invalid_date_range_from_the_html_form_renders_an_html_error_not_json() {
    let body = "gender=&min_date=2020-01-01&max_date=1990-01-01&seed=1&count=1";
    let request = Request::builder()
        .method("POST")
        .uri("/generate")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();

    let response = router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(content_type.starts_with("text/html"));

    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("invalid date range"));
    assert!(html.contains(r#"class="error""#));
}

#[tokio::test]
async fn submitted_form_values_are_redisplayed_after_generating() {
    let body = "gender=female&min_date=1990-06-01&max_date=1999-12-31&seed=42&count=3";
    let request = Request::builder()
        .method("POST")
        .uri("/generate")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();

    let response = router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(bytes.to_vec()).unwrap();

    assert!(html.contains(r#"value="1990-06-01""#));
    assert!(html.contains(r#"value="1999-12-31""#));
    assert!(html.contains(r#"value="42""#));
    assert!(html.contains(r#"value="3""#));
    assert!(html.contains(r#"value="female" selected"#));
}

#[tokio::test]
async fn blank_seed_field_stays_blank_after_generating() {
    let body = "gender=&min_date=&max_date=&seed=&count=3";
    let request = Request::builder()
        .method("POST")
        .uri("/generate")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();

    let response = router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(bytes.to_vec()).unwrap();

    assert!(html.contains(r#"id="seed" name="seed" min="0" value=""#));
}

#[tokio::test]
async fn malformed_iso_date_from_the_html_form_returns_400_naming_the_field() {
    let body = "gender=&min_date=1990-13-40&max_date=&seed=1&count=1";
    let request = Request::builder()
        .method("POST")
        .uri("/generate")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();

    let response = router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("min_date"));
}

#[tokio::test]
async fn styles_css_is_served_with_the_right_content_type() {
    let response = router()
        .oneshot(
            Request::builder()
                .uri("/styles.css")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_type.starts_with("text/css"));
}

#[tokio::test]
async fn index_page_links_the_external_stylesheet() {
    let response = router()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains(r#"href="/styles.css""#));
}
