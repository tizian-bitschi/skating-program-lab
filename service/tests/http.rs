mod support;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;
#[tokio::test]
async fn six_profiles_and_twenty_forms_are_real_and_unknown_context_is_rejected() {
    let app = skating_planning_service::app();
    let response = app
        .clone()
        .oneshot(
            Request::get("/api/v1/profiles")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let profiles: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(profiles.as_array().unwrap().len(), 6);
    for profile in profiles.as_array().unwrap() {
        let request = json!({"requestId":"test", "revision":1,"context":profile["context"],"personalLimits":{}});
        let response = app
            .clone()
            .oneshot(
                Request::post("/api/v1/repertoire")
                    .header("content-type", "application/json")
                    .body(Body::from(request.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let repertoire: Value =
            serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(
            repertoire["forms"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|f| f["family"] == "spin")
                .count(),
            20
        );
        assert!(repertoire["elements"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["code"] == "1T"));
    }
    let response=app.oneshot(Request::post("/api/v1/repertoire").header("content-type","application/json").body(Body::from(json!({"requestId":"bad","revision":1,"context":{"profile":"missing","ruleEdition":"none","scoringTable":"none"},"personalLimits":{}}).to_string())).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn off_basic_and_omitted_maxima_are_distinct_and_v_levels_share_the_form() {
    let app = skating_planning_service::app();
    let context = json!({"profile":"isu-adult-2026-27-silver","ruleEdition":"2026-07-16","scoringTable":"isu-2786-2026-06-02"});
    for (limit, expected, absent) in [
        (json!({"enabled":true,"maximum":0}), "CSpB", "CSp1"),
        (json!({"enabled":true}), "CSp1", "invalid"),
        (json!({"enabled":false}), "1T", "CSpB"),
    ] {
        let request = json!({"requestId":"limits","revision":8,"context":context,"personalLimits":{"spins":{"CSp":limit}}});
        let response = app
            .clone()
            .oneshot(
                Request::post("/api/v1/repertoire")
                    .header("content-type", "application/json")
                    .body(Body::from(request.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body: Value =
            serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        let codes: Vec<_> = body["elements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["code"].as_str().unwrap())
            .collect();
        assert!(
            codes.contains(&expected),
            "expected {expected}, actual {codes:?}"
        );
        assert!(!codes.contains(&absent));
        if limit["maximum"] == 0 {
            assert!(!codes.contains(&"CSp1V"));
        }
        assert_eq!(body["revision"], 8);
    }
    for bad in [
        json!({"enabled":true,"maximum":5}),
        json!({"enabled":true,"maximum":-1}),
        json!({"enabled":true,"unknown":3}),
    ] {
        let request = json!({"requestId":"invalid","revision":9,"context":context,"personalLimits":{"spins":{"CSp":bad}}});
        let response = app
            .clone()
            .oneshot(
                Request::post("/api/v1/repertoire")
                    .header("content-type", "application/json")
                    .body(Body::from(request.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(matches!(
            response.status(),
            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY
        ));
    }
}

#[tokio::test]
async fn independently_registered_policy_is_advertised_and_controls_eligibility() {
    let catalog = support::catalog();
    let profile = catalog.profiles.last().unwrap().clone();
    let app = skating_planning_service::app_with_catalog(catalog);
    let request = json!({"requestId":"registered","revision":1,"context":skating_planning_service::Context::for_profile(&profile),"personalLimits":{}});
    let response = app
        .oneshot(
            Request::post("/api/v1/repertoire")
                .header("content-type", "application/json")
                .body(Body::from(request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert!(body["elements"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["code"] == "1Lz"));
    assert!(!body["elements"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["code"] == "1T"));
    assert!(body["forms"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["id"] == "T" && f["available"] == false));
}

#[tokio::test]
async fn original_source_urls_match_exact_document_editions() {
    let mut catalog = support::catalog();
    let custom = catalog.profiles.last_mut().unwrap();
    custom.sources.push(skating_program_lab::SourceReference {
        document: "ISU Communication 2786".into(),
        version: "custom-edition".into(),
        location: "custom location".into(),
    });
    let response = skating_planning_service::app_with_catalog(catalog)
        .oneshot(
            Request::get("/api/v1/profiles")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let profiles: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let sources = profiles
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|p| p["sources"].as_array().unwrap())
        .collect::<Vec<_>>();
    let scale = sources
        .iter()
        .find(|s| s["document"] == "ISU Communication 2786" && s["version"] == "2026-06-02")
        .unwrap();
    assert_eq!(scale["url"], "https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Governance/Transparency/ISU-Communications/2786-SinglesPairs-SOV-Update-260602-1780481099-6311.pdf");
    let custom = sources
        .iter()
        .find(|s| s["version"] == "custom-edition")
        .unwrap();
    assert_eq!(custom["location"], "custom location");
    assert!(custom.get("url").is_none());
}
