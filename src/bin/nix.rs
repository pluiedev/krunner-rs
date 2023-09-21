use std::borrow::Cow;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context as _, Result};
use async_trait::async_trait;
use dbus::arg::Variant;
use dbus::blocking::{Connection, Proxy};
use dbus::nonblock::SyncConnection;
use dbus::MethodErr;
use dbus_crossroads::Context;
use krunner_dbus::{ActionInfo, Match, MatchType};
use probly_search::score::zero_to_one;
use probly_search::{Index, QueryResult};
use serde::Deserialize;
use tokio::process::Command;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Action {
	Run,
	Shell,
}
impl krunner_dbus::Action for Action {
	fn all() -> Vec<Self> {
		vec![Self::Run, Self::Shell]
	}

	fn from_id(s: &str) -> Option<Self> {
		match s {
			"run" => Some(Self::Run),
			"shell" => Some(Self::Shell),
			_ => None,
		}
	}

	fn to_id(&self) -> String {
		match self {
			Self::Run => "run",
			Self::Shell => "shell",
		}
		.to_owned()
	}

	fn info(&self) -> ActionInfo {
		match self {
			Self::Run => ActionInfo {
				text: "Run Nix program".to_owned(),
				icon_source: "system-run-symbolic".to_owned(),
			},
			Self::Shell => ActionInfo {
				text: "Spawn a new shell with Nix program".to_owned(),
				icon_source: "new-command-alarm".to_owned(),
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
	conn: Arc<SyncConnection>,
	programs: Vec<Program>,
	index: Index<usize>,
}
impl Runner {
	async fn new() -> Result<Self> {
		let (resource, conn) =
			dbus_tokio::connection::new_session_sync().context("failed to start new connection")?;
		tokio::spawn(async {
			panic!("Lost connection to D-Bus: {}", resource.await);
		});

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
			.await
			.context("could not get nix index")?
			.stdout;

		let progs: HashMap<String, Program> =
			serde_json::from_slice(&output).context("got malformed JSON from Nix")?;

		let programs: Vec<_> = progs
			.into_iter()
			.enumerate()
			.map(|(i, (id, mut prog))| {
				prog.id = id.splitn(3, '.').nth(2).unwrap().to_string();

				index.add_document(&[Program::indexable_fields], tokenizer, i, &prog);
				prog
			})
			.collect();

		println!("Loaded {} programs", programs.len());

		Ok(Self {
			conn,
			programs,
			index,
		})
	}

	fn to_match(
		&self,
		query: &str,
		QueryResult { key, score }: QueryResult<usize>,
	) -> Match<Action> {
		let Program {
			id,
			description,
			version,
			..
		} = &self.programs[key];

		let mut text = format!("Nix: {id}");
		if !version.is_empty() {
			text.push_str(" (");
			text.push_str(version);
			text.push(')');
		}

		let ty = if query.trim().eq_ignore_ascii_case(id) {
			MatchType::ExactMatch
		} else {
			MatchType::PossibleMatch
		};

		Match::new(id.clone())
			.text(text)
			.subtext(description.clone())
			.icon("nix-snowflake".to_owned())
			.ty(ty)
			.action(Action::Run)
			.action(Action::Shell)
			.relevance(score)
	}
}

#[async_trait]
impl krunner_dbus::AsyncRunner for Runner {
	type Action = Action;
	type Err = MethodErr;

	async fn matches(
		&mut self,
		_ctx: &mut Context,
		query: String,
	) -> Result<Vec<Match<Self::Action>>, MethodErr> {
		let matches: Vec<_> = self
			.index
			.query(&query, &mut zero_to_one::new(), tokenizer, &[])
			.into_iter()
			.map(|q| self.to_match(&query, q))
			.take(10)
			.collect();
		Ok(matches)
	}

	async fn run(
		&mut self,
		_ctx: &mut Context,
		match_id: String,
		action: Option<Self::Action>,
	) -> Result<(), MethodErr> {
		match action {
			Some(Action::Shell) => {
				Command::new("konsole")
					.args([
						"-e",
						"nix",
						"shell",
						&format!("nixpkgs#{match_id}"),
						"--extra-experimental-features",
						"nix-command",
					])
					.spawn()
					.unwrap();
			}
			None | Some(Action::Run) => {
				Command::new("konsole")
					.args([
						"-e",
						"nix",
						"run",
						&format!("nixpkgs#{match_id}"),
						"--extra-experimental-features",
						"nix-command",
					])
					.spawn()
					.unwrap();
			}
		}
		Ok(())
	}
}

fn tokenizer(s: &str) -> Vec<Cow<str>> {
	s.split(' ').map(Cow::from).collect()
}

#[tokio::main]
async fn main() -> Result<()> {
	let runner = Runner::new().await?;
	krunner_dbus::run_async(runner, "me.pluie.krunner_nix", "/krunner_nix").await?;
	Ok(())
}
