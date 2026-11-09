//! Incremental, session-local access to all proved optimal compositions.
use super::*;
use std::{cell::Cell, num::NonZeroUsize};

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

/// Polled cooperatively during collection, including solver termination checks.
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

/// Owns one fixed request and its prepared solver until dropped.
/// A full batch does not prove exhaustion; call `collect` again for that proof.
/// Persistence, thread transfer and instruction-level solver continuation are not promised.
pub struct PlanningSearchSession<'registry> {
    registry: &'registry ProfileRegistry,
    request: PlanningSearchRequest,
    maximum_step_level: Option<Level>,
    state: PlanningSessionState,
    maximum: Option<u32>,
    confirmed: Vec<PlanningProgram>,
    diagnostics: Vec<PlanningDiagnostic>,
    prepared: Option<model::Prepared>,
}
impl<'registry> PlanningSearchSession<'registry> {
    pub fn new(
        registry: &'registry ProfileRegistry,
        request: PlanningSearchRequest,
        maximum_step_level: Option<Level>,
    ) -> Self {
        Self {
            registry,
            request,
            maximum_step_level,
            state: PlanningSessionState::Unproved,
            maximum: None,
            confirmed: Vec::new(),
            diagnostics: Vec::new(),
            prepared: None,
        }
    }
    pub fn state(&self) -> PlanningSessionState {
        self.state
    }
    pub fn maximum_base_hundredths(&self) -> Option<u32> {
        self.maximum
    }
    pub fn confirmed(&self) -> &[PlanningProgram] {
        &self.confirmed
    }
    pub fn diagnostics(&self) -> &[PlanningDiagnostic] {
        &self.diagnostics
    }

    pub fn prove(
        &mut self,
        control: &impl PlanningSearchControl,
    ) -> Result<PlanningSessionState, PlanningSessionError> {
        if self.state != PlanningSessionState::Unproved {
            return Err(PlanningSessionError::InvalidState { state: self.state });
        }
        match search_prepared(
            self.registry,
            &self.request,
            self.maximum_step_level,
            control,
            Some(&mut self.prepared),
        ) {
            Err(error) => {
                self.state = PlanningSessionState::Failed;
                Err(PlanningSessionError::Search(error))
            }
            Ok(PlanningSearchOutcome::Cancelled) => {
                self.state = PlanningSessionState::Cancelled;
                Ok(self.state)
            }
            Ok(PlanningSearchOutcome::Infeasible { diagnostics }) => {
                self.diagnostics = diagnostics;
                self.state = PlanningSessionState::Infeasible;
                Ok(self.state)
            }
            Ok(PlanningSearchOutcome::ProvenMaximum(result)) => {
                self.maximum = Some(result.maximum_base_hundredths);
                self.state = PlanningSessionState::Collecting;
                if control.is_cancelled(PlanningSearchPhase::MaximumProved) {
                    self.state = PlanningSessionState::Cancelled;
                }
                Ok(self.state)
            }
        }
    }

    /// Returns only new deliveries. Resume a paused session with another call.
    /// Cancellation is terminal; all previously delivered programs remain available.
    pub fn collect(
        &mut self,
        batch_size: NonZeroUsize,
        control: &impl PlanningCollectionControl,
    ) -> Result<PlanningSearchBatch, PlanningSessionError> {
        if !matches!(
            self.state,
            PlanningSessionState::Collecting | PlanningSessionState::Paused
        ) {
            return Err(PlanningSessionError::InvalidState { state: self.state });
        }
        self.state = PlanningSessionState::Collecting;
        let polling = CollectionPolling {
            control,
            action: Cell::new(PlanningCollectionAction::Continue),
        };
        let mut alternatives = Vec::new();
        loop {
            if polling.is_cancelled(PlanningSearchPhase::Alternatives) {
                break;
            }
            let prepared = self
                .prepared
                .as_mut()
                .expect("proved session retains its model");
            let profile = self
                .registry
                .resolve(&self.request.profile, self.request.scoring_table)
                .expect("immutable registry resolved during proof");
            if prepared.pending.is_none() {
                prepared.pending = match prepared.next(
                    profile,
                    &self.request,
                    self.maximum.expect("proved maximum"),
                    &polling,
                ) {
                    Ok(next) => next,
                    Err(error) => {
                        self.state = PlanningSessionState::Failed;
                        return Err(PlanningSessionError::Search(error));
                    }
                };
                if prepared.pending.is_none() {
                    if polling.action.get() == PlanningCollectionAction::Continue {
                        self.state = PlanningSessionState::Exhausted;
                    }
                    break;
                }
            }
            let vector = prepared
                .pending
                .as_ref()
                .expect("candidate copied before reconstruction");
            let result = model::reconstruct(
                self.registry,
                &self.request,
                profile,
                &prepared.atoms,
                vector,
                self.maximum.expect("proved maximum"),
                &polling,
            )
            .and_then(|candidate| {
                candidate
                    .map(|mut candidate| {
                        finish_candidate(
                            self.registry,
                            &self.request,
                            profile,
                            self.maximum_step_level,
                            self.maximum.expect("proved maximum"),
                            &mut candidate,
                        )?;
                        Ok(candidate)
                    })
                    .transpose()
            });
            match result {
                Err(error) => {
                    self.state = PlanningSessionState::Failed;
                    return Err(PlanningSessionError::Search(error));
                }
                Ok(None) => break,
                Ok(Some(candidate)) => {
                    if polling.is_cancelled(PlanningSearchPhase::Reconstruction) {
                        break;
                    }
                    // No control poll may split this delivery transaction.
                    prepared
                        .delivered
                        .push(prepared.pending.take().expect("pending candidate"));
                    self.confirmed.push(candidate.clone());
                    alternatives.push(candidate);
                }
            }
            if alternatives.len() == batch_size.get() {
                break;
            }
        }
        match polling.action.get() {
            PlanningCollectionAction::Continue => {}
            PlanningCollectionAction::Pause => self.state = PlanningSessionState::Paused,
            PlanningCollectionAction::Cancel => self.state = PlanningSessionState::Cancelled,
        }
        Ok(PlanningSearchBatch {
            alternatives,
            state: self.state,
        })
    }
}

struct CollectionPolling<'a, C> {
    control: &'a C,
    action: Cell<PlanningCollectionAction>,
}
impl<C: PlanningCollectionControl> PlanningSearchControl for CollectionPolling<'_, C> {
    fn is_cancelled(&self, phase: PlanningSearchPhase) -> bool {
        if self.action.get() == PlanningCollectionAction::Continue {
            self.action.set(self.control.action(phase));
        }
        self.action.get() != PlanningCollectionAction::Continue
    }
}
