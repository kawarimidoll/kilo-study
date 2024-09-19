mod size;
pub use size::Size;
mod position;
pub use position::Position;
mod location;
pub use location::Location;

pub type GraphemeIdx = usize;
pub type ByteIdx = usize;
pub type LineIdx = usize;
pub type ColIdx = usize;
pub type RowIdx = usize;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
