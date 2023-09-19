use std::future::Future;

use async_trait::async_trait;
use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;
use dbus::{arg, strings, MethodErr};
use dbus_crossroads::{Context, Crossroads, IfaceBuilder, IfaceToken, MethodDesc};

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
		action: Self::Action,
	) -> Result<(), MethodErr>;

	async fn config(&mut self, ctx: &mut Context) -> Result<Config<Self::Action>, Self::Err> {
		todo!()
	}

	async fn teardown(&mut self, ctx: &mut Context) -> Result<(), Self::Err> {
		Ok(())
	}
}

pub async fn run_async<R>(
	runner: R,
	service: &'static str,
	path: &'static str,
) -> Result<(), dbus::Error>
where
	R: AsyncRunner + Send + 'static,
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
	cr.insert(path, &[token], runner);

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

fn register<R>(cr: &mut Crossroads) -> IfaceToken<R>
where
	R: AsyncRunner + Send + 'static,
	R::Action: Send,
{
	cr.register("org.kde.krunner1", |b| {
		b.method("Actions", (), ("matches",), |ctx, runner: &mut R, _: ()| {
			let actions: Vec<_> = R::Action::all()
				.into_iter()
				.map(|a| {
					let ActionInfo { text, icon_source } = a.info();
					(a.to_id(), text, icon_source)
				})
				.collect();

			Ok((actions,))
		});
		b.method_with_cr_async(
			"Run",
			("matchId", "actionId"),
			(),
			|mut ctx, cr, (match_id, action_id): (String, String)| async move {
				let Some(runner): Option<&mut R> = cr.data_mut(ctx.path()) else {
					return ctx.reply(Err(MethodErr::no_path(ctx.path())));
				};
				let Some(action) = R::Action::from_id(&action_id) else {
					return ctx.reply(Err(MethodErr::invalid_arg("Unknown action")));
				};
				let _ = runner.run(&mut ctx, match_id, action).await;
				ctx.reply(Ok(()))
			},
		);
		// b.method_async(
		// 	"Run",
		// 	("matchId", "actionId"),
		// 	(),
		// 	|ctx, runner: &mut R, (match_id, action_id): (String, String)| async {
		// 		let Some(action) = R::Action::from_id(&action_id) else {
		// 			return Err(MethodErr::invalid_arg("Unknown action"));
		// 		};
		// 		runner.run(ctx, match_id, action).await?;
		// 		Ok(())
		// 	},
		// );
		// b.method_async(
		// 	"Match",
		// 	("query",),
		// 	("matches",),
		// 	|ctx, runner: &mut R, (query,): (String,)| async {
		// 		runner
		// 			.matches(ctx, query)
		// 			.await
		// 			.map(|v| (v,))
		// 			.map_err(Into::into)
		// 	},
		// );
		// b.method_async(
		// 	"Config",
		// 	(),
		// 	("config",),
		// 	|ctx, runner: &mut R, _: ()| async {
		// 		runner.config(ctx).await.map(|v| (v,)).map_err(Into::into)
		// 	},
		// );
		// b.method_async("Teardown", (), (), |ctx, runner: &mut R, _: ()| async
		// { 	runner.teardown(ctx).await.map_err(Into::into)
		// });
	})
}

// trait IfaceBuilderExt<T> {
// 	fn method_async<IA, OA, N, R, CB>(
// 		&mut self,
// 		name: N,
// 		input_args: IA::strs,
// 		output_args: OA::strs,
// 		cb: CB,
// 	) -> &mut MethodDesc
// 	where
// 		IA: arg::ArgAll + arg::ReadAll + Send + 'static,
// 		OA: arg::ArgAll + arg::AppendAll + Send + 'static,
// 		N: Into<strings::Member<'static>>,
// 		CB: FnMut(&mut Context, &mut T, IA) -> R + Send + 'static,
// 		R: Future<Output = Result<OA, MethodErr>> + Send + 'static;
// }
//
// impl<T: Send + 'static> IfaceBuilderExt<T> for IfaceBuilder<T> {
// 	fn method_async<IA, OA, N, R, CB>(
// 		&mut self,
// 		name: N,
// 		input_args: IA::strs,
// 		output_args: OA::strs,
// 		mut cb: CB,
// 	) -> &mut MethodDesc
// 	where
// 		IA: arg::ArgAll + arg::ReadAll + Send + 'static,
// 		OA: arg::ArgAll + arg::AppendAll + Send + 'static,
// 		N: Into<strings::Member<'static>>,
// 		CB: FnMut(&mut Context, &mut T, IA) -> R + Send + 'static,
// 		R: Future<Output = Result<OA, MethodErr>> + Send + 'static,
// 	{
// 		self.method_with_cr_async(
// 			name,
// 			input_args,
// 			output_args,
// 			move |mut ctx, cr, args| async move {
// 				let Some(data) = cr.data_mut(ctx.path()) else {
// 					return ctx.reply(Err(MethodErr::no_path(ctx.path())));
// 				};
// 				let ret = cb(&mut ctx, data, args).await;
// 				ctx.reply(ret)
// 			},
// 		)
// 	}
// }
