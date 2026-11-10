//! Native owners keep the registry, solver and borrowed session off the HTTP runtime.
use super::*;
use axum::{
    extract::{Path, Query},
    http::HeaderMap,
    routing::delete,
};
use std::{
    collections::BTreeSet,
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering},
        Condvar, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
const LEASE: Duration = Duration::from_secs(15 * 60);
/// Monotonic service time. Injection changes tests' clock, never the 15-minute policy.
pub trait SearchClock: Send + Sync {
    fn now(&self) -> Duration;
}
struct MonotonicClock(Instant);
impl SearchClock for MonotonicClock {
    fn now(&self) -> Duration {
        self.0.elapsed()
    }
}
pub struct SearchServiceOptions {
    pub capacity: usize,
    pub clock: Arc<dyn SearchClock>,
}
impl Default for SearchServiceOptions {
    fn default() -> Self {
        Self {
            capacity: 2,
            clock: Arc::new(MonotonicClock(Instant::now())),
        }
    }
}
fn opaque() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("operating system randomness");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn error(status: StatusCode, code: &str) -> ApiError {
    (
        status,
        Json(
            json!({"code":code,"category":if status.is_server_error(){"processing"}else{"input"},"translationKey":format!("errors.{code}")}),
        ),
    )
}
fn correlated(
    r: &SearchRequest,
    mut e: ApiError,
    field: Option<&str>,
    entry: Option<&str>,
) -> ApiError {
    e.1 .0["requestId"] = json!(r.repertoire.request_id);
    e.1 .0["revision"] = json!(r.repertoire.revision);
    if let Some(f) = field {
        e.1 .0["field"] = json!(f);
    }
    if let Some(id) = entry {
        e.1 .0["entryId"] = json!(id);
    }
    if e.0 == StatusCode::SERVICE_UNAVAILABLE {
        e.1 .0["retryAfterSeconds"] = json!(1);
    }
    e
}
fn unavailable() -> ApiError {
    error(StatusCode::NOT_FOUND, "session_unavailable")
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SearchRequest {
    #[serde(flatten)]
    repertoire: RepertoireRequest,
    creation_key: String,
    required_elements: Vec<ProgramEntry>,
    #[serde(default)]
    allow_mixed_directions: bool,
    #[serde(default)]
    excluded_connections: Vec<String>,
}
// Receipts intentionally outlive public handles: disposal must never make a lost
// creation response allocate a second owner. The namespace rejects pre-restart keys.
struct Receipt {
    fingerprint: Option<String>,
    job: Option<Arc<Job>>,
}
struct Manager {
    namespace: String,
    catalog: Arc<Catalog>,
    jobs: Mutex<BTreeMap<String, Arc<Job>>>,
    receipts: Mutex<BTreeMap<String, Receipt>>,
    live: AtomicUsize,
    options: SearchServiceOptions,
}
struct Job {
    id: String,
    token: String,
    request: SearchRequest,
    control: AtomicU8,
    disposed: AtomicBool,
    last_contact: Mutex<Duration>,
    data: Mutex<Data>,
    join: Mutex<Option<JoinHandle<()>>>,
    wake: Condvar,
}
struct Data {
    status: &'static str,
    phase: &'static str,
    maximum: Option<u32>,
    results: Vec<Value>,
    failure: Option<Value>,
    diagnostics: Vec<Value>,
    commands: BTreeMap<String, String>,
}
impl Manager {
    fn new(catalog: Arc<Catalog>, options: SearchServiceOptions) -> Arc<Self> {
        let manager = Arc::new(Self {
            namespace: opaque(),
            catalog,
            jobs: Mutex::default(),
            receipts: Mutex::default(),
            live: AtomicUsize::new(0),
            options,
        });
        let weak = Arc::downgrade(&manager);
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(250));
            let Some(m) = weak.upgrade() else { break };
            m.sweep();
        });
        manager
    }
    fn sweep(&self) {
        let mut jobs = self.jobs.lock().unwrap();
        jobs.retain(|_, job| {
            if self
                .options
                .clock
                .now()
                .saturating_sub(*job.last_contact.lock().unwrap())
                >= LEASE
            {
                job.disposed.store(true, Ordering::Release);
                Control::Cancel.store(&job.control);
                job.wake.notify_all();
                false
            } else {
                true
            }
        });
        drop(jobs);
        // Join only finished native owners. Disposed or expired jobs still consume capacity
        // until the OS thread really terminates; no detached solver escapes the bound.
        for receipt in self.receipts.lock().unwrap().values_mut() {
            let unavailable = if let Some(job) = &receipt.job {
                let mut join = job.join.lock().unwrap();
                if join.as_ref().is_some_and(|j| j.is_finished()) {
                    let _ = join.take().unwrap().join();
                    self.live.fetch_sub(1, Ordering::AcqRel);
                }
                join.is_none() && job.disposed.load(Ordering::Acquire)
            } else {
                false
            };
            if unavailable {
                receipt.job = None;
                receipt.fingerprint = None;
            }
        }
    }
    fn authorize(&self, id: &str, headers: &HeaderMap) -> Result<Arc<Job>, ApiError> {
        self.sweep();
        let job = self
            .jobs
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(unavailable)?;
        let provided = headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .unwrap_or("");
        // Fixed-length unpredictable capabilities; compare every byte.
        if provided.len() != job.token.len()
            || provided
                .bytes()
                .zip(job.token.bytes())
                .fold(0u8, |a, (x, y)| a | (x ^ y))
                != 0
        {
            return Err(unavailable());
        }
        *job.last_contact.lock().unwrap() = self.options.clock.now();
        Ok(job)
    }
}
impl Drop for Manager {
    fn drop(&mut self) {
        for job in self
            .receipts
            .get_mut()
            .unwrap()
            .values()
            .filter_map(|r| r.job.as_ref())
        {
            Control::Cancel.store(&job.control);
            job.wake.notify_all();
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Control {
    Running = 0,
    Pause = 1,
    Cancel = 2,
    Continue = 3,
}
impl Control {
    fn load(atomic: &AtomicU8) -> Self {
        match atomic.load(Ordering::Acquire) {
            0 => Self::Running,
            1 => Self::Pause,
            2 => Self::Cancel,
            3 => Self::Continue,
            _ => unreachable!("private control value"),
        }
    }
    fn store(self, atomic: &AtomicU8) {
        atomic.store(self as u8, Ordering::Release);
    }
    fn resume(atomic: &AtomicU8) -> bool {
        atomic
            .compare_exchange(
                Self::Continue as u8,
                Self::Running as u8,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }
}
impl PlanningSearchControl for Job {
    fn is_cancelled(&self, _: PlanningSearchPhase) -> bool {
        Control::load(&self.control) == Control::Cancel
    }
}
impl PlanningCollectionControl for Job {
    fn action(&self, _: PlanningSearchPhase) -> PlanningCollectionAction {
        match Control::load(&self.control) {
            Control::Cancel => PlanningCollectionAction::Cancel,
            Control::Pause => PlanningCollectionAction::Pause,
            _ => PlanningCollectionAction::Continue,
        }
    }
}
fn snapshot(job: &Job, after: usize, limit: usize) -> Value {
    let d = job.data.lock().unwrap();
    let end = after.saturating_add(limit).min(d.results.len());
    let after = after.min(d.results.len());
    json!({"id":job.id,"requestId":job.request.repertoire.request_id,"revision":job.request.repertoire.revision,"context":job.request.repertoire.context,"status":d.status,"phase":d.phase,"proof":if d.maximum.is_some(){"proved"}else{"unproved"},"maximumBaseHundredths":d.maximum,"confirmedCount":d.results.len(),"collectionComplete":d.status=="completed","controlRequested":match (d.status,Control::load(&job.control)){("running"|"paused",Control::Cancel)=>Some("cancel"),("running",Control::Pause)=>Some("pause"),("paused",Control::Continue)=>Some("continue"),_=>None},"cursor":end,"results":d.results[after..end],"failure":d.failure,"diagnostics":d.diagnostics})
}
pub(super) fn router(catalog: Arc<Catalog>, options: SearchServiceOptions) -> Router {
    Router::new()
        .route("/api/v1/searches/identity", get(identity))
        .route("/api/v1/searches", post(create))
        .route("/api/v1/searches/{id}/updates", get(updates))
        .route("/api/v1/searches/{id}/cancel", post(cancel))
        .route("/api/v1/searches/{id}/pause", post(pause))
        .route("/api/v1/searches/{id}/continue", post(resume))
        .route("/api/v1/searches/{id}", delete(dispose))
        .with_state(Manager::new(catalog, options))
}
async fn identity(State(m): State<Arc<Manager>>) -> Json<Value> {
    Json(json!({"creationNamespace":m.namespace}))
}
async fn create(
    State(m): State<Arc<Manager>>,
    payload: Result<Json<SearchRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let Json(r) = payload.map_err(|_| error(StatusCode::BAD_REQUEST, "malformed_json"))?;
    let prefix = format!("{}:", m.namespace);
    if !r
        .creation_key
        .strip_prefix(&prefix)
        .is_some_and(|s| s.len() == 64 && s.bytes().all(|c| c.is_ascii_hexdigit()))
    {
        return Err(correlated(
            &r,
            error(StatusCode::GONE, "session_unavailable"),
            Some("creationKey"),
            None,
        ));
    }
    let fingerprint =
        serde_json::to_string(&r).map_err(|_| error(StatusCode::BAD_REQUEST, "malformed_json"))?;
    m.sweep();
    let mut receipts = m.receipts.lock().unwrap();
    if let Some(receipt) = receipts.get(&r.creation_key) {
        let job = receipt
            .job
            .as_ref()
            .ok_or_else(|| correlated(&r, unavailable(), None, None))?;
        if job.disposed.load(Ordering::Acquire) {
            return Err(correlated(&r, unavailable(), None, None));
        }
        if receipt.fingerprint.as_ref() != Some(&fingerprint) {
            return Err(correlated(
                &r,
                error(StatusCode::CONFLICT, "creation_conflict"),
                Some("creationKey"),
                None,
            ));
        }
        *job.last_contact.lock().unwrap() = m.options.clock.now();
        return Ok((StatusCode::ACCEPTED, Json(created(job))));
    }
    validate_limits(&r.repertoire).map_err(|e| correlated(&r, e, None, None))?;
    resolve_context(&m.catalog, &r.repertoire.context)
        .map_err(|e| correlated(&r, e, Some("context"), None))?;
    let mut ids = BTreeSet::new();
    for entry in &r.required_elements {
        if entry.id.is_empty() || !ids.insert(&entry.id) {
            return Err(correlated(
                &r,
                input_error(Some(&r.repertoire), "invalid_entry_id", "requiredElements"),
                None,
                Some(&entry.id),
            ));
        }
        resolve_workspace_element(&m.catalog, &entry.element_key)
            .map_err(|e| correlated(&r, e, Some("requiredElements"), Some(&entry.id)))?;
    }
    for key in &r.excluded_connections {
        resolve_connection_exclusion(&m.catalog, key)
            .map_err(|e| correlated(&r, e, Some("excludedConnections"), None))?;
    }
    if m.live.load(Ordering::Acquire) >= m.options.capacity {
        return Err(correlated(
            &r,
            error(StatusCode::SERVICE_UNAVAILABLE, "search_capacity"),
            None,
            None,
        ));
    }
    let job = Arc::new(Job {
        id: opaque(),
        token: opaque(),
        request: r,
        control: AtomicU8::new(Control::Running as u8),
        disposed: AtomicBool::new(false),
        last_contact: Mutex::new(m.options.clock.now()),
        data: Mutex::new(Data {
            status: "running",
            phase: "proving",
            maximum: None,
            results: vec![],
            failure: None,
            diagnostics: vec![],
            commands: BTreeMap::new(),
        }),
        join: Mutex::new(None),
        wake: Condvar::new(),
    });
    m.live.fetch_add(1, Ordering::AcqRel);
    let owner = job.clone();
    let catalog = m.catalog.clone();
    let join=thread::Builder::new().name("planning-search".into()).spawn(move||{
  let outcome=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{let local_catalog=(*catalog).clone();run(&local_catalog,&owner)}));
  if outcome.is_err(){let mut d=owner.data.lock().unwrap_or_else(|e|e.into_inner());d.status="failed";d.phase="terminal";d.failure=Some(json!({"code":"internal_failure","category":"processing","translationKey":"errors.internal_failure","requestId":owner.request.repertoire.request_id,"revision":owner.request.repertoire.revision}));}
 }).map_err(|_|{m.live.fetch_sub(1,Ordering::AcqRel);correlated(&job.request,error(StatusCode::SERVICE_UNAVAILABLE,"search_capacity"),None,None)})?;
    *job.join.lock().unwrap() = Some(join);
    m.jobs.lock().unwrap().insert(job.id.clone(), job.clone());
    receipts.insert(
        job.request.creation_key.clone(),
        Receipt {
            fingerprint: Some(fingerprint),
            job: Some(job.clone()),
        },
    );
    Ok((StatusCode::ACCEPTED, Json(created(&job))))
}
fn created(job: &Job) -> Value {
    json!({"id":job.id,"accessToken":job.token,"requestId":job.request.repertoire.request_id,"revision":job.request.repertoire.revision,"status":"running","phase":"proving"})
}
#[derive(Default, Deserialize)]
struct Page {
    after: Option<usize>,
    limit: Option<usize>,
}
async fn updates(
    State(m): State<Arc<Manager>>,
    Path(id): Path<String>,
    Query(page): Query<Page>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let job = m.authorize(&id, &headers)?;
    let after = page.after.unwrap_or(0);
    if after > job.data.lock().unwrap().results.len() {
        return Err(correlated(
            &job.request,
            error(StatusCode::BAD_REQUEST, "invalid_cursor"),
            Some("after"),
            None,
        ));
    }
    Ok(Json(snapshot(
        &job,
        after,
        page.limit.unwrap_or(25).min(100),
    )))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Command {
    command_id: String,
}
async fn pause(
    State(m): State<Arc<Manager>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(command): Json<Command>,
) -> Result<Json<Value>, ApiError> {
    control_command(&m, &id, &headers, command, "pause")
}
async fn resume(
    State(m): State<Arc<Manager>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(command): Json<Command>,
) -> Result<Json<Value>, ApiError> {
    control_command(&m, &id, &headers, command, "continue")
}
async fn cancel(
    State(m): State<Arc<Manager>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(command): Json<Command>,
) -> Result<Json<Value>, ApiError> {
    control_command(&m, &id, &headers, command, "cancel")
}
fn control_command(
    m: &Manager,
    id: &str,
    headers: &HeaderMap,
    command: Command,
    action: &str,
) -> Result<Json<Value>, ApiError> {
    let job = m.authorize(id, headers)?;
    if command.command_id.is_empty() {
        return Err(error(StatusCode::BAD_REQUEST, "invalid_command"));
    }
    let mut d = job.data.lock().unwrap();
    if let Some(previous) = d.commands.get(&command.command_id) {
        if previous != action {
            return Err(error(StatusCode::CONFLICT, "command_conflict"));
        }
    } else {
        let valid = match action {
            "pause" => {
                d.status == "running"
                    && d.phase == "collecting"
                    && Control::load(&job.control) == Control::Running
            }
            "continue" => d.status == "paused" && Control::load(&job.control) == Control::Pause,
            "cancel" => true,
            _ => false,
        };
        if !valid {
            return Err(error(StatusCode::CONFLICT, "invalid_transition"));
        }
        d.commands.insert(command.command_id, action.into());
        match action {
            "pause" => Control::Pause,
            "cancel" => Control::Cancel,
            _ => Control::Continue,
        }
        .store(&job.control);
        job.wake.notify_all();
    }
    drop(d);
    Ok(Json(snapshot(&job, 0, 0)))
}
async fn dispose(
    State(m): State<Arc<Manager>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let job = m.authorize(&id, &headers)?;
    job.disposed.store(true, Ordering::Release);
    Control::Cancel.store(&job.control);
    job.wake.notify_all();
    m.jobs.lock().unwrap().remove(&id);
    Ok(Json(json!({"disposed":true})))
}
fn failure(job: &Job, code: &str, ids: Vec<String>) {
    let mut d = job.data.lock().unwrap();
    d.status = "failed";
    d.phase = "terminal";
    d.failure = Some(
        json!({"code":code,"category":if code=="request_invalid"{"input"}else{"processing"},"translationKey":format!("errors.{code}"),"entryIds":ids,"requestId":job.request.repertoire.request_id,"revision":job.request.repertoire.revision}),
    );
}
fn search_error(job: &Job, error: PlanningSessionError) {
    let (code, indices) = match error {
        PlanningSessionError::Search(PlanningSearchError::InvalidRequest {
            diagnostics, ..
        }) => {
            job.data.lock().unwrap().diagnostics = diagnostic_values(job, &diagnostics);
            (
                "request_invalid",
                diagnostics
                    .into_iter()
                    .flat_map(|d| d.required_input_indices)
                    .collect(),
            )
        }
        PlanningSessionError::Search(PlanningSearchError::PersonalLimitConflict(conflicts)) => (
            "request_invalid",
            conflicts.into_iter().map(|c| c.element_index).collect(),
        ),
        PlanningSessionError::Search(PlanningSearchError::RotationPreferenceConflict(
            conflicts,
        )) => (
            "request_invalid",
            conflicts.into_iter().map(|c| c.element_index).collect(),
        ),
        PlanningSessionError::Search(
            PlanningSearchError::UnsupportedCoverage { .. } | PlanningSearchError::Profile(_),
        ) => ("unsupported_search", vec![]),
        _ => ("internal_failure", vec![]),
    };
    failure(
        job,
        code,
        indices
            .into_iter()
            .filter_map(|i| job.request.required_elements.get(i).map(|e| e.id.clone()))
            .collect(),
    );
}
fn diagnostic_values(job: &Job, diagnostics: &[PlanningDiagnostic]) -> Vec<Value> {
    diagnostics.iter().map(|d|json!({"translationKey":"errors.planning_constraint","ruleId":d.rule_id,"entryIds":d.required_input_indices.iter().filter_map(|i|job.request.required_elements.get(*i).map(|e|&e.id)).collect::<Vec<_>>()})).collect()
}
fn run(catalog: &Catalog, job: &Job) {
    let r = &job.request;
    let profile = match resolve_context(catalog, &r.repertoire.context) {
        Ok(p) => p,
        Err(_) => {
            failure(job, "request_invalid", vec![]);
            return;
        }
    };
    let choices = catalog
        .profiles
        .iter()
        .flat_map(|p| selectable_elements(&catalog.registry, &p.id, TABLE).unwrap_or_default())
        .collect::<Vec<_>>();
    let (limits, step) = core_limits(&r.repertoire.personal_limits, &choices);
    let mut exclusions = Vec::new();
    for c in &choices {
        let (id, f, n) = form(c);
        if !permits(&r.repertoire.personal_limits, &id, f, n) {
            match c.kind {
                SelectableElementKind::SoloJump { kind, .. } => {
                    if r.repertoire
                        .personal_limits
                        .jumps
                        .get(&id)
                        .is_some_and(|l| !l.enabled)
                    {
                        exclusions.push(PlanningExclusion::JumpKind(kind));
                    }
                }
                _ => exclusions.push(PlanningExclusion::ExactElement(c.row.notation.into())),
            }
        }
    }
    if r.repertoire.personal_limits.unlevelled_elements.get("W") == Some(&false) {
        exclusions.push(PlanningExclusion::JumpKind(JumpKind::Waltz));
    }
    for key in &r.excluded_connections {
        match resolve_connection_exclusion(catalog, key) {
            Ok(a) => exclusions.push(PlanningExclusion::ExactElement(a.code)),
            Err(_) => {
                failure(job, "request_invalid", vec![]);
                return;
            }
        }
    }
    let mut required = vec![];
    for e in &r.required_elements {
        match resolve_workspace_element(catalog, &e.element_key) {
            Ok(element) => required.push(RequiredPlanningElement {
                id: e.id.clone(),
                element,
            }),
            Err(_) => {
                failure(job, "request_invalid", vec![e.id.clone()]);
                return;
            }
        }
    }
    let mut session = PlanningSearchSession::new(
        &catalog.registry,
        PlanningSearchRequest {
            personal_limits: limits,
            rotation_preference: rotation(r.allow_mixed_directions),
            profile: profile.id.clone(),
            scoring_table: TABLE,
            required,
            exclusions,
        },
        step,
    );
    if let Err(e) = session.prove(job) {
        search_error(job, e);
        return;
    }
    loop {
        {
            let mut d = job.data.lock().unwrap();
            d.maximum = session.maximum_base_hundredths();
            d.diagnostics = diagnostic_values(job, session.diagnostics());
            match session.state() {
                PlanningSessionState::Collecting => {
                    d.status = "running";
                    d.phase = "collecting"
                }
                PlanningSessionState::Paused => {
                    d.status = "paused";
                    d.phase = "collecting"
                }
                PlanningSessionState::Exhausted => {
                    d.status = "completed";
                    d.phase = "terminal"
                }
                PlanningSessionState::Infeasible => {
                    d.status = "infeasible";
                    d.phase = "terminal"
                }
                PlanningSessionState::Cancelled => {
                    d.status = "cancelled";
                    d.phase = "terminal"
                }
                _ => {
                    d.status = "failed";
                    d.phase = "terminal"
                }
            }
        }
        if session.state() == PlanningSessionState::Paused {
            let mut d = job.data.lock().unwrap();
            while Control::load(&job.control) == Control::Pause {
                // The mutex synchronizes the predicate and notification. Timed wake
                // also lets disposal/lease cancellation reach a sleeping native owner.
                d = job
                    .wake
                    .wait_timeout(d, Duration::from_millis(250))
                    .unwrap()
                    .0;
            }
            if Control::resume(&job.control) {
                d.status = "running";
            }
            drop(d);
        } else if session.state() != PlanningSessionState::Collecting {
            return;
        }
        let batch = match session.collect(NonZeroUsize::new(25).unwrap(), job) {
            Ok(b) => b,
            Err(e) => {
                search_error(job, e);
                return;
            }
        };
        for program in batch.alternatives {
            let entries=program.elements.iter().zip(&program.provenance).map(|(element,provenance)|{let required=match provenance{PlanningProvenance::Required{id,..}=>Some(id),PlanningProvenance::Added=>None};json!({"elementKey":element_key(&r.repertoire.context,element),"code":element.code,"requiredId":required})}).collect::<Vec<_>>();
            let assessment_request = AssessmentRequest {
                repertoire: r.repertoire.clone(),
                entries: program
                    .elements
                    .iter()
                    .enumerate()
                    .map(|(i, e)| ProgramEntry {
                        id: match &program.provenance[i] {
                            PlanningProvenance::Required { id, .. } => id.clone(),
                            PlanningProvenance::Added => opaque(),
                        },
                        element_key: element_key(&r.repertoire.context, e),
                    })
                    .collect(),
                submission: "complete".into(),
                allow_mixed_directions: r.allow_mixed_directions,
                excluded_connections: r.excluded_connections.clone(),
            };
            let assessment = match assessment_response(catalog, &assessment_request) {
                Ok(a)
                    if a["construction"] == "compliant"
                        && a["personalConflicts"]
                            .as_array()
                            .is_some_and(|v| v.is_empty())
                        && a["totalHundredths"] == json!(session.maximum_base_hundredths()) =>
                {
                    a
                }
                _ => {
                    failure(job, "internal_failure", vec![]);
                    return;
                }
            };
            let mut d = job.data.lock().unwrap();
            let id = format!("{}:{}", job.id, d.results.len() + 1);
            d.results
                .push(json!({"id":id,"entries":entries,"assessment":assessment}));
        }
    }
}
