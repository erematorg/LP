pub mod types;

/// Prelude for the memory module.
///
/// This includes types for storing and managing entity memories.
pub mod prelude {
    pub use crate::memory::types::{MemoryEvent, MemoryEventType, MemoryTimestamp};
}