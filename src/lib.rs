pub use crate::types::*;

#[path = "async.rs"]
mod _async;
mod sync;
pub mod types;

pub use _async::{run_async, AsyncRunner};
pub use sync::{run, Runner};

pub trait Action: Sized {
	fn all() -> Vec<Self>;

	fn from_id(s: &str) -> Option<Self>;
	fn to_id(&self) -> String;
	fn info(&self) -> ActionInfo;
}
