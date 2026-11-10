mod support;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;
async fn call(
    app: &axum::Router,
    method: &str,
    path: &str,
    body: Option<Value>,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(path);
    if let Some(t) = token {
        req = req.header("authorization", format!("Bearer {t}"));
    }
    if body.is_some() {
        req = req.header("content-type", "application/json");
    }
    let response = app
        .clone()
        .oneshot(
            req.body(body.map_or(Body::empty(), |v| Body::from(v.to_string())))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

// Independent literals from tests/backend_acceptance.rs (official Adult
// announcement, Rule 612 and SOV 2786); these are not captured solver results.
// Synthetic Lutz-only Bronze: two 1Lz (120) + two distinct change-foot
// level-one spins (480) + ChSq1 (350) = 950 in every workflow.
#[tokio::test]
async fn seven_profiles_four_workflows_prove_literal_bounds_at_public_http() {
    let app = skating_planning_service::app_with_catalog_and_search_options(
        support::catalog(),
        skating_planning_service::SearchServiceOptions {
            capacity: 1,
            ..Default::default()
        },
    );
    let (_, profiles) = call(&app, "GET", "/api/v1/profiles", None, None).await;
    let (_, identity) = call(&app, "GET", "/api/v1/searches/identity", None, None).await;
    let bounds = [
        [1100, 1100, 1100, 1100],
        [1480, 1470, 1480, 1470],
        [2420, 2420, 2280, 2280],
        [3340, 3280, 3340, 3280],
        [13310, 12020, 13310, 12020],
        [13310, 12020, 13310, 12020],
        [950, 950, 950, 950],
    ];
    for (index, expected) in bounds.into_iter().enumerate() {
        let base = json!({"requestId":"workflow-matrix","revision":1,"context":profiles[index]["context"],"personalLimits":{}});
        let (status, repertoire) =
            call(&app, "POST", "/api/v1/repertoire", Some(base.clone()), None).await;
        assert_eq!(status, StatusCode::OK, "{repertoire}");
        let required_codes: &[&str] = match index {
            0 => &["1F"],
            1 => &["1F+1Lo", "CSSp2"],
            6 => &["1Lz"],
            _ => &["1A"],
        };
        let mut required = Vec::new();
        for (n, code) in required_codes.iter().enumerate() {
            let key = if code.contains('+') {
                let mut query = base.clone();
                query["prefix"] = json!(code.split('+').collect::<Vec<_>>());
                let (status, options) = call(
                    &app,
                    "POST",
                    "/api/v1/connections/options",
                    Some(query),
                    None,
                )
                .await;
                assert_eq!(status, StatusCode::OK, "{options}");
                options["completions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|e| e["code"] == *code)
                    .unwrap()["key"]
                    .clone()
            } else {
                repertoire["elements"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|e| e["code"] == *code)
                    .unwrap()["key"]
                    .clone()
            };
            required.push(json!({"id":format!("kept-{n}"),"elementKey":key}));
        }
        for (workflow, bound) in expected.into_iter().enumerate() {
            let mut request = base.clone();
            request["creationKey"] = json!(format!(
                "{}:{:064x}",
                identity["creationNamespace"].as_str().unwrap(),
                index * 4 + workflow + 1
            ));
            request["requiredElements"] = if workflow % 2 == 1 {
                json!(required)
            } else {
                json!([])
            };
            if workflow >= 2 {
                request["personalLimits"] = json!({"jumps":{"T":{"enabled":false}}});
            }
            // Capacity 1 proves the previous DELETE released the actual owner,
            // rather than only its HTTP lookup entry.
            let start = std::time::Instant::now();
            let owner = loop {
                let (status, owner) = call(
                    &app,
                    "POST",
                    "/api/v1/searches",
                    Some(request.clone()),
                    None,
                )
                .await;
                if status == StatusCode::ACCEPTED {
                    break owner;
                }
                assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{owner}");
                assert!(start.elapsed().as_secs() < 30, "owner failed to terminate");
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            };
            let id = owner["id"].as_str().unwrap();
            let token = owner["accessToken"].as_str().unwrap();
            let path = format!("/api/v1/searches/{id}/updates?after=0&limit=1");
            let deadline = std::time::Instant::now();
            let result = loop {
                let (status, result) = call(&app, "GET", &path, None, Some(token)).await;
                assert_eq!(status, StatusCode::OK, "{result}");
                assert_ne!(
                    result["status"], "failed",
                    "profile {index} workflow {workflow}: {result}"
                );
                if !result["results"].as_array().unwrap().is_empty() {
                    break result;
                }
                assert!(
                    deadline.elapsed().as_secs() < 180,
                    "test harness wait exceeded: {index}/{workflow}: {result}"
                );
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            };
            assert_eq!(result["proof"], "proved");
            assert_eq!(
                result["maximumBaseHundredths"], bound,
                "profile {index} workflow {workflow}"
            );
            let row = &result["results"][0];
            let entries = row["entries"].as_array().unwrap();
            if workflow % 2 == 1 {
                for input in &required {
                    assert!(
                        entries.iter().any(|e| e["requiredId"] == input["id"]
                            && e["elementKey"] == input["elementKey"]),
                        "{row}"
                    );
                }
            }
            if workflow >= 2 {
                assert!(entries.iter().all(|e| !e["code"]
                    .as_str()
                    .unwrap()
                    .split('+')
                    .any(|p| p.ends_with('T'))));
            }
            let mut manual = base.clone();
            manual["personalLimits"] = request["personalLimits"].clone();
            manual["submission"] = json!("complete");
            manual["entries"] = json!(entries
                .iter()
                .enumerate()
                .map(|(n, e)| json!({"id":format!("manual-{n}"),"elementKey":e["elementKey"]}))
                .collect::<Vec<_>>());
            let (status, assessment) = call(
                &app,
                "POST",
                "/api/v1/program-assessments",
                Some(manual),
                None,
            )
            .await;
            assert_eq!(status, StatusCode::OK, "{assessment}");
            assert_eq!(assessment["totalHundredths"], bound);
            assert_eq!(assessment["construction"], "compliant");
            assert!(assessment["personalConflicts"]
                .as_array()
                .unwrap()
                .is_empty());
            let (_, repeated) = call(&app, "GET", &path, None, Some(token)).await;
            assert_eq!(repeated["results"][0], *row);
            assert_eq!(
                call(
                    &app,
                    "DELETE",
                    &format!("/api/v1/searches/{id}"),
                    None,
                    Some(token)
                )
                .await
                .0,
                StatusCode::OK
            );
            assert_eq!(
                call(&app, "GET", &path, None, Some(token)).await.0,
                StatusCode::NOT_FOUND
            );
        }
    }
}
