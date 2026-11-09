//! THROWAWAY, LOOPBACK ONLY. Proves an integration seam; not an approved API.
use serde_json::{json, Value};
use skating_program_lab::*;
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
fn profile(key: &str) -> Result<(ProfileRegistry, RuleProfile), String> {
    Ok(match key {
        "bronze" => (
            ProfileRegistry::adult(),
            adult_profile(AdultCategory::Bronze),
        ),
        "silver" => (
            ProfileRegistry::adult(),
            adult_profile(AdultCategory::Silver),
        ),
        "gold" => (ProfileRegistry::adult(), adult_profile(AdultCategory::Gold)),
        "masters" => (
            ProfileRegistry::adult(),
            adult_profile(AdultCategory::Masters),
        ),
        "women" => (
            ProfileRegistry::senior(),
            senior_profile(SeniorCategory::Women),
        ),
        "men" => (
            ProfileRegistry::senior(),
            senior_profile(SeniorCategory::Men),
        ),
        _ => return Err("Unknown profile".into()),
    })
}
fn strings(v: &Value, key: &str) -> Result<Vec<String>, String> {
    match v.get(key) {
        None => Ok(vec![]),
        Some(Value::Array(a)) if a.len() <= 128 => a
            .iter()
            .map(|x| {
                x.as_str()
                    .map(str::to_owned)
                    .ok_or(format!("{key}: expected strings"))
            })
            .collect(),
        _ => Err(format!("{key}: expected array, at most 128 entries")),
    }
}
fn jump(k: &str) -> Result<JumpKind, String> {
    Ok(match k {
        "T" => JumpKind::ToeLoop,
        "S" => JumpKind::Salchow,
        "Lo" => JumpKind::Loop,
        "F" => JumpKind::Flip,
        "Lz" => JumpKind::Lutz,
        "A" => JumpKind::Axel,
        "W" => JumpKind::Waltz,
        _ => return Err(format!("Unknown jump kind {k}")),
    })
}
fn level(n: i64) -> Result<Level, String> {
    Ok(match n {
        0 => Level::Basic,
        1 => Level::One,
        2 => Level::Two,
        3 => Level::Three,
        4 => Level::Four,
        _ => return Err("Level must be 0 (Basic) through 4".into()),
    })
}
fn recognition(code: String) -> AssumedRecognition {
    AssumedRecognition {
        direct_transitions_assumed: code.ends_with("+SEQ"),
        code,
    }
}
struct Input {
    key: String,
    limits: PersonalLimits,
    step: Option<Level>,
    exclusions: Vec<PlanningExclusion>,
    required: Vec<RequiredPlanningElement>,
    mixed: bool,
    gate: bool,
}
fn input(v: &Value) -> Result<Input, String> {
    if !v.is_object() {
        return Err("Expected a JSON object".into());
    }
    let key = match v.get("profile") {
        None => "bronze".to_owned(),
        Some(Value::String(s)) => s.clone(),
        _ => return Err("profile must be a string".into()),
    };
    profile(&key)?;
    let mut limits = PersonalLimits::default();
    if let Some(map) = v.get("jump_max") {
        for (k, n) in map.as_object().ok_or("jump_max must be an object")? {
            let n = n
                .as_u64()
                .and_then(|n| u8::try_from(n).ok())
                .ok_or("Invalid rotation")?;
            limits.jump_rotations.push(JumpRotationLimit {
                kind: jump(k)?,
                maximum: JumpRotationClass::try_from(n)
                    .map_err(|_| "Rotation must be 1 through 5")?,
            });
        }
    }
    let reference = adult_profile(AdultCategory::Silver);
    let reference_registry = ProfileRegistry::adult();
    let choices = selectable_elements(&reference_registry, &reference.id, reference.scoring_table)
        .map_err(|e| format!("{e:?}"))?;
    if let Some(map) = v.get("spin_max") {
        for (k, n) in map.as_object().ok_or("spin_max must be an object")? {
            let abbreviation = choices
                .iter()
                .find_map(|e| match e.kind {
                    SelectableElementKind::RegularSpin(a) if e.row.notation == format!("{k}B") => {
                        Some(a)
                    }
                    _ => None,
                })
                .ok_or(format!("Unknown spin form {k}"))?;
            limits.spin_levels.push(SpinLevelLimit {
                abbreviation,
                maximum: level(n.as_i64().ok_or("Invalid spin level")?)?,
            });
        }
    }
    let step = v
        .get("step_max")
        .filter(|v| !v.is_null())
        .map(|v| level(v.as_i64().ok_or("Invalid step level")?))
        .transpose()?;
    let mut exclusions = strings(v, "exclude")?
        .into_iter()
        .map(PlanningExclusion::ExactElement)
        .collect::<Vec<_>>();
    for k in strings(v, "jump_off")? {
        exclusions.push(PlanningExclusion::JumpKind(jump(&k)?));
    }
    for k in strings(v, "spin_off")? {
        let codes = choices
            .iter()
            .filter(|e| {
                matches!(e.kind, SelectableElementKind::RegularSpin(_))
                    && e.row
                        .notation
                        .trim_end_matches('V')
                        .trim_end_matches(|c: char| c == 'B' || c.is_ascii_digit())
                        == k
            })
            .map(|e| e.row.notation.to_owned())
            .collect::<Vec<_>>();
        if codes.is_empty() {
            return Err(format!("Unknown spin form {k}"));
        }
        exclusions.extend(codes.into_iter().map(PlanningExclusion::ExactElement));
    }
    let required = match v.get("required") {
        None => vec![],
        Some(Value::Array(a)) if a.len() <= 32 => a
            .iter()
            .enumerate()
            .map(|(i, x)| {
                let code = x
                    .as_str()
                    .or_else(|| x.get("code").and_then(Value::as_str))
                    .ok_or("Required element needs a code")?
                    .to_owned();
                let id = x
                    .get("id")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("required-{i}"));
                Ok(RequiredPlanningElement {
                    id,
                    element: recognition(code),
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        _ => return Err("required must be an array of at most 32 entries".into()),
    };
    let mixed = match v.get("mixed") {
        None => false,
        Some(Value::Bool(b)) => *b,
        _ => return Err("mixed must be boolean".into()),
    };
    Ok(Input {
        key,
        limits,
        step,
        exclusions,
        required,
        mixed,
        gate: v
            .get("test_gate_collection")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}
fn rotation(mixed: bool) -> RotationPreference {
    if mixed {
        RotationPreference::AllowMixedDirections
    } else {
        RotationPreference::default()
    }
}
fn assessment(v: &Value) -> Result<Value, String> {
    let i = input(v)?;
    let (reg, p) = profile(&i.key)?;
    let r = ManualBvRequest {
        profile: p.id,
        scoring_table: p.scoring_table,
        personal_limits: i.limits,
        rotation_preference: rotation(i.mixed),
        submission: SubmissionState::Complete,
        elements: strings(v, "elements")?
            .into_iter()
            .map(recognition)
            .collect(),
    };
    let a = assess_manual_bv_with_step_limit(&reg, &r, i.step).map_err(|e| format!("{e:?}"))?;
    let verdict = match a.planning_compliance() {
        Ok(c) => json!({"verdict":format!("{c:?}")}),
        Err(e) => json!({"input_error":format!("{e:?}")}),
    };
    Ok(
        json!({"bv_hundredths":a.total.exact_hundredths,"compliance":verdict,"findings":format!("{:?}",a.legality.validation.validation.findings),"personal_conflicts":format!("{:?}",a.personal_conflicts),"scope":"planned construction; no duration or execution"}),
    )
}
enum Command {
    Prove,
    Collect(usize),
    Pause,
    Cancel,
    Dispose,
}
struct Job {
    command_lock: Mutex<()>,
    view: Mutex<Value>,
    tx: mpsc::Sender<Command>,
    flag: AtomicU8,
    busy: AtomicBool,
    done: AtomicBool,
    last: AtomicU64,
    gate: AtomicBool,
    phase: AtomicU8,
}
impl Job {
    fn patch(&self, key: &str, v: Value) {
        self.view.lock().unwrap()[key] = v;
    }
    fn phase(&self, p: PlanningSearchPhase) {
        let n = p as u8;
        if self.phase.swap(n, Ordering::SeqCst) != n {
            self.patch("phase", json!(format!("{p:?}")));
        }
    }
}
struct Control(Arc<Job>);
impl PlanningSearchControl for Control {
    fn is_cancelled(&self, p: PlanningSearchPhase) -> bool {
        self.0.phase(p);
        self.0.flag.load(Ordering::SeqCst) == 2
    }
}
impl PlanningCollectionControl for Control {
    fn action(&self, p: PlanningSearchPhase) -> PlanningCollectionAction {
        self.0.phase(p);
        // Test-only barrier: makes an in-call control race deterministic, not a speed measurement.
        if self.0.gate.swap(false, Ordering::SeqCst) {
            self.0.patch("gate_waiting", json!(true));
            let t = Instant::now();
            while self.0.flag.load(Ordering::SeqCst) == 0 && t.elapsed() < Duration::from_secs(10) {
                thread::sleep(Duration::from_millis(1));
            }
            self.0.patch("gate_waiting", json!(false));
        }
        match self.0.flag.load(Ordering::SeqCst) {
            2 => PlanningCollectionAction::Cancel,
            1 => PlanningCollectionAction::Pause,
            _ => PlanningCollectionAction::Continue,
        }
    }
}
type Jobs = Arc<Mutex<HashMap<String, Arc<Job>>>>;
fn owner(job: Arc<Job>, rx: mpsc::Receiver<Command>, i: Input, idle: u64) {
    // Registry and non-Send solver are constructed and dropped on this same owner thread.
    let (reg, p) = profile(&i.key).unwrap();
    let req = PlanningSearchRequest {
        profile: p.id,
        scoring_table: p.scoring_table,
        personal_limits: i.limits,
        rotation_preference: rotation(i.mixed),
        exclusions: i.exclusions,
        required: i.required,
    };
    let mut session = PlanningSearchSession::new(&reg, req, i.step);
    let control = Control(job.clone());
    loop {
        let cmd = match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(c) => c,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if now().saturating_sub(job.last.load(Ordering::SeqCst)) > idle * 1000 {
                    job.patch("state", json!("Expired"));
                    break;
                }
                continue;
            }
            Err(_) => break,
        };
        if matches!(cmd, Command::Dispose) {
            job.patch("state", json!("Disposed"));
            break;
        }
        let work = matches!(cmd, Command::Prove | Command::Collect(_));
        let started = Instant::now();
        let mut batch = Vec::new();
        let result = match cmd {
            Command::Prove => session.prove(&control).map(|_| ()),
            Command::Collect(n) => session
                .collect(NonZeroUsize::new(n).unwrap(), &control)
                .map(|b| batch = b.alternatives),
            Command::Pause if job.flag.load(Ordering::SeqCst) != 1 => Ok(()),
            Command::Pause | Command::Cancel => match session.state() {
                PlanningSessionState::Collecting | PlanningSessionState::Paused => session
                    .collect(NonZeroUsize::new(1).unwrap(), &control)
                    .map(|b| batch = b.alternatives),
                PlanningSessionState::Unproved if job.flag.load(Ordering::SeqCst) == 2 => {
                    session.prove(&control).map(|_| ())
                }
                _ => Ok(()),
            },
            Command::Dispose => unreachable!(),
        };
        let delivered=batch.iter().map(|p|json!({"codes":p.elements.iter().map(|e|e.code.clone()).collect::<Vec<_>>(),"bv_hundredths":p.assessment.total.exact_hundredths,"provenance":format!("{:?}",p.provenance),"compliance":format!("{:?}",p.assessment.planning_compliance())})).collect::<Vec<_>>();
        {
            let mut v = job.view.lock().unwrap();
            v["state"] = json!(format!("{:?}", session.state()));
            v["maximum_hundredths"] = json!(session.maximum_base_hundredths());
            v["last_operation_ms"] = json!(started.elapsed().as_secs_f64() * 1000.0);
            v["confirmed_count"] = json!(session.confirmed().len());
            v["error"] = result
                .err()
                .map(|e| json!(format!("{e:?}")))
                .unwrap_or(Value::Null);
            v["alternatives"].as_array_mut().unwrap().extend(delivered);
        }
        if work {
            job.busy.store(false, Ordering::SeqCst);
        }
    }
    job.done.store(true, Ordering::SeqCst);
    job.busy.store(false, Ordering::SeqCst);
}
fn create(jobs: &Jobs, v: &Value, idle: u64) -> Result<Value, String> {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let i = input(v)?;
    let (tx, rx) = mpsc::channel();
    let job = Arc::new(Job {
        command_lock: Mutex::new(()),
        view: Mutex::new(
            json!({"state":"Unproved","phase":null,"maximum_hundredths":null,"alternatives":[],"confirmed_count":0}),
        ),
        tx,
        flag: AtomicU8::new(0),
        busy: AtomicBool::new(false),
        done: AtomicBool::new(false),
        last: AtomicU64::new(now()),
        gate: AtomicBool::new(i.gate),
        phase: AtomicU8::new(255),
    });
    let id = NEXT.fetch_add(1, Ordering::SeqCst).to_string();
    {
        let mut all = jobs.lock().unwrap();
        all.retain(|_, j| !j.done.load(Ordering::SeqCst));
        if all.len() >= 2 {
            return Err("Proof capacity: two jobs; dispose one first".into());
        }
        all.insert(id.clone(), job.clone());
    }
    thread::spawn(move || owner(job, rx, i, idle));
    Ok(json!({"id":id,"state":"Unproved"}))
}
fn route(
    method: &str,
    path: &str,
    v: Value,
    jobs: &Jobs,
    idle: u64,
) -> Result<Value, (u16, String)> {
    let bad = |e: String| (400, e);
    if method == "GET" && path == "/api/profiles" {
        return Ok(json!([
            "bronze", "silver", "gold", "masters", "women", "men"
        ]));
    }
    if method == "POST" && path == "/api/elements" {
        let i = input(&v).map_err(bad)?;
        let (reg, p) = profile(&i.key).map_err(bad)?;
        let step_choices =
            selectable_elements_with_step_limit(&reg, &p.id, p.scoring_table, &i.limits, i.step)
                .map_err(|e| bad(format!("{e:?}")))?;
        let rows = selectable_elements_with_exclusions(
            &reg,
            &p.id,
            p.scoring_table,
            &i.limits,
            &i.exclusions,
        )
        .map_err(|e| bad(format!("{e:?}")))?
        .into_iter()
        .filter(|e| {
            step_choices
                .iter()
                .any(|s| s.row.notation == e.row.notation)
        })
        .collect::<Vec<_>>();
        return Ok(
            json!({"profile":format!("{:?}",p.id),"table":format!("{:?}",p.scoring_table),"codes":rows.iter().map(|e|e.row.notation).collect::<Vec<_>>()}),
        );
    }
    if method == "POST" && path == "/api/assess" {
        return assessment(&v).map_err(bad);
    }
    if method == "POST" && path == "/api/jobs" {
        return create(jobs, &v, idle).map_err(bad);
    }
    let parts = path.trim_matches('/').split('/').collect::<Vec<_>>();
    if parts.len() >= 3 && parts[0] == "api" && parts[1] == "jobs" {
        let job = jobs
            .lock()
            .unwrap()
            .get(parts[2])
            .cloned()
            .ok_or((404, "Unknown or disposed job".into()))?;
        job.last.store(now(), Ordering::SeqCst);
        if method == "GET" && parts.len() == 3 {
            let mut state = job.view.lock().unwrap().clone();
            state["busy"] = json!(job.busy.load(Ordering::SeqCst));
            return Ok(state);
        }
        if method == "DELETE" && parts.len() == 3 {
            job.flag.store(2, Ordering::SeqCst);
            let _ = job.tx.send(Command::Dispose);
            jobs.lock().unwrap().remove(parts[2]);
            return Ok(json!({"disposed":true}));
        }
        if method == "POST" && parts.len() == 4 {
            let _command_guard = job.command_lock.lock().unwrap();
            let state = job.view.lock().unwrap()["state"]
                .as_str()
                .unwrap()
                .to_owned();
            if job.done.load(Ordering::SeqCst)
                || ["Cancelled", "Failed", "Infeasible", "Exhausted", "Expired"]
                    .contains(&state.as_str())
            {
                return Err((409, format!("Terminal state {state}")));
            }
            let command = match parts[3] {
                "prove" => {
                    if state != "Unproved" {
                        return Err((409, "Proof already completed".into()));
                    }
                    Command::Prove
                }
                "collect" => {
                    if !["Collecting", "Paused"].contains(&state.as_str()) {
                        return Err((409, "Maximum not proved".into()));
                    }
                    let n = v.get("size").and_then(Value::as_u64).unwrap_or(3);
                    if !(1..=100).contains(&n) {
                        return Err((400, "Batch size must be 1..100".into()));
                    }
                    Command::Collect(n as usize)
                }
                "pause" => {
                    if !["Collecting", "Paused"].contains(&state.as_str()) {
                        return Err((
                            409,
                            "Only collection can pause; proof can be cancelled".into(),
                        ));
                    }
                    if job.flag.load(Ordering::SeqCst) != 2 {
                        job.flag.store(1, Ordering::SeqCst)
                    }
                    job.tx
                        .send(Command::Pause)
                        .map_err(|_| (410, "Owner exited".into()))?;
                    return Ok(json!({"control":"pause requested"}));
                }
                "cancel" => {
                    job.flag.store(2, Ordering::SeqCst);
                    job.tx
                        .send(Command::Cancel)
                        .map_err(|_| (410, "Owner exited".into()))?;
                    return Ok(json!({"control":"cancel requested"}));
                }
                _ => return Err((404, "Unknown action".into())),
            };
            if job.busy.swap(true, Ordering::SeqCst) {
                return Err((409, "Owner already busy".into()));
            }
            if matches!(command, Command::Collect(_)) && job.flag.load(Ordering::SeqCst) != 2 {
                job.flag.store(0, Ordering::SeqCst)
            }
            job.tx
                .send(command)
                .map_err(|_| (410, "Owner exited".into()))?;
            return Ok(json!({"accepted":true}));
        }
    }
    Err((404, "Unknown route".into()))
}
fn connection(mut stream: TcpStream, jobs: Jobs, idle: u64) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));
    let result = (|| -> Result<(u16, String, String), (u16, String)> {
        let mut data = Vec::new();
        let mut buf = [0; 4096];
        let end = loop {
            let n = stream
                .read(&mut buf)
                .map_err(|_| (400, "Read failed".into()))?;
            if n == 0 {
                return Err((400, "Incomplete HTTP request".into()));
            }
            data.extend_from_slice(&buf[..n]);
            if data.len() > 1_048_576 {
                return Err((413, "Request too large".into()));
            }
            if let Some(i) = data.windows(4).position(|w| w == b"\r\n\r\n") {
                break i + 4;
            }
        };
        let head = std::str::from_utf8(&data[..end])
            .map_err(|_| (400, "Invalid headers".into()))?
            .to_owned();
        let mut first = head.lines().next().unwrap().split_whitespace();
        let method = first.next().unwrap_or("");
        let path = first.next().unwrap_or("");
        let length = head
            .lines()
            .filter_map(|line| line.split_once(':'))
            .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
            .map(|(_, v)| v.trim().parse::<usize>())
            .transpose()
            .map_err(|_| (400, "Invalid length".into()))?
            .unwrap_or(0);
        if length > 1_000_000 {
            return Err((413, "Body too large".into()));
        }
        while data.len() < end + length {
            let n = stream
                .read(&mut buf)
                .map_err(|_| (400, "Read failed".into()))?;
            if n == 0 {
                return Err((400, "Incomplete body".into()));
            }
            data.extend_from_slice(&buf[..n]);
        }
        if method == "GET" && path == "/" {
            return Ok((
                200,
                "text/html; charset=utf-8".into(),
                include_str!("../harness.html").into(),
            ));
        }
        let value = if length == 0 {
            json!({})
        } else {
            serde_json::from_slice(&data[end..end + length])
                .map_err(|_| (400, "Malformed JSON".into()))?
        };
        route(method, path, value, &jobs, idle)
            .map(|v| (200, "application/json".into(), v.to_string()))
    })();
    let (status, kind, body) = result
        .unwrap_or_else(|(s, e)| (s, "application/json".into(), json!({"error":e}).to_string()));
    let response=format!("HTTP/1.1 {status} Result\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n{body}",body.len());
    let _ = stream.write_all(response.as_bytes());
}
fn main() {
    let port = std::env::var("SKATING_PROOF_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(8767);
    let idle = std::env::var("SKATING_PROOF_IDLE_SECONDS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(120)
        .clamp(2, 600);
    let listener = TcpListener::bind(("127.0.0.1", port)).expect("loopback proof listener");
    let jobs: Jobs = Arc::new(Mutex::new(HashMap::new()));
    println!("Service proof on http://127.0.0.1:{port}; idle expiry {idle}s");
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let jobs = jobs.clone();
                thread::spawn(move || connection(s, jobs, idle));
            }
            Err(e) => eprintln!("accept: {e}"),
        }
    }
}
