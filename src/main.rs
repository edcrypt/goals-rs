#![allow(dead_code, unused_variables, unused_imports)]
use clap::{Parser, Subcommand};
use goals::Goal;
use rusqlite::Result;

const NAME: &str = "Goals";
const BIN: &str = "goals";
const AUTHOR: &str = "Eduardo Padoan";
const EMAIL: &str = "eduardo.padoan@gmail.com";
const VERSION: &str = "0.0.1b";
const ABOUT: &str = "Set productivity goals and daily tasks to achieve them";

#[derive(Subcommand)]
enum GoalsCommands {
    Weekly,
    Daily,
    ListTasks,
}

#[derive(Parser)]
#[command(name = NAME)]
#[command(bin_name = BIN)]
#[command(author = "{AUTHOR} <{EMAIL}>")]
#[command(version = VERSION)]
#[command(about = ABOUT, long_about = None)]
struct GoalsCli {
    #[command(subcommand)]
    command: Option<GoalsCommands>,
}

fn main() -> Result<()> {
    let cli = GoalsCli::parse();

    let mut goal = match &cli.command {
        Some(GoalsCommands::Weekly) => Goal::input(None),
        Some(GoalsCommands::Daily) => Goal::input(None), // TODO...
        Some(GoalsCommands::ListTasks) => Goal::input(None), // TODO...
        None => Goal::wizard(),
    };
    goal.save()
}
