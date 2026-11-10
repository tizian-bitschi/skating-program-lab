use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;
#[tokio::test]
async fn manual_credit_preserves_occurrences_and_binary_construction() {
    let app = skating_planning_service::app();
    let context = json!({"profile":"isu-adult-2026-27-bronze","ruleEdition":"2026-07-16","scoringTable":"isu-2786-2026-06-02"});
    let entries:Vec<_>=["1Lz","SSp1","ChSq1"].iter().enumerate().map(|(i,code)|json!({"id":format!("occ-{i}"),"elementKey":format!("isu-adult-2026-27-bronze:2026-07-16:isu-2786-2026-06-02:{code}:false")})).collect();
    let response=app.oneshot(Request::post("/api/v1/program-assessments").header("content-type","application/json").body(Body::from(json!({"requestId":"manual","revision":4,"context":context,"personalLimits":{},"entries":entries,"submission":"complete"}).to_string())).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["revision"], 4);
    assert_eq!(body["construction"], "compliant");
    assert_eq!(body["elements"][0]["entryId"], "occ-0");
    assert_eq!(body["elements"][0]["creditedHundredths"], 60);
}
mod support;
#[tokio::test]
async fn exclusion_field_rejects_non_connections_with_correlated_errors() {
    let context = json!({"profile":"isu-adult-2026-27-bronze","ruleEdition":"2026-07-16","scoringTable":"isu-2786-2026-06-02"});
    for code in ["1Lz", "SSp1", "1Lz+SSp1"] {
        for path in ["/api/v1/program-assessments", "/api/v1/connections/options"] {
            let mut body = json!({"requestId":"excluded-invalid","revision":19,"context":context,"personalLimits":{},"excludedConnections":[format!("isu-adult-2026-27-bronze:2026-07-16:isu-2786-2026-06-02:{code}:false")]});
            if path.ends_with("options") {
                body["prefix"] = json!([]);
            } else {
                body["entries"] = json!([]);
                body["submission"] = json!("draft");
            }
            let (status, result) = post(skating_planning_service::app(), path, body).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{result}");
            assert_eq!(result["requestId"], "excluded-invalid");
            assert_eq!(result["revision"], 19);
            assert_eq!(result["field"], "excludedConnections");
        }
    }
}
async fn post(app: axum::Router, path: &str, body: Value) -> (StatusCode, Value) {
    let response = app
        .oneshot(
            Request::post(path)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}
fn entries(context: &Value, codes: &[&str]) -> Vec<Value> {
    codes.iter().enumerate().map(|(i,code)|json!({"id":format!("occ-{i}"),"elementKey":format!("{}:{}:{}:{code}:{}",context["profile"].as_str().unwrap(),context["ruleEdition"].as_str().unwrap(),context["scoringTable"].as_str().unwrap(),code.ends_with("+SEQ"))})).collect()
}
#[tokio::test]
async fn six_profiles_and_registered_policy_use_source_based_complete_fixtures() {
    let catalog = support::catalog();
    let bases = [
        vec!["1Lz", "SSp1", "ChSq1"],
        vec!["1A", "CLSp2", "ChSp1", "ChSq1"],
        vec!["1A", "CCoSp3", "FSSp3", "ChSp1", "StSq2"],
        vec!["1A", "CoSp4", "FSSp4", "ChSp1", "StSq4"],
        vec!["1A", "CoSp4", "FSSp4", "ChSp1", "StSq4", "ChSq1"],
        vec!["1A", "CoSp4", "FSSp4", "ChSp1", "StSq4", "ChSq1"],
        vec!["1Lz", "SSp1", "ChSq1"],
    ];
    let contexts = catalog
        .profiles
        .iter()
        .map(|p| serde_json::to_value(skating_planning_service::Context::for_profile(p)).unwrap())
        .collect::<Vec<_>>();
    let app = skating_planning_service::app_with_catalog(catalog);
    for (context, base) in contexts.iter().zip(bases) {
        let mut body = json!({"requestId":"seven","revision":12,"context":context,"personalLimits":{},"entries":entries(context,&base),"submission":"complete"});
        let (status, result) = post(app.clone(), "/api/v1/program-assessments", body.clone()).await;
        assert_eq!(status, StatusCode::OK, "{result}");
        assert_eq!(result["construction"], "compliant", "{result}");
        body["personalLimits"] = json!({"jumps":{"Lz":{"enabled":false},"A":{"enabled":false}}});
        let (_, limited) = post(app.clone(), "/api/v1/program-assessments", body.clone()).await;
        assert_eq!(limited["construction"], "compliant");
        assert_eq!(limited["totalHundredths"], result["totalHundredths"]);
        assert_eq!(limited["personalConflicts"][0]["entryId"], "occ-0");
        let spin = *base.iter().find(|c| c.contains("Sp")).unwrap();
        let mut invalid = base.clone();
        invalid.push(spin);
        body["entries"] = json!(entries(context, &invalid));
        let (status, result) = post(app.clone(), "/api/v1/program-assessments", body).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(result["construction"], "noncompliant");
        assert!(!result["categoryFindings"].as_array().unwrap().is_empty());
        for finding in result["categoryFindings"].as_array().unwrap() {
            assert!(finding["source"]["document"].as_str().unwrap().len() > 3);
        }
    }
}
#[tokio::test]
async fn optional_pair_completion_third_jump_and_exact_exclusion_are_independent() {
    let context = json!({"profile":"isu-adult-2026-27-gold","ruleEdition":"2026-07-16","scoringTable":"isu-2786-2026-06-02"});
    let app = skating_planning_service::app();
    let body = json!({"requestId":"prefix","revision":3,"context":context,"personalLimits":{},"prefix":["1Lz","1Lo"],"excludedConnections":[]});
    let (status, options) = post(app.clone(), "/api/v1/connections/options", body.clone()).await;
    assert_eq!(status, StatusCode::OK);
    assert!(options["next"].as_array().unwrap().contains(&json!("1Lo")));
    let pair = options["completions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["code"] == "1Lz+1Lo")
        .unwrap();
    assert_eq!(pair["directTransitionsAssumed"], false);
    let mut excluded = body;
    excluded["excludedConnections"] = json!([pair["key"]]);
    let (_, options) = post(app.clone(), "/api/v1/connections/options", excluded.clone()).await;
    assert_eq!(options["completions"][0]["excluded"], true);
    assert!(options["next"].as_array().unwrap().contains(&json!("1Lo")));
    excluded["prefix"] = json!(["1Lz", "1Lo", "1Lo"]);
    let (_, three) = post(app, "/api/v1/connections/options", excluded).await;
    assert!(three["completions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["code"] == "1Lz+1Lo+1Lo" && c["excluded"] == false));
}
#[tokio::test]
async fn exclusion_identity_survives_context_changes_and_reports_responsible_occurrence() {
    let catalog = support::catalog();
    let contexts = catalog
        .profiles
        .iter()
        .map(|p| serde_json::to_value(skating_planning_service::Context::for_profile(p)).unwrap())
        .collect::<Vec<_>>();
    let app = skating_planning_service::app_with_catalog(catalog);
    let original = &contexts[2];
    let (_, options) = post(app.clone(), "/api/v1/connections/options", json!({"requestId":"original","revision":1,"context":original,"personalLimits":{},"prefix":["1Lz","1A"]})).await;
    let sequence = &options["completions"][0];
    for context in &contexts {
        let body = json!({"requestId":"preserved","revision":42,"context":context,"personalLimits":{"jumps":{"A":{"enabled":false}}},"prefix":[],"excludedConnections":[sequence["key"]]});
        let (status, choices) = post(app.clone(), "/api/v1/connections/options", body).await;
        assert_eq!(status, StatusCode::OK, "{choices}");
        assert_eq!(choices["excludedConnections"][0]["key"], sequence["key"]);
        assert_eq!(
            choices["excludedConnections"][0]["directTransitionsAssumed"],
            true
        );
        assert_eq!(choices["excludedConnections"][0]["parts"][0]["form"], "Lz");
        if context["profile"] == "independent-http-policy" {
            assert_eq!(choices["next"], json!(["1Lz", "W"]));
            continue;
        }
        let mut pair = json!({"requestId":"pair","revision":43,"context":context,"personalLimits":{},"prefix":["1F","1Lo"]});
        let (status, selected) =
            post(app.clone(), "/api/v1/connections/options", pair.clone()).await;
        assert_eq!(status, StatusCode::OK, "{selected}");
        let whole = &selected["completions"][0];
        assert_eq!(
            whole["code"], "1F+1Lo",
            "context={context} options={selected}"
        );
        pair["excludedConnections"] = json!([whole["key"]]);
        let (_, excluded) = post(app.clone(), "/api/v1/connections/options", pair.clone()).await;
        assert_eq!(excluded["completions"][0]["excluded"], true);
        pair["prefix"] = json!([]);
        let (_, roots) = post(app.clone(), "/api/v1/connections/options", pair).await;
        assert!(roots["next"].as_array().unwrap().contains(&json!("1F")));
        assert!(roots["next"].as_array().unwrap().contains(&json!("1Lo")));
        let body = json!({"requestId":"conflict","revision":44,"context":context,"personalLimits":{},"excludedConnections":[whole["key"]],"entries":[{"id":"retained-whole","elementKey":whole["key"]}],"submission":"draft"});
        let (status, assessed) = post(app.clone(), "/api/v1/program-assessments", body).await;
        assert_eq!(status, StatusCode::OK, "{assessed}");
        assert_eq!(
            assessed["personalConflicts"][0]["entryId"],
            "retained-whole"
        );
        assert_eq!(
            assessed["personalConflicts"][0]["translationKey"],
            "excluded"
        );
    }
}
#[tokio::test]
async fn reordered_duplicate_credit_and_old_context_identity_are_preserved() {
    let bronze = json!({"profile":"isu-senior-2026-27-women-free","ruleEdition":"2026-06-12","scoringTable":"isu-2786-2026-06-02"});
    let silver = json!({"profile":"isu-senior-2026-27-men-free","ruleEdition":"2026-06-12","scoringTable":"isu-2786-2026-06-02"});
    let app = skating_planning_service::app();
    let mut body = json!({"requestId":"duplicates","revision":42,"context":silver,"entries":entries(&bronze,&["4T","4T"]),"personalLimits":{},"submission":"draft"});
    let (_, result) = post(app.clone(), "/api/v1/program-assessments", body.clone()).await;
    assert_eq!(result["elements"][0]["creditedHundredths"], 950);
    assert_eq!(result["elements"][1]["creditedHundredths"], 760);
    body["entries"].as_array_mut().unwrap().reverse();
    let (_, result) = post(app.clone(), "/api/v1/program-assessments", body.clone()).await;
    assert_eq!(result["elements"][1]["entryId"], "occ-0");
    assert_eq!(result["elements"][1]["creditedHundredths"], 760);
    body["entries"][0]["elementKey"] = json!("unknown:edition:table:1Lz:false");
    let (status, result) = post(app, "/api/v1/program-assessments", body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(result["revision"], 42);
}
#[tokio::test]
async fn sequence_assumption_is_preserved_and_malformed_transport_is_typed() {
    let context = json!({"profile":"isu-adult-2026-27-gold","ruleEdition":"2026-07-16","scoringTable":"isu-2786-2026-06-02"});
    let app = skating_planning_service::app();
    let(status,options)=post(app.clone(),"/api/v1/connections/options",json!({"requestId":"sequence","revision":9,"context":context,"personalLimits":{},"prefix":["1Lz","1A"]})).await;
    assert_eq!(status, StatusCode::OK);
    let sequence = options["completions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["code"] == "1Lz+1A+SEQ")
        .unwrap();
    assert_eq!(sequence["directTransitionsAssumed"], true);
    assert!(sequence["key"].as_str().unwrap().ends_with(":true"));
    let body = json!({"requestId":"missing-assumption","revision":10,"context":context,"personalLimits":{},"entries":[{"id":"sequence-id","elementKey":sequence["key"].as_str().unwrap().replace(":true",":false")}],"submission":"complete"});
    let (status, error) = post(app.clone(), "/api/v1/program-assessments", body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error["code"], "assessment_input_unavailable");
    let (status, error) = post(app, "/api/v1/program-assessments", json!({"unknown":true})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error["category"], "transport");
}
#[tokio::test]
async fn category_switch_keeps_unavailable_content_and_personal_conflict() {
    let old = json!({"profile":"isu-adult-2026-27-silver","ruleEdition":"2026-07-16","scoringTable":"isu-2786-2026-06-02"});
    let current = json!({"profile":"isu-adult-2026-27-bronze","ruleEdition":"2026-07-16","scoringTable":"isu-2786-2026-06-02"});
    let(status,result)=post(skating_planning_service::app(),"/api/v1/program-assessments",json!({"requestId":"switch","revision":33,"context":current,"entries":entries(&old,&["1A"]),"personalLimits":{"jumps":{"A":{"enabled":false}}},"submission":"complete"})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(result["construction"], "noncompliant");
    assert_eq!(result["personalConflicts"][0]["entryId"], "occ-0");
    assert!(result["categoryFindings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["entryId"] == "occ-0"));
}
