//! Optimizer passes for PromptIR
//!
//! Applies transformations and optimizations to the prompt IR.

use rapidagent_ir::PromptIR;
use tracing::debug;

/// Optimize the PromptIR by applying transformation passes
pub fn optimize(ir: &mut PromptIR) {
    debug!("Optimizing PromptIR");
    
    // Sections are already sorted by priority in add_section
    // Ensure final sort
    ir.sections.sort_by(|a, b| b.priority.weight().cmp(&a.priority.weight()));
    
    debug!("Optimization complete: {} sections", ir.sections.len());
}