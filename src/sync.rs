use std::fmt::Display;

use dbus::blocking::Connection;
use dbus::MethodErr;
use dbus_crossroads::{Crossroads, IfaceToken};

use crate::{Action, Config, Match};

/// A synchronous runner.
#[doc = concat!("\n\n", include_str!("./docs/runner/runner.md"), "\n\n")]
/// If you need to run asynchronous code in your runner, consider enabling the
/// `tokio` feature and using an [`AsyncRunner`](crate::AsyncRunner).
pub trait Runner {
	#[doc = include_str!("./docs/runner/action.md")]
	type Action: Action;
	#[doc = include_str!("./docs/runner/err.md")]
	type Err: Display;

	#[doc = concat!(include_str!("./docs/runner/matches.md"), "\n\n")]
	/// # Example
	///
	/// ```ignore
	/// struct Runner {
	///     known_words: Vec<String>,
	/// }
	///
	/// impl krunner::Runner for Runner {
	///     // ...
	///
	///     fn matches(
	///         &mut self,
	///         query: String
	///     ) -> Result<Vec<Match<Self::Action>>, Self::Err> {
	///         let matches = if self.known_words.contains(&query) {
	///             vec![Match {
	///                 id: query.clone(),
	///                 title: format!("Matched word: {query}"),
	///                 ty: MatchType::ExactMatch,
	///                 relevance: 1.0,
	///
	///                 ..Match::default()
	///             }]
	///         } else {
	///             vec![]
	///         };
	///         Ok(matches)
	///     }
	///
	///     // ...
	/// }
	/// ```
	fn matches(&mut self, query: String) -> Result<Vec<Match<Self::Action>>, Self::Err>;

	#[doc = concat!(include_str!("./docs/runner/run.md"), "\n\n")]
	/// # Example
	///
	/// ```ignore
	/// struct Runner {
	///     known_words: Vec<String>,
	/// }
	///
	/// impl krunner::Runner for Runner {
	///     // ...
	///
	///     fn run(
	///         &mut self,
	///         match_id: String,
	///         action: Option<Self::Action>,
	///     ) -> Result<(), Self::Err> {
	///         match action {
	///             Some(Action::LaunchDictionary) => {
	///                 // Launch dictionary via xdg-open
	///                 std::process::Command::new("xdg-open")
	///                     .arg(&format!("https://en.wiktionary.org/wiki/{match_id}"))
	///                     .spawn()
	///                     .unwrap();
	///             }
	///             None => {
	///                 // If the user didn't choose any specific action, do nothing
	///             }
	///         }
	///     }
	///
	///     // ...
	/// }
	/// ```
	fn run(&mut self, match_id: String, action: Option<Self::Action>) -> Result<(), Self::Err>;

	#[doc = include_str!("./docs/runner/config.md")]
	fn config(&mut self) -> Result<Config<Self::Action>, Self::Err> {
		Ok(Config::default())
	}

	#[doc = include_str!("./docs/runner/teardown.md")]
	fn teardown(&mut self) -> Result<(), Self::Err> {
		Ok(())
	}
}

/// Helper methods for [`Runner`]s.
pub trait RunnerExt: Runner + Sized + Send + 'static {
	/// Starts running this runner on the main thread indefinitely.
	///
	/// This is a convenience function that starts a new D-Bus connection,
	/// requests the given service name, [registers the KRunner
	/// interface](Self::register), and starts indefinitely listening on the
	/// session bus.
	///
	/// # Example
	/// ```ignore
	/// use krunner::RunnerExt;
	///
	/// struct Runner;
	///
	/// impl krunner::Runner for Runner {
	/// 	// ...
	/// }
	///
	/// fn main() -> Result<(), Box<dyn std::error::Error>> {
	/// 	Runner.start("some.runner.path", "/SomeRunner")?;
	/// 	Ok(())
	/// }
	/// ```
	fn start(self, service: &'static str, path: &'static str) -> Result<(), dbus::Error>;

	#[doc = include_str!("./docs/runnerext/register.md")]
	fn register(cr: &mut Crossroads) -> IfaceToken<Self>;
}

impl<R: Runner + Sized + Send + 'static> RunnerExt for R {
	fn start(self, service: &'static str, path: &'static str) -> Result<(), dbus::Error> {
		let c = Connection::new_session()?;
		c.request_name(service, false, true, false)?;

		let mut cr = Crossroads::new();

		let token = Self::register(&mut cr);
		cr.insert(path, &[token], self);
		cr.serve(&c)
	}

	fn register(cr: &mut Crossroads) -> IfaceToken<Self> {
		cr.register("org.kde.krunner1", |b| {
			b.method("Actions", (), ("matches",), |_, _: &mut Self, _: ()| {
				let actions: Vec<_> = Self::Action::all()
					.iter()
					.map(crate::action_as_arg)
					.collect();
				Ok((actions,))
			});
			b.method(
				"Run",
				("matchId", "actionId"),
				(),
				|_, runner, (match_id, action_id): (String, String)| {
					let action = if let Some(action) = Self::Action::from_id(&action_id) {
						Some(action)
					} else if action_id.is_empty() {
						None
					} else {
						return Err(MethodErr::invalid_arg("Unknown action"));
					};
					runner
						.run(match_id, action)
						.map_err(|e| MethodErr::failed(&e))
				},
			);
			b.method(
				"Match",
				("query",),
				("matches",),
				|_, runner, (query,): (String,)| match runner.matches(query) {
					Ok(v) => Ok((v,)),
					Err(e) => Err(MethodErr::failed(&e)),
				},
			);
			b.method("Config", (), ("config",), |_, runner, _: ()| {
				match runner.config() {
					Ok(c) => Ok((c,)),
					Err(e) => Err(MethodErr::failed(&e)),
				}
			});
			b.method("Teardown", (), (), |_, runner, _: ()| {
				runner.teardown().map_err(|e| MethodErr::failed(&e))
			});
		})
	}
}
