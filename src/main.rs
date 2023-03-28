#![allow(dead_code, unused_variables, unused_imports)]
use clap::{Parser, Subcommand};
use goals::WeeklyGoal;
use human_panic::setup_panic;
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

fn main() {
    setup_panic!();

    let cli = GoalsCli::parse();

    match &cli.command {
        Some(GoalsCommands::Weekly) => WeeklyGoal::input(None),
        Some(GoalsCommands::Daily) => WeeklyGoal::input(None), // TODO...
        Some(GoalsCommands::ListTasks) => WeeklyGoal::input(None), // TODO...
        None => WeeklyGoal::wizard(),
    };
}
