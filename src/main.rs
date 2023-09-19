use std::borrow::Cow;
use std::collections::HashMap;
use std::process::Command;

use dbus::MethodErr;
use dbus_crossroads::Context;
use krunner_dbus::{ActionInfo, Match, MatchType};
use probly_search::score::zero_to_one;
use probly_search::{Index, QueryResult};
use serde::Deserialize;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Action {
	Run,
}
impl krunner_dbus::Action for Action {
	fn all() -> Vec<Self> {
		vec![Self::Run]
	}

	fn from_id(s: &str) -> Option<Self> {
		match s {
			"run" => Some(Self::Run),
			_ => None,
		}
	}

	fn to_id(&self) -> String {
		match self {
			Self::Run => "run".to_owned(),
		}
	}

	fn info(&self) -> ActionInfo {
		match self {
			Self::Run => ActionInfo {
				text: "Run Nix program".to_owned(),
				icon_source: "system-run-symbolic".to_owned(),
			},
		}
	}
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
struct Program {
	#[serde(default = "String::new")]
	id: String,
	description: String,
	pname: String,
	version: String,
}
impl Program {
	fn indexable_fields(&self) -> Vec<&str> {
		vec![
			self.id.as_str(),
			self.description.as_str(),
			self.pname.as_str(),
		]
	}
}

struct Runner {
	programs: Vec<Program>,
	index: Index<usize>,
}
impl Runner {
	fn new() -> Self {
		let mut index = Index::new(1);

		// TODO: add support for different flakes (i.e. blender-bin)
		let output = Command::new("nix")
			.args([
				"search",
				"nixpkgs",
				"--json",
				"--extra-experimental-features",
				"nix-command",
			])
			.output()
			.expect("could not get nix index")
			.stdout;

		let progs: HashMap<String, Program> =
			serde_json::from_slice(&output).expect("malformed JSON");

		let mut programs = vec![];

		for (i, (id, mut prog)) in progs.into_iter().enumerate() {
			prog.id = id.splitn(3, '.').nth(2).unwrap().to_string();

			index.add_document(&[Program::indexable_fields], tokenizer, i, &prog);
			programs.push(prog);
		}

		println!("Loaded {} programs", programs.len());

		Self { programs, index }
	}
}

impl krunner_dbus::Runner for Runner {
	type Action = Action;
	type Err = MethodErr;

	fn matches(
		&mut self,
		_ctx: &mut Context,
		query: String,
	) -> Result<Vec<Match<Self::Action>>, MethodErr> {
		let matches: Vec<_> = self
			.index
			.query(&query, &mut zero_to_one::new(), tokenizer, &[])
			.into_iter()
			.map(|QueryResult { key, score }| {
				let Program {
					id, description, ..
				} = &self.programs[key];

				Match::new(id.clone())
					.text(format!("Nix: {id}"))
					.subtext(description.clone())
					.icon("nix-snowflake".to_owned())
					.ty(MatchType::PossibleMatch)
					.action(Action::Run)
					.relevance(score)
			})
			.collect();
		Ok(matches)
	}

	fn run(
		&mut self,
		_ctx: &mut Context,
		match_id: String,
		action: Self::Action,
	) -> Result<(), MethodErr> {
		match action {
			Action::Run => {
				let mut cmd = Command::new("nix");
				cmd.args([
					"run",
					&format!("nixpkgs#{match_id}"),
					"--extra-experimental-features",
					"nix-command",
				]);
				dbg!(cmd).spawn().unwrap();
			}
		}
		Ok(())
	}
}

fn tokenizer(s: &str) -> Vec<Cow<str>> {
	s.split(' ').map(Cow::from).collect()
}
fn main() -> Result<(), dbus::Error> {
	krunner_dbus::run(Runner::new(), "me.pluie.krunner_nix", "/krunner_nix")
}
