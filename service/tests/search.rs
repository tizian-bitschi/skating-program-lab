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
#[tokio::test]
async fn proved_program_is_assessed_and_required_occurrences_survive_http() {
    let app = skating_planning_service::app();
    let (_, identity) = call(&app, "GET", "/api/v1/searches/identity", None, None).await;
    let (_, profiles) = call(&app, "GET", "/api/v1/profiles", None, None).await;
    let context = &profiles[0]["context"];
    let key = |code: &str| {
        format!(
            "{}:{}:{}:{code}:false",
            context["profile"].as_str().unwrap(),
            context["ruleEdition"].as_str().unwrap(),
            context["scoringTable"].as_str().unwrap()
        )
    };
    let required = vec![
        json!({"id":"spin-a","elementKey":key("CCSp1")}),
        json!({"id":"spin-b","elementKey":key("CLSp1")}),
        json!({"id":"choreo","elementKey":key("ChSq1")}),
    ];
    let request = json!({"creationKey":format!("{}:{}",identity["creationNamespace"].as_str().unwrap(),"a".repeat(64)),"requestId":"http-proof","revision":7,"context":context,"requiredElements":required,"allowMixedDirections":true,"personalLimits":{"jumps":{"T":{"enabled":false},"S":{"enabled":false},"Lz":{"enabled":false},"A":{"enabled":false}},"unlevelledElements":{"W":false}}});
    let (status, created) = call(
        &app,
        "POST",
        "/api/v1/searches",
        Some(request.clone()),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED, "{created}");
    let (_, retry) = call(
        &app,
        "POST",
        "/api/v1/searches",
        Some(request.clone()),
        None,
    )
    .await;
    assert_eq!(created["id"], retry["id"]);
    assert_eq!(created["accessToken"], retry["accessToken"]);
    let id = created["id"].as_str().unwrap();
    let token = created["accessToken"].as_str().unwrap();
    let path = format!("/api/v1/searches/{id}/updates?after=0&limit=100");
    assert_eq!(
        call(&app, "GET", &path, None, None).await.0,
        StatusCode::NOT_FOUND
    );
    let start = std::time::Instant::now();
    let result = loop {
        let (_, v) = call(&app, "GET", &path, None, Some(token)).await;
        if v["maximumBaseHundredths"].is_null() {
            assert!(v["results"].as_array().unwrap().is_empty());
        }
        if v["status"] == "completed" {
            break v;
        }
        assert_ne!(v["status"], "failed", "{v}");
        assert!(start.elapsed().as_secs() < 30, "{v}");
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    assert_eq!(result["maximumBaseHundredths"], 1030);
    assert_eq!(result["confirmedCount"], 5);
    let programs = result["results"].as_array().unwrap();
    assert_eq!(programs.len(), 5);
    let actual = programs
        .iter()
        .map(|p| {
            let mut codes = p["entries"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|e| e["code"].as_str())
                .filter(|c| c.starts_with('1'))
                .map(str::to_owned)
                .collect::<Vec<_>>();
            codes.sort();
            codes
        })
        .collect::<std::collections::BTreeSet<_>>();
    let expected = [
        vec!["1F", "1F", "1Lo", "1Lo"],
        vec!["1F+1F", "1Lo", "1Lo"],
        vec!["1F", "1F+1Lo", "1Lo"],
        vec!["1F", "1Lo", "1Lo+1F"],
        vec!["1F", "1F", "1Lo+1Lo"],
    ]
    .into_iter()
    .map(|v| v.into_iter().map(str::to_owned).collect())
    .collect();
    assert_eq!(actual, expected);

    for p in programs {
        assert_eq!(p["assessment"]["construction"], "compliant");
        assert_eq!(p["assessment"]["totalHundredths"], 1030);
        for (id, code) in [
            ("spin-a", "CCSp1"),
            ("spin-b", "CLSp1"),
            ("choreo", "ChSq1"),
        ] {
            assert!(p["entries"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["requiredId"] == id && e["code"] == code));
        }
    }
    assert_eq!(
        call(&app, "GET", &path, None, Some(token)).await.1["results"],
        result["results"]
    );
    let single = format!("/api/v1/searches/{id}/updates?after=0&limit=1");
    let first = call(&app, "GET", &single, None, Some(token)).await.1;
    assert_eq!(first["results"].as_array().unwrap().len(), 1);
    assert_eq!(first["cursor"], 1);
    assert_eq!(first["results"][0], result["results"][0]);
    let status_only = call(
        &app,
        "GET",
        &format!("/api/v1/searches/{id}/updates?after=1&limit=0"),
        None,
        Some(token),
    )
    .await
    .1;
    assert_eq!(status_only["cursor"], 1);
    assert!(status_only["results"].as_array().unwrap().is_empty());
    assert_eq!(status_only["confirmedCount"], 5);
    assert_eq!(
        call(
            &app,
            "GET",
            &format!("/api/v1/searches/{id}/updates?after=999"),
            None,
            Some(token)
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let mut conflict = request.clone();
    conflict["revision"] = json!(8);
    let (status, v) = call(&app, "POST", "/api/v1/searches", Some(conflict), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(v["code"], "creation_conflict");
    assert_eq!(v["revision"], 8);

    let (_, cancel) = call(
        &app,
        "POST",
        &format!("/api/v1/searches/{id}/cancel"),
        Some(json!({"commandId":"cancel-1"})),
        Some(token),
    )
    .await;
    assert_eq!(cancel["maximumBaseHundredths"], 1030);
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
    // A retained whole remains indivisible; excluding the inverse whole leaves
    // component jumps available. Independent stock bound is still 10.30.
    let mut whole = request.clone();
    whole["creationKey"] = json!(format!(
        "{}:{}",
        identity["creationNamespace"].as_str().unwrap(),
        "b".repeat(64)
    ));
    whole["requiredElements"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"whole","elementKey":key("1Lo+1F")}));
    whole["excludedConnections"] = json!([key("1F+1Lo")]);
    whole["personalLimits"]["jumps"]["F"] = json!({"enabled":true,"maximum":1});
    whole["personalLimits"]["jumps"]["Lo"] = json!({"enabled":true,"maximum":1});
    whole["personalLimits"]["spins"] =
        json!({"CCSp":{"enabled":true,"maximum":1},"CLSp":{"enabled":true,"maximum":1}});
    whole["personalLimits"]["steps"] = json!({"enabled":false});
    whole["personalLimits"]["unlevelledElements"]["ChSp"] = json!(false);
    let (status, owner) = call(&app, "POST", "/api/v1/searches", Some(whole), None).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let result = terminal(&app, &owner).await;
    assert_eq!(result["status"], "completed", "{result}");
    assert_eq!(result["maximumBaseHundredths"], 1030);
    assert_eq!(result["confirmedCount"], 1);
    let rows = result["results"][0]["entries"].as_array().unwrap();
    assert!(rows
        .iter()
        .any(|e| e["requiredId"] == "whole" && e["code"] == "1Lo+1F"));
    let mut codes = rows
        .iter()
        .filter_map(|e| e["code"].as_str())
        .filter(|c| c.starts_with('1'))
        .collect::<Vec<_>>();
    codes.sort();
    assert_eq!(codes, vec!["1F", "1Lo", "1Lo+1F"]);
}
#[derive(Default)]
struct Clock(std::sync::atomic::AtomicU64);
impl skating_planning_service::SearchClock for Clock {
    fn now(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.0.load(std::sync::atomic::Ordering::SeqCst))
    }
}
#[tokio::test]
async fn capacity_and_busy_owner_lease_are_enforced_without_a_solver_deadline() {
    let clock = std::sync::Arc::new(Clock::default());
    let app = skating_planning_service::app_with_catalog_and_search_options(
        skating_planning_service::Catalog::default(),
        skating_planning_service::SearchServiceOptions {
            capacity: 1,
            clock: clock.clone(),
        },
    );
    let (_, identity) = call(&app, "GET", "/api/v1/searches/identity", None, None).await;
    let (_, profiles) = call(&app, "GET", "/api/v1/profiles", None, None).await;
    let request = |key: &str| json!({"creationKey":format!("{}:{}",identity["creationNamespace"].as_str().unwrap(),key.repeat(64)),"requestId":"capacity","revision":3,"context":profiles[5]["context"],"requiredElements":[],"personalLimits":{}});
    let (status, owner) = call(&app, "POST", "/api/v1/searches", Some(request("a")), None).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let (status, denied) = call(&app, "POST", "/api/v1/searches", Some(request("b")), None).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{denied}");
    assert_eq!(denied["code"], "search_capacity");
    clock.0.store(899, std::sync::atomic::Ordering::SeqCst);
    let path = format!(
        "/api/v1/searches/{}/updates?limit=0",
        owner["id"].as_str().unwrap()
    );
    let token = owner["accessToken"].as_str().unwrap();
    assert_eq!(
        call(&app, "GET", &path, None, Some("wrong")).await.0,
        StatusCode::NOT_FOUND
    );
    clock.0.store(901, std::sync::atomic::Ordering::SeqCst);
    assert_eq!(
        call(&app, "GET", &path, None, Some(token)).await.0,
        StatusCode::NOT_FOUND
    );
    let start = std::time::Instant::now();
    loop {
        let (status, _) = call(&app, "POST", "/api/v1/searches", Some(request("b")), None).await;
        if status == StatusCode::ACCEPTED {
            break;
        }
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(start.elapsed().as_secs() < 10);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    // A disposed/expired creation receipt never starts replacement work.
    assert_eq!(
        call(&app, "POST", "/api/v1/searches", Some(request("a")), None)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    let restarted = skating_planning_service::app();
    assert_eq!(
        call(
            &restarted,
            "POST",
            "/api/v1/searches",
            Some(request("c")),
            None
        )
        .await
        .0,
        StatusCode::GONE
    );
}
async fn terminal(app: &axum::Router, owner: &Value) -> Value {
    let path = format!(
        "/api/v1/searches/{}/updates?limit=100",
        owner["id"].as_str().unwrap()
    );
    let token = owner["accessToken"].as_str().unwrap();
    let start = std::time::Instant::now();
    loop {
        let (status, v) = call(app, "GET", &path, None, Some(token)).await;
        assert_eq!(status, StatusCode::OK);
        if v["status"] != "running" {
            return v;
        }
        assert!(start.elapsed().as_secs() < 30, "{v}");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
#[tokio::test]
async fn invalid_inputs_keep_correlation_and_are_distinct_from_infeasibility() {
    let app = skating_planning_service::app();
    let (_, identity) = call(&app, "GET", "/api/v1/searches/identity", None, None).await;
    let (_, profiles) = call(&app, "GET", "/api/v1/profiles", None, None).await;
    let context = &profiles[0]["context"];
    let key = |code: &str| {
        format!(
            "{}:{}:{}:{code}:false",
            context["profile"].as_str().unwrap(),
            context["ruleEdition"].as_str().unwrap(),
            context["scoringTable"].as_str().unwrap()
        )
    };
    let base = |c: &str| json!({"creationKey":format!("{}:{}",identity["creationNamespace"].as_str().unwrap(),c.repeat(64)),"requestId":"invalid-http","revision":19,"context":context,"requiredElements":[],"personalLimits":{}});
    let mut malformed = base("a");
    malformed["requiredElements"] = json!([{"id":"broken","elementKey":"old:bad:key"}]);
    let (status, error) = call(&app, "POST", "/api/v1/searches", Some(malformed), None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error["requestId"], "invalid-http");
    assert_eq!(error["revision"], 19);
    assert_eq!(error["entryId"], "broken");
    assert_eq!(error["field"], "requiredElements");
    let mut invalid = base("b");
    invalid["requiredElements"] = json!([{"id":"disabled-spin","elementKey":key("CCSp1")}]);
    invalid["personalLimits"] = json!({"spins":{"CCSp":{"enabled":false}}});
    let (status, owner) = call(&app, "POST", "/api/v1/searches", Some(invalid), None).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let result = terminal(&app, &owner).await;
    assert_eq!(result["status"], "failed");
    assert_eq!(result["failure"]["code"], "request_invalid");
    assert_eq!(result["failure"]["entryIds"], json!(["disabled-spin"]));
    assert_eq!(result["failure"]["requestId"], "invalid-http");
    assert_eq!(result["failure"]["revision"], 19);
    let mut infeasible = base("c");
    infeasible["requiredElements"] = json!([{"id":"one","elementKey":key("CCSp1")},{"id":"two","elementKey":key("CCSp1")},{"id":"three","elementKey":key("CCSp1")}]);
    let (status, owner) = call(&app, "POST", "/api/v1/searches", Some(infeasible), None).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let result = terminal(&app, &owner).await;
    assert_eq!(result["status"], "infeasible", "{result}");
    assert!(result["maximumBaseHundredths"].is_null());
    assert!(result["results"].as_array().unwrap().is_empty());
    assert!(!result["diagnostics"].as_array().unwrap().is_empty());
}
