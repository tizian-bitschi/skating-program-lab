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
    use skating_program_lab::*;
    let mut catalog = skating_planning_service::Catalog::default();
    let mut profile = adult_profile(AdultCategory::Bronze);
    profile.id.name = "independent-http-policy".into();
    profile.category = "Independent HTTP test policy".into();
    profile.provenance = ProfileProvenance::Synthetic;
    profile
        .jumps
        .allowed
        .retain(|permission| permission.kind == JumpKind::Lutz);
    let mut policy = adult_program_policy(AdultCategory::Bronze);
    policy.spins.profile = profile.id.clone();
    policy.spins.source = profile.sources[0].clone();
    profile.sources.extend([
        policy.step_source.clone(),
        policy.choreography_source.clone(),
        policy.feature_source.clone(),
    ]);
    profile
        .sources
        .extend(policy.spins.positions.iter().map(|p| p.source.clone()));
    profile
        .sources
        .extend(policy.spins.per_foot.iter().map(|p| p.source.clone()));
    catalog
        .registry
        .register_with_program_policy(profile.clone(), policy)
        .unwrap();
    catalog.profiles.push(profile.clone());
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
