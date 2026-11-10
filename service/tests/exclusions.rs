use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;
async fn call(app: &axum::Router, path: &str, body: Option<Value>) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(match body {
            Some(body) => Request::post(path)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
            None => Request::get(path).body(Body::empty()).unwrap(),
        })
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}
#[tokio::test]
async fn searches_reject_spin_solo_and_invalid_assumption_exclusion_holes_before_admission() {
    let app = skating_planning_service::app();
    let (_, identity) = call(&app, "/api/v1/searches/identity", None).await;
    let context = json!({"profile":"isu-adult-2026-27-gold","ruleEdition":"2026-07-16","scoringTable":"isu-2786-2026-06-02"});
    for code in ["SSp1", "1Lz", "1Lz+SSp1", "1Lz+1A+SEQ"] {
        let body = json!({"creationKey":format!("{}:{}",identity["creationNamespace"].as_str().unwrap(),"b".repeat(64)),"requestId":"exclusion-hole","revision":87,"context":context,"personalLimits":{},"requiredElements":[],"excludedConnections":[format!("isu-adult-2026-27-gold:2026-07-16:isu-2786-2026-06-02:{code}:false")]});
        let (status, error) = call(&app, "/api/v1/searches", Some(body)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{error}");
        assert_eq!(error["code"], "invalid_connection_exclusion");
        assert_eq!(error["field"], "excludedConnections");
        assert_eq!(error["requestId"], "exclusion-hole");
        assert_eq!(error["revision"], 87);
    }
}
