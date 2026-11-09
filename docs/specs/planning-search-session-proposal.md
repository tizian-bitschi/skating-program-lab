# Public planning search session proposal (#142)

Status: approved by the requesting user on 2026-11-09 before implementation.
Parent: [#140](https://github.com/tizian-bitschi/skating-program-lab/issues/140).
Contract: [#142](https://github.com/tizian-bitschi/skating-program-lab/issues/142).
Baseline: `c295da4`. The approved interface below is implemented by #142; this
document retains the approval contract. #143 records workflow acceptance.

## Proposed public interface

Export these additions alongside the existing planning search types. Existing
types, signatures and enum variants stay unchanged. The new interface is
available under the same feature conditions as the existing public search seam.

```rust
use std::num::NonZeroUsize;

pub struct PlanningSearchSession<'registry> { /* private fields */ }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanningSessionState {
    Unproved,
    Collecting,
    Paused,
    Infeasible,
    Exhausted,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanningCollectionAction {
    Continue,
    Pause,
    Cancel,
}

pub trait PlanningCollectionControl {
    fn action(&self, phase: PlanningSearchPhase) -> PlanningCollectionAction;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanningSessionError {
    Search(PlanningSearchError),
    InvalidState { state: PlanningSessionState },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningSearchBatch {
    pub alternatives: Vec<PlanningProgram>,
    pub state: PlanningSessionState,
}

impl<'registry> PlanningSearchSession<'registry> {
    pub fn new(
        registry: &'registry ProfileRegistry,
        request: PlanningSearchRequest,
        maximum_step_level: Option<Level>,
    ) -> Self;

    pub fn state(&self) -> PlanningSessionState;
    pub fn maximum_base_hundredths(&self) -> Option<u32>;
    pub fn confirmed(&self) -> &[PlanningProgram];
    pub fn diagnostics(&self) -> &[PlanningDiagnostic];

    pub fn prove(
        &mut self,
        control: &impl PlanningSearchControl,
    ) -> Result<PlanningSessionState, PlanningSessionError>;

    pub fn collect(
        &mut self,
        batch_size: NonZeroUsize,
        control: &impl PlanningCollectionControl,
    ) -> Result<PlanningSearchBatch, PlanningSessionError>;
}
```

`new` stores the owned request and step ceiling and borrows the registry
immutably for the session lifetime. It performs no domain construction or solving.
The stored inputs have no mutable accessor. Profile/table identity, required
IDs and occurrences, exclusions, rotation preference, personal jump/spin limits
and the separate step ceiling stay fixed. Changing any input requires a new
session. The registry borrow prevents safe mutation of its profile definitions
while this session exists. No background thread or execution begins in `new`.

## Proof and observable lifecycle

`prove` is synchronous and valid only in `Unproved`. It runs existing validation,
domain construction, model construction and maximum proof with the existing
phase polls, including validated `IncumbentFound` notifications. Callers use
`PlanningSearchControl` to observe those phases and request cooperative
cancellation, for example through an atomic flag. No wall-clock or node limit is
added. `Unproved` means proof has not completed; phase polls expose its active
work. Session methods require exclusive access, so callers do not query the
session from their control callback.

Before proof completes, `confirmed()` is empty and the maximum getter returns
`None`. Solver proof and fresh shared manual assessment of its representative
must both succeed before the maximum getter becomes `Some(M)` and the state
becomes `Collecting`. Keep that representative privately as the first pending
collection candidate; proof does not deliver an alternative. Poll
`MaximumProved` after committing that proof and validated representative. If
that poll requests cancellation, return `Cancelled` with `Some(M)` and no
delivered programs. Earlier cancellation returns `Cancelled` with `None`.
Cancellation during initial reconstruction exposes no program or maximum.

A supported contradiction or solver infeasibility proof returns `Infeasible`;
the diagnostic getter retains its indexed diagnostics. Request/profile/coverage
errors return `Err(Search(error))` and enter `Failed`, preserving the existing
error distinctions and diagnostics inside the error. No error establishes
infeasibility. `diagnostics()` is empty except for `Infeasible`; callers retain
the returned error for request or implementation failures.

## Incremental collection and pause

`collect` is synchronous and valid in `Collecting` or `Paused`. Calling it in
`Paused` resumes the same session and returns to `Collecting` before polling the
control. It delivers at most `batch_size.get()` new alternatives. No fixed total
cap applies. Every delivery passes fresh shared reconstruction/manual assessment
against the fixed request, step ceiling and proved maximum.

Poll the collection control at `Alternatives` and `Reconstruction` checkpoints,
and from the solver termination condition using `Alternatives`. Latch the first
observed `Pause` or `Cancel` for that call. `Pause` returns a possibly empty batch
with state `Paused`; `Cancel` returns a possibly empty batch with state
`Cancelled`. Both preserve all previously delivered programs and all programs
committed earlier in the current batch. Neither means exhaustion. A request can
arrive during a SAT call; Pumpkin stops cooperatively at its next termination
poll. A pause imposes no deadline or guarantee of a bounded response time.

Commit delivery atomically: finish assessment, record the complete count vector
as delivered, append the program to `confirmed()`, and append its clone to the
current batch. Never poll between those changes. An interrupted reconstruction
keeps its copied candidate count vector pending for resumption and delivers
nothing from that reconstruction. Cancellation leaves pending work inaccessible.

A full batch returns `Collecting` immediately, without a speculative extra solve.
This holds when the entire solution set happens to equal that batch size. Only a
later unsatisfiable fixed-objective solve with every delivered vector excluded
sets `Exhausted`. A short or empty batch carries `Exhausted` only after that proof.
The returned state equals `session.state()` on return. A control request that
was never polled does not override a completed batch or exhaustion proof.

`confirmed()` retains all delivered alternatives in delivery order, including
after pause, cancellation, exhaustion or failure. `maximum_base_hundredths()`
retains `Some(M)` after any post-proof terminal state. Batches contain only new
deliveries, so callers append batch alternatives, or reread `confirmed()` as a
snapshot, without combining both. Callers own returned batch values.

## Identity, errors and terminal states

Use the approved complete whole-element count-vector identity from the planning
search specification: exact compound order/code and transition assumptions,
and occurrence multiplicities remain distinct; placement permutations,
auxiliary solver assignments and required-ID swaps do not create alternatives.
Preserve deterministic required-ID assignment and all indexed conflict
diagnostics. No result is delivered twice within one session, including across
pause/resume boundaries. Ordering is deterministic where existing repertoire
ordering determines it, but callers must not depend on ordering or poll counts.

`Infeasible`, `Exhausted`, `Cancelled` and `Failed` are terminal. `prove` or
`collect` in any disallowed state returns `InvalidState` without changing state
or retained data. In particular, cancelled proof cannot resume. A collection
search/reconstruction error enters `Failed` and returns `Search(error)`;
confirmed programs remain accessible. Programs committed during that failed
call are available through `confirmed()` even though no batch is returned.
There is no restart, clear, mutable request, serialization or session clone API.
Dropping the session releases its solver and memory. It provides no persistence
after close/reload, and promises no `Send`/`Sync`, HTTP, UI or WASM integration.

## Concrete solver ownership

Keep the prepared atom repertoire, count/objective variable IDs, Pumpkin
`Solver`, concrete `CountBrancher`, `ResolutionResolver`, proved maximum,
pending candidate, delivered count vectors and collection cursors in private
owned fields. Borrow the registry; own the request. Build solver/brancher once.
After `optimise(LinearSatUnsat)` proves M, constrain that same solver to objective
M and retain it for all collection calls. Preserve existing sound conditional
pruning and count-vector exclusion logic. The smallest complete collection path
uses repeated `satisfy` calls plus those exclusions; compound-alias traversal is
an optional internal speed optimization, not part of the public contract.

Pumpkin 0.5.0 reexports `pumpkin-core` 0.5.0. Its
`api/solver.rs::satisfy` returns a result borrowing solver, brancher and resolver;
`api/outputs/mod.rs::SatisfactionResult` documents that dropping a satisfiable
result resets the solver to root. `Unknown` also restores root before return.
Therefore never store `SatisfactionResult`, `Satisfiable`, `SolutionReference`
or `SolutionIterator` inside the session. In a short scope copy all atomic counts
and objective value into owned Rust data, then drop the borrowed result before
reconstruction, posting exclusions, returning or taking another solver borrow.
This avoids self-referential fields and needs no unsafe code.

A paused SAT invocation loses its active decision trail because Pumpkin resets
to root. The same solver, retained learned constraints, brancher/resolver,
fixed objective, posted exclusions, pending count vector and confirmed delivery
history survive. Resume continues the same prepared search problem without
rebuilding domain/model or losing confirmed work. Exact instruction-level solver
continuation is outside this contract. Ordinary batch boundaries stop between
SAT calls. Every delivered vector is posted as an exclusion before the next SAT
call; retain a pending exclusion if pause happens before posting it.

## Compatibility and acceptance proof

Keep `search_maximum_bv`, `search_maximum_bv_with_step_limit`,
`PlanningSearchOutcome`, `PlanningMaximum` and `PlanningAlternativeStatus`
unchanged. Legacy functions retain their five-alternative cap and legacy
`LimitReached` meaning. They can later call shared private helpers, but must not
change their observable cancellation or maximum-proof behavior. No public
request field or enum variant is added to an existing type.

Test the new public seam using an independently enumerated finite fixture with
more than five optima and exact set equality. Cover infeasible/invalid input,
one optimum, exactly one batch, multiple batches, explicit empty pause, pause
during reconstruction/solving, cancellation before and after proof, retained
confirmed results, invalid state calls, and a final separate exhaustion call.
Compare paused/resumed and uninterrupted sets without asserting poll counts or
result order. Fresh public manual assessment must reproduce M for every result
with required IDs/multiplicities, indivisible compounds, exclusions and ceilings.
Retain legacy tests and run repository-prescribed build, formatting, static
analysis, default and minimal-feature tests after implementation.
