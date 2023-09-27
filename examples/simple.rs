// Requires the `derive` feature.

use krunner::{Match, RunnerExt};

#[derive(krunner::Action)]
enum Action {
	#[action(id = "run", title = "Run", icon = "running")]
	Run,
}

#[derive(Debug)]
enum Error {}

struct Runner;

impl krunner::Runner for Runner {
	type Action = Action;
	type Err = String;

	fn matches(&mut self, query: String) -> Result<Vec<Match<Self::Action>>, Self::Err> {
		let mut matches = vec![];

		if query == "hi" {
			matches.push(Match {
				id: "hi".to_owned(),
				title: "Hello there!".to_owned(),
				icon: "user-available".to_owned().into(),
				subtitle: Some("This is a sample KRunner match!".to_owned()),

				..Match::default()
			})
		}

		Ok(matches)
	}

	fn run(&mut self, match_id: String, action: Option<Self::Action>) -> Result<(), Self::Err> {
		Ok(())
	}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	Runner.start("your.service.name", "/YourPath")?;
	Ok(())
}
