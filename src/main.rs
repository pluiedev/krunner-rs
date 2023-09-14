use std::process::Command;

use anyhow::Result;
use dbus::blocking::Connection;
use krunner_dbus::{Action, AnyVariant, Context, Match, MatchType, MethodErr, Properties, Runner};

struct CnrtlRunner;

impl Runner for CnrtlRunner {
	fn actions(&mut self, _ctx: &mut Context) -> Result<Vec<Action>, MethodErr> {
		Ok(vec![])
	}

	fn matches(&mut self, _ctx: &mut Context, query: String) -> Result<Vec<Match>, MethodErr> {
		let mut matches = vec![];
		if let Some(word) = query.strip_prefix("def ") {
			matches.push(Match {
				id: word.to_owned(),
				text: format!("Definition: {word}"),
				icon: "internet-web-browser".to_owned(),
				ty: MatchType::ExactMatch,
				relevance: 1.0,
				properties: Properties::default(),
			});
		}
		Ok(matches)
	}

	fn run(
		&mut self,
		_ctx: &mut Context,
		match_id: String,
		_action_id: String,
	) -> Result<(), MethodErr> {
		Command::new("xdg-open")
			.arg(format!("https://www.cnrtl.fr/definition/{match_id}"))
			.output()
			.unwrap();
		Ok(())
	}
}

fn main() -> Result<()> {
	let c = Connection::new_session()?;
	c.request_name("me.pluie.krunner_cnrtl", false, true, false)?;

	krunner_dbus::run_runner(&c, "/krunner_cnrtl", CnrtlRunner)
}
