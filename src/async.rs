use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;
use dbus::MethodErr;
use dbus_crossroads::{Context, Crossroads, IfaceToken};
use tokio::sync::Mutex;

use crate::{Action, ActionInfo, Config, Match};

#[async_trait]
pub trait AsyncRunner {
	type Action: Action;
	type Err: Into<MethodErr>;

	async fn matches(
		&mut self,
		ctx: &mut Context,
		query: String,
	) -> Result<Vec<Match<Self::Action>>, Self::Err>;

	async fn run(
		&mut self,
		ctx: &mut Context,
		match_id: String,
		action: Option<Self::Action>,
	) -> Result<(), MethodErr>;

	async fn config(&mut self, _ctx: &mut Context) -> Result<Config<Self::Action>, Self::Err> {
		todo!()
	}

	async fn teardown(&mut self, _ctx: &mut Context) -> Result<(), Self::Err> {
		Ok(())
	}
}

pub async fn run_async<R>(
	runner: R,
	service: &'static str,
	path: &'static str,
) -> Result<(), dbus::Error>
where
	R: AsyncRunner + Send + Sync + 'static,
	R::Action: Send,
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

	let token = register::<R>(&mut cr);
	cr.insert(path, &[token], Arc::new(Mutex::new(runner)));

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

fn register<R>(cr: &mut Crossroads) -> IfaceToken<Arc<Mutex<R>>>
where
	R: AsyncRunner + Send + Sync + 'static,
	R::Action: Send,
{
	cr.register("org.kde.krunner1", |b| {
		b.method(
			"Actions",
			(),
			("matches",),
			|_, _: &mut Arc<Mutex<R>>, _: ()| {
				let actions: Vec<_> = R::Action::all()
					.into_iter()
					.map(|a| {
						let ActionInfo { text, icon_source } = a.info();
						(a.to_id(), text, icon_source)
					})
					.collect();

				Ok((actions,))
			},
		);
		b.method_with_cr_async(
			"Run",
			("matchId", "actionId"),
			(),
			|mut ctx, cr, (match_id, action_id): (String, String)| {
				let runner: Arc<Mutex<R>> = Arc::clone(cr.data_mut(ctx.path()).unwrap());

				async move {
					let r = {
						let mut lock = runner.lock().await;

						if let Some(action) = R::Action::from_id(&action_id) {
							lock.run(&mut ctx, match_id, Some(action)).await
						} else if action_id.is_empty() {
							lock.run(&mut ctx, match_id, None).await
						} else {
							Err(MethodErr::invalid_arg("unknown action"))
						}
					};
					ctx.reply(r)
				}
			},
		);
		b.method_with_cr_async(
			"Match",
			("query",),
			("matches",),
			|mut ctx, cr, (query,): (String,)| {
				let runner: Arc<Mutex<R>> = Arc::clone(cr.data_mut(ctx.path()).unwrap());

				async move {
					let r = {
						let mut lock = runner.lock().await;

						lock.matches(&mut ctx, query)
							.await
							.map(|v| (v,))
							.map_err(Into::into)
					};
					ctx.reply(r)
				}
			},
		);
		b.method_with_cr_async("Config", (), ("config",), |mut ctx, cr, _: ()| {
			let runner: Arc<Mutex<R>> = Arc::clone(cr.data_mut(ctx.path()).unwrap());

			async move {
				let r = {
					let mut lock = runner.lock().await;

					lock.config(&mut ctx)
						.await
						.map(|v| (v,))
						.map_err(Into::into)
				};
				ctx.reply(r)
			}
		});
		b.method_with_cr_async("Teardown", (), (), |mut ctx, cr, _: ()| {
			let runner: Arc<Mutex<R>> = Arc::clone(cr.data_mut(ctx.path()).unwrap());

			async move {
				let r = {
					let mut lock = runner.lock().await;

					lock.teardown(&mut ctx).await.map_err(Into::into)
				};
				ctx.reply(r)
			}
		});
	})
}
