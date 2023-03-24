#![allow(dead_code, unused_variables, unused_imports)]
use clap::{Parser, Subcommand};
use goals::Goal;
use rusqlite::Result;

#[derive(Subcommand)]
enum GoalsCommands {
    Weekly,
    Daily,
    ListTasks,
}

#[derive(Parser)]
#[command(name = "goals")]
#[command(bin_name = "goals")]
struct GoalsCli {
    #[command(subcommand)]
    command: Option<GoalsCommands>,
}

fn main() -> Result<()> {
    let cli = GoalsCli::parse();

    let goal: Option<Goal> = match &cli.command {
        Some(GoalsCommands::Weekly) => Some(Goal::input()),
        Some(GoalsCommands::Daily) => Some(Goal::input()), // TODO...
        Some(GoalsCommands::ListTasks) => Some(Goal::input()), // TODO...
        None => None,
    };

    if let Some(goal) = goal {
        goal.save()
    } else {
        Ok(())
    }
}
