//! MicroVM / Wasm isolation: setup, teardown, and the isolation guarantee that alleles and
//! genes execute only here, never against the live host
//! (see ../../.claude/skills/sandbox-isolation-check).

use crate::core::Pid;
use crate::evolution::alleles::Allele;
use wasmi::{Engine, Linker, Module, Store};

/// The sandbox's fixed decision logic, compiled as real Wasm bytecode instead of a Rust
/// `match`. Bit `n` of the input corresponds to `Allele` discriminant `n` being present in
/// the combo under test. Mirrors [`Sandbox::run`]'s mock-host physics exactly: `Allele09`
/// (bit 2) alone destabilizes the host; `Allele04` + `Allele12` together (bits 0 and 1)
/// crash the target; anything else, the target survives.
///
/// Takes zero imports — this module cannot call back into the host, touch host memory, or
/// reach the network/filesystem no matter what bytecode runs inside it.
const EVALUATE_WAT: &str = r#"
(module
  (func $evaluate (param $bitmask i32) (result i32)
    (if (i32.ne (i32.and (local.get $bitmask) (i32.const 4)) (i32.const 0))
      (then (return (i32.const 2))))
    (if (i32.eq (i32.and (local.get $bitmask) (i32.const 3)) (i32.const 3))
      (then (return (i32.const 1))))
    (i32.const 0)
  )
  (export "evaluate" (func $evaluate))
)
"#;

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
/// a live sandbox handle. Execution happens inside a real `wasmi` Wasm runtime: the
/// isolation boundary is genuine (the compiled module has zero host imports, so it is
/// provably incapable of touching the real host), while the attack-effectiveness decision
/// stays the same synthetic physics as before, now expressed as compiled bytecode.
pub struct Sandbox {
    target: FrozenProcess,
    engine: Engine,
    /// Compiled once per `Sandbox` and reused across every `run()` call — the WAT source
    /// never changes, so re-parsing it on every trial (`fuzz()` tries up to 7 combos) would
    /// be wasted work. Only the execution state below is fresh per run.
    module: Module,
}

impl Sandbox {
    /// Sets up the sandbox: takes the cloned target into the MicroVM/Wasm container
    /// against a mock host OS.
    pub fn spawn(target: FrozenProcess) -> Self {
        let engine = Engine::default();
        let module = Module::new(&engine, EVALUATE_WAT)
            .expect("EVALUATE_WAT is a fixed, hand-written constant validated at compile time");
        Self {
            target,
            engine,
            module,
        }
    }

    pub fn target_pid(&self) -> Pid {
        self.target.pid
    }

    /// Runs one allele combination against the cloned target and reports the outcome.
    ///
    /// Encodes `combo` into a bitmask and calls the sandboxed Wasm module's `evaluate`
    /// export against a fresh `Store`/`Instance` — a new, isolated execution context every
    /// call, so no state (host or otherwise) carries over between trials. Mock host
    /// physics: `Allele04` + `Allele12` together reliably crash the target without
    /// destabilizing the host (the Source of Truth's worked example); `Allele09` alone is
    /// too aggressive and destabilizes the mock host, so it's rejected even if paired with
    /// something that would otherwise kill the target.
    pub fn run(&mut self, combo: &[Allele]) -> TrialOutcome {
        let bitmask: i32 = combo
            .iter()
            .fold(0i32, |mask, allele| mask | (1 << *allele as u8));

        let mut store = Store::new(&self.engine, ());
        let linker = Linker::<()>::new(&self.engine);
        let instance = linker
            .instantiate_and_start(&mut store, &self.module)
            .expect("EVALUATE_WAT declares zero imports, so instantiation cannot fail on a missing import");
        let evaluate = instance
            .get_typed_func::<i32, i32>(&store, "evaluate")
            .expect("EVALUATE_WAT exports exactly one function: evaluate(i32) -> i32");
        let result = evaluate
            .call(&mut store, bitmask)
            .expect("evaluate has no unreachable/OOB/host-call paths, so it cannot trap");

        match result {
            2 => TrialOutcome::HostDestabilized,
            1 => TrialOutcome::TargetCrashed,
            _ => TrialOutcome::TargetSurvived,
        }
    }

    /// Tears down the sandbox: consumes and drops the cloned target and container. Normal
    /// `Drop` semantics release the `wasmi` engine/module with it — no explicit cleanup
    /// needed.
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

    #[test]
    fn compiled_gene_module_has_zero_imports() {
        // Concrete, automatable proof of the isolation claim: the sandboxed module has no
        // ambient host capability at all, regardless of what bytecode runs inside it.
        let engine = Engine::default();
        let module = Module::new(&engine, EVALUATE_WAT).expect("EVALUATE_WAT is valid WAT");
        assert_eq!(
            module.imports().count(),
            0,
            "the gene-evaluation module must not import any host function"
        );
    }
}
