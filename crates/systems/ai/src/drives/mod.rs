pub mod needs;

/// Prelude for the drives module.
///
/// This includes core need types and drive components.
pub mod prelude {
    pub use crate::drives::needs::{get_most_urgent_need, update_needs, Need, NeedType};
}
