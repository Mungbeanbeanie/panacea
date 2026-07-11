//! Stage 3 — Lymph Node Regression Matrix (Allergy Check).
//!
//! Before a candidate Wasm gene reaches the ledger (Phase 8), it's replayed against a
//! crowded validator environment standing in for the Source of Truth's "standard OS base +
//! Top-5,000 apps": if a whitelisted app would crash or leak, the gene is autoimmune and
//! gets dropped instead of published. Decoupled from Stage 2 — it takes any [`GenePayload`]
//! candidate, not just one that came from [`crate::evolution::alleles::fuzz`], so Phase 7
//! doesn't block on how the gene was produced.

use crate::evolution::alleles::{Allele, GenePayload};

/// One whitelisted app in the crowded regression environment — a mock subset standing in
/// for the Top-5,000 most common apps (Chrome, VS Code, Slack, ...).
#[derive(Debug, Clone, Copy)]
pub struct WhitelistedApp {
    pub name: &'static str,
    /// `Some(allele)` if this app crashes/leaks whenever a candidate gene's sequence
    /// contains that allele; `None` tolerates every allele in the catalog. Mock host
    /// physics: only `Allele09` (already flagged in `sandbox.rs` as "too aggressive") has
    /// an app sensitive to it here.
    allergic_to: Option<Allele>,
}

/// Mock subset of the Top-5,000 regression matrix.
pub const TOP_APPS: [WhitelistedApp; 4] = [
    WhitelistedApp {
        name: "Chrome",
        allergic_to: None,
    },
    WhitelistedApp {
        name: "VS Code",
        allergic_to: None,
    },
    WhitelistedApp {
        name: "Slack",
        allergic_to: None,
    },
    WhitelistedApp {
        name: "LegacyBackupAgent",
        allergic_to: Some(Allele::Allele09),
    },
];

/// Result of replaying a candidate gene against the Lymph Node's crowded environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegressionOutcome {
    /// No whitelisted app reacted; the gene may proceed to Stage 4.
    Passed,
    /// The named whitelisted app crashed/leaked — the gene is autoimmune and must be dropped.
    AllergyFlagged(&'static str),
}

/// A Lymph Node validator: the standard OS base + Top-5,000 apps a candidate gene is
/// replayed against before it can reach the ledger.
pub struct LymphNode {
    apps: &'static [WhitelistedApp],
}

impl LymphNode {
    /// The standard validator environment (full mock Top-5,000 subset).
    pub fn standard_environment() -> Self {
        Self { apps: &TOP_APPS }
    }

    /// Executes the candidate gene's allele sequence in the crowded environment and reports
    /// whether any whitelisted app reacted. Checks every app rather than stopping at the
    /// first pass, so any single allergic reaction anywhere in the matrix raises the flag.
    pub fn run_regression(&self, gene: &GenePayload) -> RegressionOutcome {
        for app in self.apps {
            if let Some(allele) = app.allergic_to {
                if gene.sequence.contains(&allele) {
                    return RegressionOutcome::AllergyFlagged(app.name);
                }
            }
        }
        RegressionOutcome::Passed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gene_that_breaks_a_whitelisted_app_is_flagged_and_dropped() {
        let lymph_node = LymphNode::standard_environment();
        let gene = GenePayload {
            sequence: vec![Allele::Allele09],
        };

        let outcome = lymph_node.run_regression(&gene);

        assert_eq!(
            outcome,
            RegressionOutcome::AllergyFlagged("LegacyBackupAgent")
        );
    }

    #[test]
    fn gene_the_whitelisted_apps_tolerate_passes() {
        let lymph_node = LymphNode::standard_environment();
        // The Stage-2 fuzz winner (Allele04 + Allele12) touches no allergic allele.
        let gene = GenePayload {
            sequence: vec![Allele::Allele04, Allele::Allele12],
        };

        assert_eq!(lymph_node.run_regression(&gene), RegressionOutcome::Passed);
    }
}
