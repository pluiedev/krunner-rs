pub use crate::types::*;

#[path = "async.rs"]
#[cfg(feature = "tokio")]
mod _async;
mod sync;
pub mod types;

#[cfg(feature = "tokio")]
pub use _async::{run_async, AsyncRunner};
pub use dbus::MethodErr;
pub use dbus_crossroads::Context;
pub use sync::{run, Runner};

pub trait Action: Sized {
	fn all() -> Vec<Self>;

	fn from_id(s: &str) -> Option<Self>;
	fn to_id(&self) -> String;
	fn info(&self) -> ActionInfo;
}
