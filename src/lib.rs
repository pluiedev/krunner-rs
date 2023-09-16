use std::collections::HashMap;
use std::str::FromStr;

use anyhow::Result;
use dbus::blocking::Connection;
use dbus_crossroads::Crossroads;
pub use dbus_crossroads::{Context, MethodErr};

pub use crate::types::*;

pub mod types;

pub trait Runner {
	type Action: FromStr + ToString;
	type Err: Into<MethodErr>;

	fn actions(
		&mut self,
		ctx: &mut Context,
	) -> Result<HashMap<Self::Action, ActionInfo>, Self::Err>;
	fn matches(
		&mut self,
		ctx: &mut Context,
		query: String,
	) -> Result<Vec<Match<Self::Action>>, Self::Err>;

	fn run(
		&mut self,
		ctx: &mut Context,
		match_id: String,
		action: Self::Action,
	) -> Result<(), MethodErr>;

	fn config(&mut self, ctx: &mut Context) -> Result<Config<Self::Action>, Self::Err> {
		todo!()
	}

	fn teardown(&mut self, ctx: &mut Context) -> Result<(), Self::Err> {
		Ok(())
	}
}

pub fn run_runner<R: Runner + Send + 'static>(
	c: &Connection,
	path: &'static str,
	runner: R,
) -> Result<()> {
	let mut cr = Crossroads::new();
	let token = cr.register("org.kde.krunner1", |b| {
		b.method("Actions", (), ("matches",), |ctx, runner: &mut R, _: ()| {
			runner
				.actions(ctx)
				.map(|h| {
					(h.into_iter()
						.map(|(id, v)| (id.to_string(), v.text, v.icon_source))
						.collect::<Vec<_>>(),)
				})
				.map_err(Into::into)
		});
		b.method(
			"Run",
			("matchId", "actionId"),
			(),
			|ctx, runner: &mut R, (match_id, action_id): (String, String)| {
				let Ok(action) = R::Action::from_str(&action_id) else {
					return Err(MethodErr::invalid_arg("Failed to parse action"));
				};
				runner.run(ctx, match_id, action).map_err(Into::into)
			},
		);
		b.method(
			"Match",
			("query",),
			("matches",),
			|ctx, runner: &mut R, (query,): (String,)| {
				runner.matches(ctx, query).map(|v| (v,)).map_err(Into::into)
			},
		);
		b.method("Config", (), ("config",), |ctx, runner: &mut R, _: ()| {
			runner.config(ctx).map(|v| (v,)).map_err(Into::into)
		});
		b.method("Teardown", (), (), |ctx, runner: &mut R, _: ()| {
			runner.teardown(ctx).map_err(Into::into)
		});
	});
	cr.insert(path, &[token], runner);
	cr.serve(&c)?;

	Ok(())
}
