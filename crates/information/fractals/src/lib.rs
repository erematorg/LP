pub mod data_loader;
pub mod fractals;
pub mod generator;
pub mod grammar;
pub mod interpreter;
pub mod renderer;

pub mod prelude {
    // Core fractal functionality
    pub use crate::fractals::{LSystem, RuleManager};
    pub use crate::generator::generate;
    pub use crate::grammar::apply_rules;
    
    // Interpreter and renderer
    pub use crate::interpreter::{InterpreterOutput, SymbolType, interpret};
    pub use crate::renderer::run_renderer;
    
    // Data loading
    pub use crate::data_loader::{Template, Parameters, load_template};
}