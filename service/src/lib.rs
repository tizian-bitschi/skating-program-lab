mod search;
use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
pub use search::{SearchClock, SearchServiceOptions};
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
    app_with_catalog_and_search_options(catalog, SearchServiceOptions::default())
}
pub fn app_with_catalog_and_search_options(
    catalog: Catalog,
    options: SearchServiceOptions,
) -> Router {
    let catalog = Arc::new(catalog);
    Router::new()
        .route("/api/v1/profiles", get(profiles))
        .route("/api/v1/repertoire", post(repertoire))
        .route("/api/v1/program-assessments", post(assessment))
        .route("/api/v1/connections/options", post(connections))
        .with_state(catalog.clone())
        .merge(search::router(catalog, options))
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
    let mut elements=permitted.iter().filter_map(|choice|{let (id,family,level)=form(choice);permits(&r.personal_limits,&id,family,level).then(||json!({"key":element_key(&r.context,&AssumedRecognition::new(choice.row.notation)),"code":choice.row.notation,"form":id,"family":family,"level":level,"baseHundredths":choice.row.base_hundredths,"source":source(&choice.row.source.reference(choice.row.notation))}))}).collect::<Vec<_>>();
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
                    elements.push(json!({"key":element_key(&r.context,&AssumedRecognition::new(code)),"code":code,"form":"W","family":"unlevelled","level":null,"baseHundredths":value,"source":entry.table_base.as_ref().map(|value| source(&value.source))}));
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
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProgramEntry {
    pub id: String,
    pub element_key: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssessmentRequest {
    #[serde(flatten)]
    pub repertoire: RepertoireRequest,
    pub entries: Vec<ProgramEntry>,
    pub submission: String,
    #[serde(default)]
    pub allow_mixed_directions: bool,
    #[serde(default)]
    pub excluded_connections: Vec<String>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConnectionRequest {
    #[serde(flatten)]
    pub repertoire: RepertoireRequest,
    #[serde(default)]
    pub prefix: Vec<String>,
    #[serde(default)]
    pub excluded_connections: Vec<String>,
    #[serde(default)]
    pub allow_mixed_directions: bool,
}
/// Context-bound identity includes the complete recognition assumption.
pub fn element_key(context: &Context, element: &AssumedRecognition) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        context.profile,
        context.rule_edition,
        context.scoring_table,
        element.code,
        element.direct_transitions_assumed
    )
}
/// Decode only canonical backend keys; code and direct-step assumption remain distinct.
pub fn decode_element_key(context: &Context, key: &str) -> Result<AssumedRecognition, ApiError> {
    let prefix = format!(
        "{}:{}:{}:",
        context.profile, context.rule_edition, context.scoring_table
    );
    let suffix = key
        .strip_prefix(&prefix)
        .ok_or_else(|| input_error(None, "invalid_element_key", "entries"))?;
    let (code, direct) = suffix
        .rsplit_once(':')
        .ok_or_else(|| input_error(None, "invalid_element_key", "entries"))?;
    let direct = match direct {
        "true" => true,
        "false" => false,
        _ => return Err(input_error(None, "invalid_element_key", "entries")),
    };
    if code.is_empty() {
        return Err(input_error(None, "invalid_element_key", "entries"));
    }
    Ok(AssumedRecognition {
        code: code.into(),
        direct_transitions_assumed: direct,
    })
}
/// Revalidate the original versioned identity, then assess unchanged content in the selected context.
pub fn resolve_workspace_element(
    catalog: &Catalog,
    key: &str,
) -> Result<AssumedRecognition, ApiError> {
    let parts = key.splitn(4, ':').collect::<Vec<_>>();
    if parts.len() != 4 {
        return Err(input_error(None, "invalid_element_key", "entries"));
    }
    let original = Context {
        profile: parts[0].into(),
        rule_edition: parts[1].into(),
        scoring_table: parts[2].into(),
    };
    resolve_context(catalog, &original)?;
    decode_element_key(&original, key)
}
fn personal_setting(kind: &PersonalLimitConflictKind, choices: &[SelectableElement]) -> String {
    match kind {
        PersonalLimitConflictKind::StepLevel { .. } => "StSq".into(),
        PersonalLimitConflictKind::JumpRotation { kind, .. } => match kind {
            JumpKind::Waltz => "W",
            JumpKind::ToeLoop => "T",
            JumpKind::Salchow => "S",
            JumpKind::Loop => "Lo",
            JumpKind::Flip => "F",
            JumpKind::Lutz => "Lz",
            JumpKind::Axel => "A",
        }
        .into(),
        PersonalLimitConflictKind::SpinLevel { abbreviation, .. } => choices
            .iter()
            .find(|c| matches!(c.kind, SelectableElementKind::RegularSpin(a) if a==*abbreviation))
            .map(|c| form(c).0)
            .unwrap_or_default(),
    }
}
fn personal_maximum(kind: &PersonalLimitConflictKind) -> u8 {
    match kind {
        PersonalLimitConflictKind::JumpRotation { maximum, .. } => *maximum as u8,
        PersonalLimitConflictKind::StepLevel { maximum, .. }
        | PersonalLimitConflictKind::SpinLevel { maximum, .. } => level_number(*maximum),
    }
}
fn finding_key(f: &Finding) -> &'static str {
    let id = f.rule_id;
    if id.contains("required") && id.contains("spin") {
        "required_spin"
    } else if id.contains("required") && id.contains("step") {
        "required_step"
    } else if id.contains("required") && id.contains("choreo") {
        "required_choreo"
    } else if id.contains("required") || f.explanation.contains("requires an Axel") {
        "required_axel"
    } else if id.contains("level") {
        "level"
    } else if id.contains("repetition") || id.contains("repeat") || id.contains("distinct") {
        "repetition"
    } else if id.contains("eligibility") {
        "eligibility"
    } else if id.contains("count") || id.contains("limit") || id.contains("cap") {
        "count"
    } else if id.contains("structure")
        || id.contains("euler")
        || id.contains("sequence")
        || id.contains("transition")
    {
        "connection"
    } else if id.contains("spin") {
        "spin"
    } else if id.contains("step") {
        "step"
    } else if id.contains("choreo") {
        "choreo"
    } else {
        "input"
    }
}
fn element_parts(catalog: &Catalog, code: &str) -> Vec<Value> {
    code.split('+').filter(|p|*p!="SEQ").map(|part|{
  let choice=catalog.profiles.iter().find_map(|p|check_selectable_element(&catalog.registry,&p.id,TABLE,part).ok());
  if let Some(choice)=choice {let(id,family,level)=form(&choice);json!({"code":part,"form":id,"family":family,"level":level})}
  else {json!({"code":part,"form":if part=="Eu"{"Eu"}else if matches!(part,"W"|"W(1A<<)"){"W"}else{part},"level":null})}
 }).collect()
}
fn rotation(mixed: bool) -> RotationPreference {
    if mixed {
        RotationPreference::AllowMixedDirections
    } else {
        RotationPreference::default()
    }
}
fn envelope_error(r: &RepertoireRequest, code: &str) -> ApiError {
    input_error(Some(r), code, "entries")
}
pub fn assessment_response(catalog: &Catalog, r: &AssessmentRequest) -> Result<Value, ApiError> {
    let request = &r.repertoire;
    validate_limits(request)?;
    let profile = resolve_context(catalog, &request.context)
        .map_err(|_| envelope_error(request, "context_unavailable"))?;
    let mut ids = std::collections::BTreeSet::new();
    if r.entries
        .iter()
        .any(|e| e.id.is_empty() || !ids.insert(&e.id))
    {
        return Err(envelope_error(request, "invalid_entry_id"));
    }
    let elements = r
        .entries
        .iter()
        .map(|e| {
            resolve_workspace_element(catalog, &e.element_key)
                .map_err(|_| envelope_error(request, "invalid_element_key"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let excluded = r
        .excluded_connections
        .iter()
        .map(|key| {
            resolve_workspace_element(catalog, key)
                .map_err(|_| envelope_error(request, "invalid_element_key"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    // Preserved entries can be unavailable in the newly selected category;
    // their personal settings still apply independently of category findings.
    let choices = catalog
        .profiles
        .iter()
        .flat_map(|p| selectable_elements(&catalog.registry, &p.id, TABLE).unwrap_or_default())
        .collect::<Vec<_>>();
    let (limits, step) = core_limits(&request.personal_limits, &choices);
    let submission = match r.submission.as_str() {
        "draft" => SubmissionState::Draft,
        "complete" => SubmissionState::Complete,
        _ => return Err(envelope_error(request, "invalid_submission")),
    };
    let assessment = assess_manual_bv_with_step_limit(
        &catalog.registry,
        &ManualBvRequest {
            personal_limits: limits,
            rotation_preference: rotation(r.allow_mixed_directions),
            profile: profile.id.clone(),
            scoring_table: TABLE,
            submission,
            elements: elements.clone(),
        },
        step,
    )
    .map_err(|_| envelope_error(request, "context_unavailable"))?;
    let construction = match assessment.planning_compliance() {
        Ok(PlanningCompliance::Compliant) => "compliant",
        Ok(PlanningCompliance::Noncompliant) => "noncompliant",
        Err(PlanningComplianceError::Draft) => "draft",
        Err(problem) => {
            let mut error = envelope_error(request, "assessment_input_unavailable");
            let (index, rule, reference) = match &problem {
                PlanningComplianceError::Element { element_index, .. } => {
                    (Some(*element_index), None, None)
                }
                PlanningComplianceError::InputFinding(f) => {
                    (f.element_index, Some(f.rule_id), Some(&f.source))
                }
                PlanningComplianceError::UnevaluatedRule(r) => {
                    (r.element_index, Some(r.rule_id), Some(&r.source))
                }
                _ => (None, None, None),
            };
            error.1 .0["entryId"] = json!(index.map(|i| &r.entries[i].id));
            error.1 .0["ruleId"] = json!(rule);
            error.1 .0["source"] = json!(reference.map(source));
            return Err(error);
        }
    };
    let rows=assessment.elements.iter().map(|e|{let (kind,value)=match &e.credited_base {Ok(BaseValueOutcome::Value(v))=>("value",Some(v.hundredths)),Ok(BaseValueOutcome::Ignored)=>("ignored",Some(0)),Ok(BaseValueOutcome::NoValue)=>("no_value",Some(0)),Err(_)=>("unresolved",None)};json!({"entryId":r.entries[e.element_index].id,"outcome":kind,"creditedHundredths":value,"parts":element_parts(catalog,&elements[e.element_index].code)})}).collect::<Vec<_>>();
    let findings=assessment.legality.validation.validation.findings.iter().map(|f|json!({"entryId":f.element_index.map(|i|&r.entries[i].id),"componentIndex":f.component_index,"ruleId":f.rule_id,"translationKey":finding_key(f),"parameters":{"code":f.element_index.map(|i|&elements[i].code)},"source":source(&f.source)})).collect::<Vec<_>>();
    let mut conflicts=assessment.personal_conflicts.iter().map(|f|json!({"entryId":r.entries[f.element_index].id,"componentIndex":f.component_index,"translationKey":"maximum","setting":personal_setting(&f.kind,&choices),"maximum":personal_maximum(&f.kind)})).collect::<Vec<_>>();
    for (index, element) in elements.iter().enumerate() {
        let excluded = excluded.contains(element);
        let off = element
            .code
            .split('+')
            .filter(|c| !matches!(*c, "Eu" | "SEQ"))
            .any(|code| {
                if matches!(code, "W" | "W(1A<<)") {
                    return !request
                        .personal_limits
                        .unlevelled_elements
                        .get("W")
                        .copied()
                        .unwrap_or(true);
                }
                choices
                    .iter()
                    .find(|c| c.row.notation == code)
                    .is_some_and(|c| {
                        let (id, family, n) = form(c);
                        !permits(&request.personal_limits, &id, family, n)
                    })
            });
        if (off || excluded)
            && !conflicts
                .iter()
                .any(|c| c["entryId"] == r.entries[index].id)
        {
            conflicts
                .push(json!({"entryId":r.entries[index].id,"translationKey":if excluded{"excluded"}else{"disabled"},"setting":element_parts(catalog,&element.code).iter().filter_map(|p|p["form"].as_str()).collect::<Vec<_>>() }));
        }
    }
    conflicts.extend(assessment.rotation_conflicts.iter().map(
        |f| json!({"entryId":r.entries[f.element_index].id,"translationKey":"connection","setting":"rotationPreference"}),
    ));
    Ok(
        json!({"requestId":request.request_id,"revision":request.revision,"context":request.context,"construction":construction,"elements":rows,"totalHundredths":assessment.total.exact_hundredths,"resolvedHundredths":assessment.total.resolved_hundredths,"categoryFindings":findings,"personalConflicts":conflicts}),
    )
}
pub fn connection_response(catalog: &Catalog, r: &ConnectionRequest) -> Result<Value, ApiError> {
    let request = &r.repertoire;
    validate_limits(request)?;
    let profile = resolve_context(catalog, &request.context)
        .map_err(|_| envelope_error(request, "context_unavailable"))?;
    let choices = selectable_elements(&catalog.registry, &profile.id, TABLE)
        .map_err(|_| envelope_error(request, "context_unavailable"))?;
    let (limits, _) = core_limits(&request.personal_limits, &choices);
    let all = selectable_jump_elements_with_limits(
        &catalog.registry,
        &profile.id,
        TABLE,
        rotation(r.allow_mixed_directions),
        &limits,
    )
    .map_err(|_| envelope_error(request, "context_unavailable"))?;
    let excluded = r
        .excluded_connections
        .iter()
        .map(|k| decode_element_key(&request.context, k))
        .collect::<Result<Vec<_>, _>>()?;
    let permitted = |a: &&AssumedRecognition| {
        a.code.split('+').all(|code| {
            if matches!(code, "Eu" | "SEQ") {
                return true;
            }
            if matches!(code, "W" | "W(1A<<)") {
                return request
                    .personal_limits
                    .unlevelled_elements
                    .get("W")
                    .copied()
                    .unwrap_or(true);
            }
            choices
                .iter()
                .find(|c| c.row.notation == code)
                .is_none_or(|c| {
                    let (id, f, n) = form(c);
                    permits(&request.personal_limits, &id, f, n)
                })
        })
    };
    let mut next = std::collections::BTreeSet::new();
    let mut completions = Vec::new();
    for a in all.iter().filter(permitted) {
        let parts = a
            .code
            .split('+')
            .filter(|p| *p != "SEQ")
            .collect::<Vec<_>>();
        if !parts.starts_with(&r.prefix.iter().map(String::as_str).collect::<Vec<_>>()) {
            continue;
        }
        if let Some(part) = parts.get(r.prefix.len()) {
            next.insert(part.to_string());
        } else if parts.len() >= 2 {
            completions.push(json!({"key":element_key(&request.context,a),"code":a.code,"directTransitionsAssumed":a.direct_transitions_assumed,"parts":element_parts(catalog,&a.code),"excluded":excluded.contains(a)}));
        }
    }
    Ok(
        json!({"requestId":request.request_id,"revision":request.revision,"context":request.context,"prefix":r.prefix,"next":next,"nextParts":next.iter().map(|code|element_parts(catalog,code)).collect::<Vec<_>>(),"completions":completions}),
    )
}
fn malformed_request(_: JsonRejection) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        Json(
            json!({"code":"malformed_request","category":"transport","translationKey":"errors.malformed_request"}),
        ),
    )
}
fn worker_failure() -> ApiError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(
            json!({"code":"internal_failure","category":"internal","translationKey":"errors.internal_failure"}),
        ),
    )
}
async fn assessment(
    State(catalog): State<Arc<Catalog>>,
    payload: Result<Json<AssessmentRequest>, JsonRejection>,
) -> Result<Json<Value>, ApiError> {
    let Json(r) = payload.map_err(malformed_request)?;
    tokio::task::spawn_blocking(move || assessment_response(&catalog, &r).map(Json))
        .await
        .map_err(|_| worker_failure())?
}
async fn connections(
    State(catalog): State<Arc<Catalog>>,
    payload: Result<Json<ConnectionRequest>, JsonRejection>,
) -> Result<Json<Value>, ApiError> {
    let Json(r) = payload.map_err(malformed_request)?;
    tokio::task::spawn_blocking(move || connection_response(&catalog, &r).map(Json))
        .await
        .map_err(|_| worker_failure())?
}
