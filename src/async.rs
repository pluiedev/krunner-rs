use std::fmt::Display;
use std::sync::Arc;

use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;
use dbus::MethodErr;
use dbus_crossroads::{Crossroads, IfaceToken};
use tokio::sync::Mutex;

use crate::{Action, Config, Match};

#[cfg_attr(not(docs_rs), async_trait::async_trait)]
/// An asynchronous runner.
#[doc = concat!("\n\n", include_str!("./docs/runner/runner.md"), "\n\n")]
/// Check out [`Runner`](crate::Runner) for a fully blocking, synchronous
/// equivalent.
pub trait AsyncRunner {
	#[doc = include_str!("./docs/runner/action.md")]
	type Action: Action;
	#[doc = include_str!("./docs/runner/err.md")]
	type Err: Display;

	#[doc = include_str!("./docs/runner/matches.md")]
	async fn matches(&mut self, query: String) -> Result<Vec<Match<Self::Action>>, Self::Err>;

	#[doc = include_str!("./docs/runner/run.md")]
	async fn run(
		&mut self,
		match_id: String,
		action: Option<Self::Action>,
	) -> Result<(), Self::Err>;

	#[doc = include_str!("./docs/runner/config.md")]
	async fn config(&mut self) -> Result<Option<Config<Self::Action>>, Self::Err> {
		Ok(None)
	}

	#[doc = include_str!("./docs/runner/teardown.md")]
	async fn teardown(&mut self) -> Result<(), Self::Err> {
		Ok(())
	}
}

#[cfg_attr(not(docs_rs), async_trait::async_trait)]
pub trait AsyncRunnerExt: AsyncRunner + Sized + Send + 'static {
	/// Runs a [runner](Runner) on the main thread forever.
	///
	/// This is a convenience function that starts a new D-Bus connection,
	/// requests the given service name, [registers the KRunner
	/// interface](Self::register), and starts indefinitely listening on the
	/// session bus.
	async fn start(self, service: &'static str, path: &'static str) -> Result<(), dbus::Error>
	where
		Self::Action: Send;

	fn register(cr: &mut Crossroads) -> IfaceToken<Arc<Mutex<Self>>>
	where
		Self::Action: Send;
}
#[cfg_attr(not(docs_rs), async_trait::async_trait)]
impl<R: AsyncRunner + Sized + Send + 'static> AsyncRunnerExt for R {
	async fn start(self, service: &'static str, path: &'static str) -> Result<(), dbus::Error>
	where
		Self::Action: Send,
	{
		let (res, c) = dbus_tokio::connection::new_session_sync()?;

		let _handle = tokio::spawn(async {
			let err = res.await;
			panic!("Lost connection to D-Bus: {err}");
		});

		c.request_name(service, false, true, false).await?;

		let mut cr = Crossroads::new();
		cr.set_async_support(Some((
			c.clone(),
			Box::new(|x| {
				tokio::spawn(x);
			}),
		)));

		let token = Self::register(&mut cr);
		cr.insert(path, &[token], Arc::new(Mutex::new(self)));

		// equiv to `serve`
		c.start_receive(
			MatchRule::new_method_call(),
			Box::new(move |msg, conn| {
				cr.handle_message(msg, conn).unwrap();
				true
			}),
		);
		std::future::pending::<()>().await;
		unreachable!()
	}

	fn register(cr: &mut Crossroads) -> IfaceToken<Arc<Mutex<Self>>>
	where
		Self::Action: Send,
	{
		cr.register("org.kde.krunner1", |b| {
			b.method(
				"Actions",
				(),
				("matches",),
				|_, _: &mut Arc<Mutex<Self>>, _: ()| {
					let actions: Vec<_> =
						R::Action::all().iter().map(crate::action_as_arg).collect();

					Ok((actions,))
				},
			);
			b.method_with_cr_async(
				"Run",
				("matchId", "actionId"),
				(),
				|mut ctx, cr, (match_id, action_id): (String, String)| {
					let runner: Arc<Mutex<Self>> = Arc::clone(cr.data_mut(ctx.path()).unwrap());

					async move {
						ctx.reply('r: {
							let mut lock = runner.lock().await;

							let action = if let Some(action) = R::Action::from_id(&action_id) {
								Some(action)
							} else if action_id.is_empty() {
								None
							} else {
								break 'r Err(MethodErr::invalid_arg("unknown action"));
							};
							lock.run(match_id, action)
								.await
								.map_err(|e| MethodErr::failed(&e))
						})
					}
				},
			);
			b.method_with_cr_async(
				"Match",
				("query",),
				("matches",),
				|mut ctx, cr, (query,): (String,)| {
					let runner: Arc<Mutex<Self>> = Arc::clone(cr.data_mut(ctx.path()).unwrap());

					async move {
						ctx.reply({
							let mut lock = runner.lock().await;

							lock.matches(query)
								.await
								.map(|v| (v,))
								.map_err(|e| MethodErr::failed(&e))
						})
					}
				},
			);
			b.method_with_cr_async("Config", (), ("config",), |mut ctx, cr, _: ()| {
				let runner: Arc<Mutex<Self>> = Arc::clone(cr.data_mut(ctx.path()).unwrap());

				async move {
					ctx.reply({
						let mut lock = runner.lock().await;

						match lock.config().await {
							Ok(Some(v)) => Ok((v,)),
							Ok(None) => Err(MethodErr::no_method("config")),
							Err(e) => Err(MethodErr::failed(&e)),
						}
					})
				}
			});
			b.method_with_cr_async("Teardown", (), (), |mut ctx, cr, _: ()| {
				let runner: Arc<Mutex<Self>> = Arc::clone(cr.data_mut(ctx.path()).unwrap());

				async move {
					ctx.reply({
						let mut lock = runner.lock().await;

						lock.teardown().await.map_err(|e| MethodErr::failed(&e))
					})
				}
			});
		})
	}
}
