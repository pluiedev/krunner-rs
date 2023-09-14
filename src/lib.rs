use anyhow::Result;
use dbus::blocking::Connection;
use dbus_crossroads::Crossroads;
pub use dbus_crossroads::{Context, MethodErr};

pub use crate::types::*;

pub mod types;

pub trait Runner {
	fn actions(&mut self, ctx: &mut Context) -> Result<Vec<Action>, MethodErr>;
	fn matches(&mut self, ctx: &mut Context, query: String) -> Result<Vec<Match>, MethodErr>;

	fn run(
		&mut self,
		ctx: &mut Context,
		match_id: String,
		action_id: String,
	) -> Result<(), MethodErr>;

	fn config(&mut self, ctx: &mut Context) -> Result<Config, MethodErr> {
		todo!()
	}

	fn teardown(&mut self, ctx: &mut Context) -> Result<(), MethodErr> {
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
			runner.actions(ctx).map(|v| (v,))
		});
		b.method(
			"Run",
			("matchId", "actionId"),
			(),
			|ctx, runner: &mut R, (match_id, action_id): (String, String)| {
				runner.run(ctx, match_id, action_id)
			},
		);
		b.method(
			"Match",
			("query",),
			("matches",),
			|ctx, runner: &mut R, (query,): (String,)| runner.matches(ctx, query).map(|v| (v,)),
		);
		b.method("Config", (), ("config",), |ctx, runner: &mut R, _: ()| {
			runner.config(ctx).map(|v| (v,))
		});
		b.method("Teardown", (), (), |ctx, runner: &mut R, _: ()| {
			runner.teardown(ctx)
		});
	});
	cr.insert(path, &[token], runner);
	cr.serve(&c)?;

	Ok(())
}
