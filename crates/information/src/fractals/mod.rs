pub mod data_loader;
pub mod fractals;
pub mod generator;
pub mod grammar;
pub mod interpreter;
pub mod renderer;

pub mod prelude {
    // Core fractal functionality
    pub use super::fractals::{LSystem, RuleManager};
    pub use super::generator::generate;
    pub use super::grammar::apply_rules;
    
    // Interpreter and renderer
    pub use super::interpreter::{InterpreterOutput, SymbolType, interpret};
    pub use super::renderer::run_renderer;
    
    // Data loading
    pub use super::data_loader::{Template, Parameters, load_template};
}