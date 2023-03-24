#![allow(dead_code, unused_variables, unused_imports)]
use clap::Parser;
use goals::Goal;
use rusqlite::{params, Connection, Result};

#[derive(Parser)] // requires `derive` feature
#[command(name = "goals")]
#[command(bin_name = "goals")]
enum GoalsCli {
    Weekly(WeeklyGoalArgs),
}

#[derive(clap::Args, Debug)]
#[command(author, version, about, long_about = "List/Set your weekly goal(s)")]
struct WeeklyGoalArgs {
    #[arg(long)]
    text: Option<String>,
}

fn main() -> Result<()> {
    let GoalsCli::Weekly(args) = GoalsCli::parse();
    let goal = Goal::input();

    print!("{:?}", args);
    println!("Weekly goal: {:?}", goal);

    goal.save()
}
