use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use skating_program_lab::*;
use std::{collections::BTreeMap, sync::Arc};
use tower_http::services::ServeDir;

pub const TABLE: ScoringTableId = ScoringTableId::Isu2786June2_2026;
#[derive(Clone)]
pub struct Catalog {
    pub registry: ProfileRegistry,
    pub profiles: Vec<RuleProfile>,
}
impl Default for Catalog {
    fn default() -> Self {
        let mut registry = ProfileRegistry::adult();
        let mut profiles = [
            AdultCategory::Bronze,
            AdultCategory::Silver,
            AdultCategory::Gold,
            AdultCategory::Masters,
        ]
        .map(adult_profile)
        .to_vec();
        for category in [SeniorCategory::Women, SeniorCategory::Men] {
            let profile = senior_profile(category);
            registry.register(profile.clone()).expect("built-in");
            profiles.push(profile);
        }
        Self { registry, profiles }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Context {
    pub profile: String,
    pub rule_edition: String,
    pub scoring_table: String,
}
impl Context {
    pub fn for_profile(p: &RuleProfile) -> Self {
        Self {
            profile: p.id.name.clone(),
            rule_edition: p.id.version.clone(),
            scoring_table: p.scoring_table.as_str().into(),
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Limit {
    pub enabled: bool,
    pub maximum: Option<u8>,
}
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Limits {
    #[serde(default)]
    pub jumps: BTreeMap<String, Limit>,
    #[serde(default)]
    pub spins: BTreeMap<String, Limit>,
    pub steps: Option<Limit>,
    #[serde(default)]
    pub unlevelled_elements: BTreeMap<String, bool>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepertoireRequest {
    pub request_id: String,
    pub revision: u64,
    pub context: Context,
    #[serde(default)]
    pub personal_limits: Limits,
}
pub type ApiError = (StatusCode, Json<Value>);
pub fn input_error(request: Option<&RepertoireRequest>, code: &str, field: &str) -> ApiError {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(
            json!({"code":code,"category":"input","translationKey":format!("errors.{code}"),"field":field,"requestId":request.map(|r| &r.request_id),"revision":request.map(|r|r.revision)}),
        ),
    )
}
pub fn app() -> Router {
    app_with_catalog(Catalog::default())
}
pub fn app_with_catalog(catalog: Catalog) -> Router {
    Router::new()
        .route("/api/v1/profiles", get(profiles))
        .route("/api/v1/repertoire", post(repertoire))
        .with_state(Arc::new(catalog))
        .fallback_service(
            ServeDir::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/dist"))
                .append_index_html_on_directories(true),
        )
}
async fn profiles(State(catalog): State<Arc<Catalog>>) -> Json<Value> {
    Json(Value::Array(catalog.profiles.iter().map(|p|json!({"context":Context::for_profile(p),"label":p.category,"translationKey":format!("profiles.{}",p.id.name),"capabilities":{"repertoire":true},"sources":p.sources.iter().map(source).collect::<Vec<_>>() })).collect()))
}
fn source(s: &SourceReference) -> Value {
    json!({"document":s.document,"version":s.version,"location":s.location})
}
pub fn resolve_context<'a>(
    catalog: &'a Catalog,
    context: &Context,
) -> Result<&'a RuleProfile, ApiError> {
    if context.scoring_table != TABLE.as_str() {
        return Err(input_error(
            None,
            "context_unavailable",
            "context.scoringTable",
        ));
    }
    catalog
        .registry
        .resolve(
            &ProfileId {
                name: context.profile.clone(),
                version: context.rule_edition.clone(),
            },
            TABLE,
        )
        .map_err(|_| input_error(None, "context_unavailable", "context"))
}
fn level_number(level: Level) -> u8 {
    match level {
        Level::Basic => 0,
        Level::One => 1,
        Level::Two => 2,
        Level::Three => 3,
        Level::Four => 4,
    }
}
fn form(choice: &SelectableElement) -> (String, &'static str, Option<u8>) {
    match choice.kind {
        SelectableElementKind::SoloJump { kind, rotations } => (
            match kind {
                JumpKind::Waltz => "W",
                JumpKind::ToeLoop => "T",
                JumpKind::Salchow => "S",
                JumpKind::Loop => "Lo",
                JumpKind::Flip => "F",
                JumpKind::Lutz => "Lz",
                JumpKind::Axel => "A",
            }
            .into(),
            "jump",
            Some(rotations),
        ),
        SelectableElementKind::RegularSpin(_) => (
            choice
                .row
                .notation
                .trim_end_matches('V')
                .trim_end_matches(['B', '1', '2', '3', '4'])
                .into(),
            "spin",
            choice.row.level.map(level_number),
        ),
        SelectableElementKind::StepSequence => {
            ("StSq".into(), "step", choice.row.level.map(level_number))
        }
        SelectableElementKind::ChoreographicSequence => ("ChSq".into(), "unlevelled", None),
        SelectableElementKind::ChoreographicSpin => ("ChSp".into(), "unlevelled", None),
    }
}
fn forms() -> Vec<(String, &'static str)> {
    let mut result = ["T", "S", "Lo", "F", "Lz", "A"]
        .map(|c| (c.into(), "jump"))
        .to_vec();
    for prefix in ["", "C", "F", "FC"] {
        for suffix in ["USp", "SSp", "CSp", "LSp", "CoSp"] {
            result.push((format!("{prefix}{suffix}"), "spin"));
        }
    }
    result.extend([
        ("StSq".into(), "step"),
        ("ChSq".into(), "unlevelled"),
        ("ChSp".into(), "unlevelled"),
        ("W".into(), "unlevelled"),
    ]);
    result
}
fn validate_limits(r: &RepertoireRequest) -> Result<(), ApiError> {
    let known = forms();
    for (values, family, min, max) in [
        (&r.personal_limits.jumps, "jump", 1, 5),
        (&r.personal_limits.spins, "spin", 0, 4),
    ] {
        for (key, limit) in values {
            if !known.iter().any(|(k, f)| k == key && *f == family)
                || limit.maximum.is_some_and(|m| m < min || m > max)
            {
                return Err(input_error(Some(r), "invalid_limit", "personalLimits"));
            }
        }
    }
    if r.personal_limits
        .steps
        .as_ref()
        .is_some_and(|l| l.maximum.is_some_and(|m| m > 4))
        || r.personal_limits
            .unlevelled_elements
            .keys()
            .any(|k| !known.iter().any(|(id, f)| id == k && *f == "unlevelled"))
    {
        return Err(input_error(Some(r), "invalid_limit", "personalLimits"));
    }
    Ok(())
}
fn permits(limits: &Limits, id: &str, family: &str, level: Option<u8>) -> bool {
    let limit = match family {
        "jump" => limits.jumps.get(id),
        "spin" => limits.spins.get(id),
        "step" => limits.steps.as_ref(),
        _ => return limits.unlevelled_elements.get(id).copied().unwrap_or(true),
    };
    limit.is_none_or(|l| l.enabled && l.maximum.is_none_or(|max| level.is_some_and(|n| n <= max)))
}
/// Shared conversion for repertoire, assessment and search. Off remains a
/// separate whole-element exclusion; it cannot be represented by Basic/Single.
pub fn core_limits(
    limits: &Limits,
    choices: &[SelectableElement],
) -> (PersonalLimits, Option<Level>) {
    let mut result = PersonalLimits::default();
    for choice in choices {
        let (id, _, _) = form(choice);
        match choice.kind {
            SelectableElementKind::SoloJump { kind, .. } => {
                if let Some(maximum) = limits
                    .jumps
                    .get(&id)
                    .and_then(|l| l.maximum)
                    .and_then(|n| JumpRotationClass::try_from(n).ok())
                {
                    if !result.jump_rotations.iter().any(|l| l.kind == kind) {
                        result
                            .jump_rotations
                            .push(JumpRotationLimit { kind, maximum });
                    }
                }
            }
            SelectableElementKind::RegularSpin(abbreviation) => {
                if let Some(maximum) = limits
                    .spins
                    .get(&id)
                    .and_then(|l| l.maximum)
                    .and_then(level)
                {
                    if !result
                        .spin_levels
                        .iter()
                        .any(|l| l.abbreviation == abbreviation)
                    {
                        result.spin_levels.push(SpinLevelLimit {
                            abbreviation,
                            maximum,
                        });
                    }
                }
            }
            _ => {}
        }
    }
    (
        result,
        limits
            .steps
            .as_ref()
            .and_then(|l| l.maximum)
            .and_then(level),
    )
}
fn level(n: u8) -> Option<Level> {
    if n == 0 {
        Some(Level::Basic)
    } else {
        Level::try_from(n).ok()
    }
}
pub fn repertoire_response(catalog: &Catalog, r: &RepertoireRequest) -> Result<Value, ApiError> {
    let profile = resolve_context(catalog, &r.context)
        .map_err(|_| input_error(Some(r), "context_unavailable", "context"))?;
    validate_limits(r)?;
    let choices = selectable_elements(&catalog.registry, &profile.id, TABLE)
        .map_err(|_| input_error(Some(r), "context_unavailable", "context"))?;
    let (personal_limits, step_limit) = core_limits(&r.personal_limits, &choices);
    let permitted = selectable_elements_with_step_limit(
        &catalog.registry,
        &profile.id,
        TABLE,
        &personal_limits,
        step_limit,
    )
    .map_err(|_| input_error(Some(r), "context_unavailable", "context"))?;
    let mut metadata=forms().into_iter().map(|(id,family)|{
  let eligible:Vec<_>=choices.iter().filter(|c|form(c).0==id).collect();
  let mut levels:Vec<_>=eligible.iter().filter_map(|c|form(c).2).collect(); levels.sort();levels.dedup();
  json!({"id":id,"family":family,"translationKey":format!("forms.{id}"),"available":!eligible.is_empty(),"eligibleLevels":levels,"permittedLevels":levels.iter().filter(|&&n|permits(&r.personal_limits,&id,family,Some(n))).collect::<Vec<_>>()})
 }).collect::<Vec<_>>();
    let mut elements=permitted.iter().filter_map(|choice|{let (id,family,level)=form(choice);permits(&r.personal_limits,&id,family,level).then(||json!({"key":format!("{}:{}:{}:{}",r.context.profile,r.context.rule_edition,r.context.scoring_table,choice.row.notation),"code":choice.row.notation,"form":id,"family":family,"level":level,"baseHundredths":choice.row.base_hundredths,"source":source(&choice.row.source.reference(choice.row.notation))}))}).collect::<Vec<_>>();
    // Planned Waltz has no clean catalog row; assessment resolves its spelling.
    let mut waltz_available = false;
    for code in ["W", "W(1A<<)"] {
        let request = ManualBvRequest {
            personal_limits: PersonalLimits::default(),
            rotation_preference: RotationPreference::default(),
            profile: profile.id.clone(),
            scoring_table: TABLE,
            submission: SubmissionState::Draft,
            elements: vec![AssumedRecognition::new(code)],
        };
        if let Ok(assessment) = assess_manual_bv(&catalog.registry, &request) {
            let entry = &assessment.elements[0];
            if let Ok(outcome) = &entry.credited_base {
                waltz_available = true;
                if r.personal_limits
                    .unlevelled_elements
                    .get("W")
                    .copied()
                    .unwrap_or(true)
                {
                    let value = match outcome {
                        BaseValueOutcome::Value(value) => value.hundredths,
                        _ => 0,
                    };
                    elements.push(json!({"key":format!("{}:{}:{}:{}",r.context.profile,r.context.rule_edition,r.context.scoring_table,code),"code":code,"form":"W","family":"unlevelled","level":null,"baseHundredths":value,"source":entry.table_base.as_ref().map(|value| source(&value.source))}));
                }
                break;
            }
        }
    }
    metadata.retain(|entry| entry["id"] != "W");
    metadata.push(json!({"id":"W","family":"unlevelled","translationKey":"forms.W","available":waltz_available,"eligibleLevels":[],"permittedLevels":[]}));
    Ok(
        json!({"requestId":r.request_id,"revision":r.revision,"context":r.context,"forms":metadata,"elements":elements}),
    )
}
async fn repertoire(
    State(catalog): State<Arc<Catalog>>,
    payload: Result<Json<RepertoireRequest>, JsonRejection>,
) -> Result<Json<Value>, ApiError> {
    let Json(request)=payload.map_err(|_|(StatusCode::BAD_REQUEST,Json(json!({"code":"malformed_request","category":"transport","translationKey":"errors.malformed_request"}))))?;
    tokio::task::spawn_blocking(move || repertoire_response(&catalog, &request).map(Json))
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"code":"internal_failure","translationKey":"errors.internal_failure"})),
            )
        })?
}
