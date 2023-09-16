use std::collections::HashMap;
use std::fmt::Display;
use std::process::Command;
use std::str::FromStr;

use anyhow::Result;
use dbus::blocking::Connection;
use krunner_dbus::{ActionInfo, Context, Match, MatchType, MethodErr, Runner};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Actions {
	LaunchDictionary,
}
impl FromStr for Actions {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"launch-dictionary" => Self::LaunchDictionary,
			_ => return Err(()),
		})
	}
}
impl Display for Actions {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::LaunchDictionary => f.write_str("launch-dictionary"),
		}
	}
}

struct CnrtlRunner;

impl Runner for CnrtlRunner {
	type Action = Actions;
	type Err = MethodErr;

	fn actions(
		&mut self,
		_ctx: &mut Context,
	) -> Result<HashMap<Self::Action, ActionInfo>, MethodErr> {
		Ok(HashMap::from([(Actions::LaunchDictionary, ActionInfo {
			text: "Launch dictionary".to_owned(),
			icon_source: String::new(),
		})]))
	}

	fn matches(
		&mut self,
		_ctx: &mut Context,
		query: String,
	) -> Result<Vec<Match<Self::Action>>, MethodErr> {
		let mut matches = vec![];
		if let Some(word) = query.strip_prefix("def ") {
			matches.push(
				Match::new(word.to_owned())
					.text(format!("Nix: {word}"))
					.icon("nix-snowflake".to_owned())
					.ty(MatchType::ExactMatch)
					.relevance(1.0),
			);
		}
		Ok(dbg!(matches))
	}

	fn run(
		&mut self,
		_ctx: &mut Context,
		match_id: String,
		action: Self::Action,
	) -> Result<(), MethodErr> {
		match action {
			Actions::LaunchDictionary => {
				Command::new("xdg-open")
					.arg(format!("https://www.cnrtl.fr/definition/{match_id}"))
					.output()
					.unwrap();
			}
		}
		Ok(())
	}
}

fn main() -> Result<()> {
	let c = Connection::new_session()?;
	c.request_name("me.pluie.krunner_nix", false, true, false)?;

	krunner_dbus::run_runner(&c, "/krunner_nix", CnrtlRunner)
}
