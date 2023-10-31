pub mod book;
pub use book::*;
pub mod entry;
pub use entry::*;
pub mod handlers;
pub use error::*;
pub mod error;

const DATE_DISPLAY_FORMATTING: &str = "%Y-%m-%d %H:%M:%S";
