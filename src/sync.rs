use dbus::blocking::Connection;
use dbus::MethodErr;
use dbus_crossroads::{Context, Crossroads, IfaceToken};

use crate::{Action, ActionInfo, Config, Match};

pub trait Runner {
	type Action: Action;
	type Err: Into<MethodErr>;

	fn matches(
		&mut self,
		ctx: &mut Context,
		query: String,
	) -> Result<Vec<Match<Self::Action>>, Self::Err>;

	fn run(
		&mut self,
		ctx: &mut Context,
		match_id: String,
		action: Option<Self::Action>,
	) -> Result<(), MethodErr>;

	fn config(&mut self, _ctx: &mut Context) -> Result<Config<Self::Action>, Self::Err> {
		todo!()
	}

	fn teardown(&mut self, _ctx: &mut Context) -> Result<(), Self::Err> {
		Ok(())
	}
}

pub fn run<R: Runner + Send + 'static>(
	runner: R,
	service: &'static str,
	path: &'static str,
) -> Result<(), dbus::Error> {
	let c = Connection::new_session()?;
	c.request_name(service, false, true, false)?;

	let mut cr = Crossroads::new();

	let token = register::<R>(&mut cr);
	cr.insert(path, &[token], runner);
	cr.serve(&c)
}

pub fn register<R: Runner + Send + 'static>(cr: &mut Crossroads) -> IfaceToken<R> {
	cr.register("org.kde.krunner1", |b| {
		b.method("Actions", (), ("matches",), |_, _: &mut R, _: ()| {
			let actions: Vec<_> = R::Action::all()
				.into_iter()
				.map(|a| {
					let ActionInfo { text, icon_source } = a.info();
					(a.to_id(), text, icon_source)
				})
				.collect();

			Ok((actions,))
		});
		b.method(
			"Run",
			("matchId", "actionId"),
			(),
			|ctx, runner: &mut R, (match_id, action_id): (String, String)| {
				let action = if let Some(action) = R::Action::from_id(&action_id) {
					Some(action)
				} else if action_id.is_empty() {
					None
				} else {
					return Err(MethodErr::invalid_arg("Unknown action"));
				};
				runner.run(ctx, match_id, action)?;
				Ok(())
			},
		);
		b.method(
			"Match",
			("query",),
			("matches",),
			|ctx, runner, (query,): (String,)| {
				runner.matches(ctx, query).map(|v| (v,)).map_err(Into::into)
			},
		);
		b.method("Config", (), ("config",), |ctx, runner, _: ()| {
			runner.config(ctx).map(|v| (v,)).map_err(Into::into)
		});
		b.method("Teardown", (), (), |ctx, runner, _: ()| {
			runner.teardown(ctx).map_err(Into::into)
		});
	})
}
