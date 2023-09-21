use anyhow::Result;
use async_trait::async_trait;
use dbus::MethodErr;
use krunner_dbus::{AsyncRunner, Match};

#[derive(Debug)]
enum Action {}
impl krunner_dbus::Action for Action {
	fn all() -> Vec<Self> {
		vec![]
	}

	fn from_id(s: &str) -> Option<Self> {
		None
	}

	fn to_id(&self) -> String {
		String::new()
	}

	fn info(&self) -> krunner_dbus::ActionInfo {
		krunner_dbus::ActionInfo {
			text: "".to_owned(),
			icon_source: "".to_owned(),
		}
	}
}

struct Runner;

#[async_trait]
impl AsyncRunner for Runner {
	type Action = Action;
	type Err = MethodErr;

	async fn matches(
		&mut self,
		ctx: &mut dbus_crossroads::Context,
		query: String,
	) -> Result<Vec<krunner_dbus::Match<Self::Action>>, Self::Err> {
		if query == "balls" {
			Ok(vec![Match::new("balls".to_string())])
		} else {
			Ok(vec![])
		}
	}

	async fn run(
		&mut self,
		ctx: &mut dbus_crossroads::Context,
		match_id: String,
		action: Self::Action,
	) -> Result<(), MethodErr> {
		Ok(())
	}
}

#[tokio::main]
async fn main() -> Result<()> {
	krunner_dbus::run_async(Runner, "me.pluie.krunner_nix", "/krunner_nix").await?;
	Ok(())
}
