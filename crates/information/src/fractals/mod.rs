//IMPORTANT DISCLAIMER: The fractals appraoch along JSON based and current predetermined/hardcoded appraoch will get override by PBMPM principles.
// This is a temporary solution until we have a more dynamic and flexible system in place.

pub mod core;
pub mod data_loader;
pub mod generator;
pub mod grammar;
pub mod interpreter;
pub mod renderer;

pub mod prelude {
    // Core fractal functionality
    pub use super::core::{LSystem, RuleManager};
    pub use super::generator::generate;
    pub use super::grammar::apply_rules;

    // Interpreter and renderer
    pub use super::interpreter::{interpret, InterpreterOutput, SymbolType};
    pub use super::renderer::run_renderer;

    // Data loading
    pub use super::data_loader::{load_template, Parameters, Template};
}
