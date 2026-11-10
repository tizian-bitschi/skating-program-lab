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
        if v["status"] != "running" && v["status"] != "paused" {
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

#[tokio::test]
async fn collection_controls_reject_proof_and_wrong_receipt_action() {
    let app = skating_planning_service::app();
    let (_, identity) = call(&app, "GET", "/api/v1/searches/identity", None, None).await;
    let (_, profiles) = call(&app, "GET", "/api/v1/profiles", None, None).await;
    let request = json!({"creationKey":format!("{}:{}",identity["creationNamespace"].as_str().unwrap(),"c".repeat(64)),"requestId":"controls","revision":0,"context":profiles[5]["context"],"requiredElements":[],"personalLimits":{}});
    let (_, owner) = call(&app, "POST", "/api/v1/searches", Some(request), None).await;
    let id = owner["id"].as_str().unwrap();
    let token = owner["accessToken"].as_str().unwrap();
    for action in ["pause", "continue"] {
        assert_eq!(
            call(
                &app,
                "POST",
                &format!("/api/v1/searches/{id}/{action}"),
                Some(json!({"commandId":action})),
                Some(token)
            )
            .await
            .0,
            StatusCode::CONFLICT
        );
    }
    let path = format!("/api/v1/searches/{id}/cancel");
    for _ in 0..2 {
        assert_eq!(
            call(
                &app,
                "POST",
                &path,
                Some(json!({"commandId":"stop"})),
                Some(token)
            )
            .await
            .0,
            StatusCode::OK
        );
    }
    assert_eq!(
        call(
            &app,
            "POST",
            &format!("/api/v1/searches/{id}/pause"),
            Some(json!({"commandId":"stop"})),
            Some(token)
        )
        .await
        .1["code"],
        "command_conflict"
    );
    let end = terminal(&app, &owner).await;
    assert_eq!(end["status"], "cancelled");
    assert!(end["controlRequested"].is_null());
}

// Independent Bronze stock argument: four entries permit one pair; the optimum
// is two Flips + two Loops + one Toe/Salchow = 240, plus retained 830.
fn finite_oracle() -> std::collections::BTreeSet<Vec<String>> {
    let mut expected = std::collections::BTreeSet::new();
    for low in ["1T", "1S"] {
        for (a, b) in [
            (low, "1F"),
            ("1F", low),
            (low, "1Lo"),
            ("1Lo", low),
            ("1F", "1F"),
            ("1F", "1Lo"),
            ("1Lo", "1F"),
            ("1Lo", "1Lo"),
        ] {
            let mut stock = vec![low, "1F", "1F", "1Lo", "1Lo"];
            for code in [a, b] {
                let i = stock.iter().position(|c| *c == code).unwrap();
                stock.remove(i);
            }
            let mut codes = stock.into_iter().map(str::to_owned).collect::<Vec<_>>();
            codes.push(format!("{a}+{b}"));
            codes.extend(["CCSp1", "CLSp1", "ChSq1"].map(str::to_owned));
            codes.sort();
            expected.insert(codes);
        }
    }
    assert_eq!(expected.len(), 16);
    expected
}
#[tokio::test]
async fn every_finite_optimum_has_full_identity_provenance_and_replayable_deltas() {
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
    for excluded in [false, true] {
        let required = [
            ("spin-a", "CCSp1"),
            ("spin-b", "CLSp1"),
            ("choreo", "ChSq1"),
        ];
        let request = json!({"creationKey":format!("{}:{}",identity["creationNamespace"].as_str().unwrap(),if excluded {"e"}else{"d"}.repeat(64)),"requestId":"finite","revision":3,"context":context,"requiredElements":required.map(|(id,code)|json!({"id":id,"elementKey":key(code)})),"allowMixedDirections":true,"excludedConnections":if excluded {vec![key("1F+1Lo")]}else{vec![]},"personalLimits":{"jumps":{"Lz":{"enabled":false},"A":{"enabled":false}},"unlevelledElements":{"W":false}}});
        let (_, owner) = call(
            &app,
            "POST",
            "/api/v1/searches",
            Some(request.clone()),
            None,
        )
        .await;
        if !excluded {
            let token = owner["accessToken"].as_str().unwrap();
            let id = owner["id"].as_str().unwrap();
            let start = std::time::Instant::now();
            loop {
                let (status, _) = call(
                    &app,
                    "POST",
                    &format!("/api/v1/searches/{id}/pause"),
                    Some(json!({"commandId":"finite-pause"})),
                    Some(token),
                )
                .await;
                if status == StatusCode::OK {
                    break;
                }
                assert_eq!(status, StatusCode::CONFLICT);
                assert!(start.elapsed().as_secs() < 30);
                tokio::task::yield_now().await;
            }
            loop {
                let (_, state) = call(
                    &app,
                    "GET",
                    &format!("/api/v1/searches/{id}/updates?limit=0"),
                    None,
                    Some(token),
                )
                .await;
                if state["status"] == "paused" {
                    break;
                }
                assert_eq!(state["status"], "running", "{state}");
                assert!(start.elapsed().as_secs() < 30);
                tokio::task::yield_now().await;
            }
            assert_eq!(
                call(
                    &app,
                    "POST",
                    &format!("/api/v1/searches/{id}/continue"),
                    Some(json!({"commandId":"finite-continue"})),
                    Some(token)
                )
                .await
                .0,
                StatusCode::OK
            );
        }
        let end = terminal(&app, &owner).await;
        assert_eq!(end["status"], "completed", "{end}");
        assert_eq!(end["maximumBaseHundredths"], 1070);
        let mut expected = finite_oracle();
        if excluded {
            expected.retain(|codes| !codes.contains(&"1F+1Lo".into()));
        }
        assert_eq!(expected.len(), if excluded { 14 } else { 16 });
        let token = owner["accessToken"].as_str().unwrap();
        let id = owner["id"].as_str().unwrap();
        let mut cursor = 0;
        let mut actual = std::collections::BTreeSet::new();
        let mut ids = std::collections::BTreeSet::new();
        while cursor < expected.len() {
            let path = format!("/api/v1/searches/{id}/updates?after={cursor}&limit=3");
            let (_, page) = call(&app, "GET", &path, None, Some(token)).await;
            assert_eq!(page, call(&app, "GET", &path, None, Some(token)).await.1);
            for result in page["results"].as_array().unwrap() {
                assert!(ids.insert(result["id"].clone().to_string()));
                let entries = result["entries"].as_array().unwrap();
                let mut codes = Vec::new();
                for entry in entries {
                    let code = entry["code"].as_str().unwrap();
                    assert_eq!(entry["elementKey"], key(code));
                    codes.push(code.to_owned());
                    if let Some((required_id, _)) = required.iter().find(|(_, c)| *c == code) {
                        assert_eq!(entry["requiredId"], *required_id);
                    } else {
                        assert!(entry["requiredId"].is_null());
                    }
                }
                codes.sort();
                assert!(actual.insert(codes));
                assert_eq!(result["assessment"]["totalHundredths"], 1070);
                assert_eq!(result["assessment"]["construction"], "compliant");
                let assessment = json!({"requestId":"fresh","revision":4,"context":context,"personalLimits":request["personalLimits"],"entries":entries.iter().enumerate().map(|(i,e)|json!({"id":format!("fresh-{i}"),"elementKey":e["elementKey"]})).collect::<Vec<_>>(),"submission":"complete","allowMixedDirections":true,"excludedConnections":request["excludedConnections"]});
                let (status, fresh) = call(
                    &app,
                    "POST",
                    "/api/v1/program-assessments",
                    Some(assessment),
                    None,
                )
                .await;
                assert_eq!(status, StatusCode::OK, "{fresh}");
                assert_eq!(fresh["totalHundredths"], 1070);
                assert_eq!(fresh["construction"], "compliant");
            }
            cursor = page["cursor"].as_u64().unwrap() as usize;
        }
        assert_eq!(actual, expected);
        for action in ["pause", "continue"] {
            assert_eq!(
                call(
                    &app,
                    "POST",
                    &format!("/api/v1/searches/{id}/{action}"),
                    Some(json!({"commandId":format!("terminal-{action}")})),
                    Some(token)
                )
                .await
                .0,
                StatusCode::CONFLICT
            );
        }
    }
}

#[tokio::test]
async fn pause_continue_and_cancel_reach_the_same_collecting_owner() {
    collecting_owner_recovery(false).await;
}

#[tokio::test]
async fn paused_heartbeat_renews_the_lease_and_expiry_releases_the_owner() {
    collecting_owner_recovery(true).await;
}

async fn collecting_owner_recovery(expire: bool) {
    let clock = std::sync::Arc::new(Clock::default());
    let app = skating_planning_service::app_with_catalog_and_search_options(
        skating_planning_service::Catalog::default(),
        skating_planning_service::SearchServiceOptions {
            capacity: 1,
            clock: clock.clone(),
        },
    );
    let (_, namespace) = call(&app, "GET", "/api/v1/searches/identity", None, None).await;
    let (_, profiles) = call(&app, "GET", "/api/v1/profiles", None, None).await;
    let request = json!({"creationKey":format!("{}:{}",namespace["creationNamespace"].as_str().unwrap(),"f".repeat(64)),"requestId":"live-controls","revision":9,"context":profiles[1]["context"],"requiredElements":[],"allowMixedDirections":true,"personalLimits":{"jumps":{"T":{"enabled":false},"S":{"enabled":false},"Lz":{"enabled":false},"F":{"enabled":true,"maximum":1},"Lo":{"enabled":true,"maximum":1},"A":{"enabled":true,"maximum":1}},"unlevelledElements":{"W":false}}});
    let (_, owner) = call(
        &app,
        "POST",
        "/api/v1/searches",
        Some(request.clone()),
        None,
    )
    .await;
    let id = owner["id"].as_str().unwrap();
    let token = owner["accessToken"].as_str().unwrap();
    let path = format!("/api/v1/searches/{id}/updates?limit=0");
    let start = std::time::Instant::now();
    loop {
        let (_, v) = call(&app, "GET", &path, None, Some(token)).await;
        if v["phase"] == "collecting" {
            break;
        }
        assert_eq!(v["status"], "running", "{v}");
        assert!(start.elapsed().as_secs() < 30);
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }
    let command = |action: &str| format!("/api/v1/searches/{id}/{action}");
    assert_eq!(
        call(
            &app,
            "POST",
            &command("pause"),
            Some(json!({"commandId":"pause"})),
            Some(token)
        )
        .await
        .0,
        StatusCode::OK
    );
    let paused = loop {
        let (_, v) = call(&app, "GET", &path, None, Some(token)).await;
        if v["status"] == "paused" {
            break v;
        }
        assert!(start.elapsed().as_secs() < 30, "{v}");
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    };
    assert!(paused["controlRequested"].is_null());
    assert_eq!(paused["proof"], "proved");
    assert_eq!(paused["collectionComplete"], false);
    if expire {
        // Public status contact renews a paused owner's lease, without continuing it.
        for seconds in [899, 1798] {
            clock.0.store(seconds, std::sync::atomic::Ordering::SeqCst);
            let (status, heartbeat) = call(&app, "GET", &path, None, Some(token)).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(heartbeat["status"], "paused");
            assert_eq!(heartbeat["confirmedCount"], paused["confirmedCount"]);
        }
        clock.0.store(2699, std::sync::atomic::Ordering::SeqCst);
        assert_eq!(
            call(&app, "GET", &path, None, Some(token)).await.0,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            call(
                &app,
                "POST",
                "/api/v1/searches",
                Some(request.clone()),
                None
            )
            .await
            .0,
            StatusCode::NOT_FOUND
        );
        for suffix in ["a", "b", "c", "d"] {
            let mut next = request.clone();
            next["creationKey"] = json!(format!(
                "{}:{}",
                namespace["creationNamespace"].as_str().unwrap(),
                suffix.repeat(64)
            ));
            let wait = std::time::Instant::now();
            loop {
                let (status, replacement) =
                    call(&app, "POST", "/api/v1/searches", Some(next.clone()), None).await;
                if status == StatusCode::ACCEPTED {
                    assert_ne!(replacement["id"], owner["id"]);
                    let replacement_id = replacement["id"].as_str().unwrap();
                    call(
                        &app,
                        "DELETE",
                        &format!("/api/v1/searches/{replacement_id}"),
                        None,
                        replacement["accessToken"].as_str(),
                    )
                    .await;
                    assert_eq!(
                        call(
                            &app,
                            "GET",
                            &format!("/api/v1/searches/{replacement_id}/updates"),
                            None,
                            replacement["accessToken"].as_str()
                        )
                        .await
                        .0,
                        StatusCode::NOT_FOUND
                    );
                    break;
                }
                assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
                assert!(wait.elapsed().as_secs() < 10);
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            }
        }
        return;
    }

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    assert_eq!(
        call(&app, "GET", &path, None, Some(token)).await.1["confirmedCount"],
        paused["confirmedCount"]
    );
    assert_eq!(
        call(
            &app,
            "POST",
            &command("continue"),
            Some(json!({"commandId":"continue"})),
            Some(token)
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &app,
            "POST",
            &command("continue"),
            Some(json!({"commandId":"continue"})),
            Some(token)
        )
        .await
        .0,
        StatusCode::OK
    );
    loop {
        let (_, v) = call(&app, "GET", &path, None, Some(token)).await;
        if v["confirmedCount"].as_u64() > paused["confirmedCount"].as_u64() {
            break;
        }
        assert!(start.elapsed().as_secs() < 30, "{v}");
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }
    assert_eq!(
        call(
            &app,
            "POST",
            &command("cancel"),
            Some(json!({"commandId":"cancel"})),
            Some(token)
        )
        .await
        .0,
        StatusCode::OK
    );
    let end = terminal(&app, &owner).await;
    assert_eq!(end["status"], "cancelled");
    assert_eq!(end["id"], owner["id"]);
    assert_eq!(
        end["maximumBaseHundredths"],
        paused["maximumBaseHundredths"]
    );
    assert!(end["controlRequested"].is_null());
    assert!(end["confirmedCount"].as_u64() >= paused["confirmedCount"].as_u64());
}

#[tokio::test]
async fn ignored_zero_value_required_occurrences_keep_distinct_http_identity() {
    let app = skating_planning_service::app();
    let (_, namespace) = call(&app, "GET", "/api/v1/searches/identity", None, None).await;
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
    let required = [
        ("spin-a", "CCSp1"),
        ("spin-b", "CLSp1"),
        ("choreo", "ChSq1"),
        ("first-waltz", "W"),
        ("second-waltz", "W"),
    ];
    let request = json!({"creationKey":format!("{}:{}",namespace["creationNamespace"].as_str().unwrap(),"a".repeat(64)),"requestId":"ignored-identities","revision":1,"context":context,"requiredElements":required.map(|(id,code)|json!({"id":id,"elementKey":key(code)})),"allowMixedDirections":true,"personalLimits":{"jumps":{"T":{"enabled":false},"S":{"enabled":false},"Lz":{"enabled":false},"A":{"enabled":false}}}});
    let (_, owner) = call(&app, "POST", "/api/v1/searches", Some(request), None).await;
    let end = terminal(&app, &owner).await;
    assert_eq!(end["status"], "completed", "{end}");
    assert_eq!(end["maximumBaseHundredths"], 1030);
    assert_eq!(end["confirmedCount"], 5);
    for result in end["results"].as_array().unwrap() {
        let entries = result["entries"].as_array().unwrap();
        for (id, code) in required {
            let index = entries.iter().position(|e| e["requiredId"] == id).unwrap();
            assert_eq!(entries[index]["elementKey"], key(code));
            assert_eq!(entries[index]["code"], code);
            if code == "W" {
                assert_eq!(
                    result["assessment"]["elements"][index]["creditedHundredths"],
                    0
                );
            }
        }
        assert_eq!(entries.iter().filter(|e| e["code"] == "W").count(), 2);
        assert_eq!(result["assessment"]["construction"], "compliant");
    }
}

#[tokio::test]
async fn called_waltz_respects_axel_off_while_ignored_waltz_stays_independent() {
    let app = skating_planning_service::app();
    let (_, profiles) = call(&app, "GET", "/api/v1/profiles", None, None).await;
    for index in 0..3 {
        let base = json!({"requestId":"waltz-permission","revision":1,"context":profiles[index]["context"],"personalLimits":{}});
        let (_, unrestricted) =
            call(&app, "POST", "/api/v1/repertoire", Some(base.clone()), None).await;
        let w = unrestricted["elements"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["form"] == "W")
            .unwrap();
        for axel in [false, true] {
            for waltz in [false, true] {
                let allowed = waltz && (index == 0 || axel);
                let mut request = base.clone();
                request["personalLimits"] =
                    json!({"jumps":{"A":{"enabled":axel}},"unlevelledElements":{"W":waltz}});
                let (_, repertoire) = call(
                    &app,
                    "POST",
                    "/api/v1/repertoire",
                    Some(request.clone()),
                    None,
                )
                .await;
                assert_eq!(
                    repertoire["elements"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|e| e["form"] == "W"),
                    allowed,
                    "profile {index}, Axel {axel}, W {waltz}"
                );
                request["entries"] = json!([{"id":"kept-waltz","elementKey":w["key"]}]);
                request["submission"] = json!("draft");
                let (status, assessed) = call(
                    &app,
                    "POST",
                    "/api/v1/program-assessments",
                    Some(request),
                    None,
                )
                .await;
                assert_eq!(status, StatusCode::OK, "{assessed}");
                assert_eq!(
                    assessed["personalConflicts"].as_array().unwrap().is_empty(),
                    allowed,
                    "{assessed}"
                );
                if waltz && !axel && index != 0 {
                    assert_eq!(assessed["personalConflicts"][0]["setting"], json!(["A"]));
                }
            }
        }
    }
}

#[tokio::test]
async fn whole_called_waltz_connections_and_retained_search_match_manual_permissions() {
    let app = skating_planning_service::app_with_catalog_and_search_options(
        skating_planning_service::Catalog::default(),
        skating_planning_service::SearchServiceOptions {
            capacity: 1,
            ..Default::default()
        },
    );
    let (_, profiles) = call(&app, "GET", "/api/v1/profiles", None, None).await;
    let (_, namespace) = call(&app, "GET", "/api/v1/searches/identity", None, None).await;
    for index in 0..3 {
        let base = json!({"requestId":"whole-waltz","revision":1,"context":profiles[index]["context"],"personalLimits":{}});
        let (_, repertoire) =
            call(&app, "POST", "/api/v1/repertoire", Some(base.clone()), None).await;
        let solo = repertoire["elements"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["form"] == "W")
            .unwrap()
            .clone();
        let w = solo["code"].as_str().unwrap().to_owned();
        let mut query = base.clone();
        query["prefix"] = json!([w, "1Lo"]);
        query["allowMixedDirections"] = json!(true);
        let (_, options) = call(
            &app,
            "POST",
            "/api/v1/connections/options",
            Some(query),
            None,
        )
        .await;
        // Ignored Bronze Waltz is a separate optional movement, never fabricated
        // as a counted compound; Silver/Gold support the exact called intention.
        let compound = options["completions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["code"] == format!("{w}+1Lo"))
            .cloned();
        if index == 0 {
            assert!(compound.is_none());
        } else {
            assert!(compound.is_some(), "{options}");
        }
        for (which, element) in std::iter::once(solo).chain(compound).enumerate() {
            for (mode, axel, waltz) in [(0, false, true), (1, true, false), (2, true, true)] {
                let allowed = waltz && (index == 0 || axel);
                let mut request = base.clone();
                request["personalLimits"] =
                    json!({"jumps":{"A":{"enabled":axel}},"unlevelledElements":{"W":waltz}});
                let mut manual = request.clone();
                manual["submission"] = json!("draft");
                manual["entries"] = json!([{"id":"retained","elementKey":element["key"]}]);
                let (_, assessed) = call(
                    &app,
                    "POST",
                    "/api/v1/program-assessments",
                    Some(manual),
                    None,
                )
                .await;
                assert_eq!(
                    assessed["personalConflicts"].as_array().unwrap().is_empty(),
                    allowed,
                    "{assessed}"
                );
                if which == 1 {
                    let mut query = request.clone();
                    query["prefix"] = json!([w, "1Lo"]);
                    query["allowMixedDirections"] = json!(true);
                    let (_, options) = call(
                        &app,
                        "POST",
                        "/api/v1/connections/options",
                        Some(query),
                        None,
                    )
                    .await;
                    assert_eq!(
                        options["completions"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|e| e["key"] == element["key"]),
                        allowed
                    );
                }
                request["creationKey"] = json!(format!(
                    "{}:{:064x}",
                    namespace["creationNamespace"].as_str().unwrap(),
                    100 + index * 10 + which * 3 + mode
                ));
                request["requiredElements"] =
                    json!([{"id":"retained","elementKey":element["key"]}]);
                request["allowMixedDirections"] = json!(true);
                let wait = std::time::Instant::now();
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
                    assert!(wait.elapsed().as_secs() < 30, "owner did not terminate");
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                };
                let id = owner["id"].as_str().unwrap();
                let token = owner["accessToken"].as_str().unwrap();
                let start = std::time::Instant::now();
                loop {
                    let (_, state) = call(
                        &app,
                        "GET",
                        &format!("/api/v1/searches/{id}/updates?limit=1"),
                        None,
                        Some(token),
                    )
                    .await;
                    if state["status"] == "failed" {
                        assert!(!allowed, "{state}");
                        assert_eq!(state["failure"]["code"], "request_invalid");
                        assert_eq!(state["failure"]["entryIds"], json!(["retained"]));
                        break;
                    }
                    if state["confirmedCount"].as_u64().unwrap() > 0 {
                        assert!(allowed, "{state}");
                        assert!(state["results"][0]["entries"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|e| e["requiredId"] == "retained"
                                && e["elementKey"] == element["key"]));
                        break;
                    }
                    assert!(start.elapsed().as_secs() < 120, "{state}");
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
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
                // Sequential owner cleanup; cancellation is cooperative.
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            }
        }
    }
}
