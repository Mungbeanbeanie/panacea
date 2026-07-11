//! MicroVM / Wasm isolation: setup, teardown, and the isolation guarantee that alleles and
//! genes execute only here, never against the live host
//! (see ../../.claude/skills/sandbox-isolation-check).

use crate::core::Pid;
use crate::evolution::alleles::Allele;

/// Snapshot of a frozen process's memory space, as a Soldier clones it into the sandbox
/// after Stage 1 suspends the PID. Phase 4 mocks this input directly since the real
/// snapshot comes from the Soldier lifecycle (Phase 3, not yet landed) — see
/// ../../.claude/docs/plan.md.
#[derive(Debug, Clone)]
pub struct FrozenProcess {
    pub pid: Pid,
    pub memory: Vec<u8>,
}

/// Result of running an allele combination against the cloned target inside the sandbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrialOutcome {
    /// Target kept running; this combo didn't do enough.
    TargetSurvived,
    /// Target aborted/crashed and the mock host stayed intact — a fuzz success.
    TargetCrashed,
    /// The mock host itself was destabilized; reject this combo regardless of target state.
    HostDestabilized,
}

/// A MicroVM/Wasm sandbox holding one cloned target, isolated from the live host.
///
/// [`Sandbox::run`] is the *only* way to execute an allele combination — it takes
/// `&mut self`, so there is no code path that runs an allele or gene without going through
/// a live sandbox handle.
pub struct Sandbox {
    target: FrozenProcess,
}

impl Sandbox {
    /// Sets up the sandbox: takes the cloned target into the MicroVM/Wasm container
    /// against a mock host OS.
    pub fn spawn(target: FrozenProcess) -> Self {
        Self { target }
    }

    pub fn target_pid(&self) -> Pid {
        self.target.pid
    }

    /// Runs one allele combination against the cloned target and reports the outcome.
    ///
    /// Mock host physics: `Allele04` + `Allele12` together reliably crash the target
    /// without destabilizing the host (the Source of Truth's worked example);
    /// `Allele09` alone is too aggressive and destabilizes the mock host, so it's rejected
    /// even if paired with something that would otherwise kill the target.
    pub fn run(&mut self, combo: &[Allele]) -> TrialOutcome {
        if combo.contains(&Allele::Allele09) {
            return TrialOutcome::HostDestabilized;
        }
        if combo.contains(&Allele::Allele04) && combo.contains(&Allele::Allele12) {
            return TrialOutcome::TargetCrashed;
        }
        TrialOutcome::TargetSurvived
    }

    /// Tears down the sandbox: consumes and drops the cloned target and container.
    pub fn teardown(self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_target() -> FrozenProcess {
        FrozenProcess {
            pid: 4242,
            memory: vec![0xAA; 16],
        }
    }

    #[test]
    fn winning_pair_crashes_target_without_destabilizing_host() {
        let mut sandbox = Sandbox::spawn(mock_target());
        let outcome = sandbox.run(&[Allele::Allele04, Allele::Allele12]);
        assert_eq!(outcome, TrialOutcome::TargetCrashed);
    }

    #[test]
    fn lone_allele_is_not_enough() {
        let mut sandbox = Sandbox::spawn(mock_target());
        assert_eq!(
            sandbox.run(&[Allele::Allele04]),
            TrialOutcome::TargetSurvived
        );
    }

    #[test]
    fn unsafe_allele_destabilizes_host_even_if_combined() {
        let mut sandbox = Sandbox::spawn(mock_target());
        let outcome = sandbox.run(&[Allele::Allele04, Allele::Allele09]);
        assert_eq!(outcome, TrialOutcome::HostDestabilized);
    }

    #[test]
    fn run_requires_a_live_sandbox_handle() {
        // Isolation guarantee (see sandbox-isolation-check skill): `Sandbox::run` is the
        // only execution entry point and it takes `&mut Sandbox` — there is no free
        // function that runs an allele combo without one.
        fn assert_takes_sandbox(_: fn(&mut Sandbox, &[Allele]) -> TrialOutcome) {}
        assert_takes_sandbox(Sandbox::run);
    }
}
