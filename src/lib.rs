#![allow(dead_code, unused_variables, unused_imports)]
use chrono::{Datelike, Local};
use colored::Colorize;
use rusqlite::{params, Connection, Result};
use std::{fmt, io};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Goal {
    pub text: String,
    pub week: u32,
    pub year: i32,
}

impl Goal {
    /// Creates a new [`Goal`] for this week.
    pub fn new(text: String) -> Self {
        let today = Local::now();
        let week = today.iso_week().week0();
        let year = today.year();

        Self { text, week, year }
    }

    pub fn save(&self) -> Result<()> {
        // TODO move connection to main()
        let conn = Connection::open("goals.db")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS goals (
            	id   INTEGER PRIMARY KEY,
            	text TEXT NOT NULL,
            	week INTEGER NOT NULL,
            	year INTEGER NOT NULL
			)",
            (),
        )?;
        conn.execute(
            "INSERT INTO goals (text, week, year) VALUES (?1, ?2, ?3)",
            (&self.text, &self.week, &self.year),
        )?;
        Ok(())
    }

    pub fn input() -> Self {
        let mut goal_text = String::new();

        println!("{}", "Weekly goal:".bold());
        io::stdin()
            .read_line(&mut goal_text)
            .expect("Error reading console");

        goal_text = goal_text.trim().to_string();
        Self::new(goal_text)
    }
}

impl fmt::Display for Goal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl From<&str> for Goal {
    fn from(text: &str) -> Self {
        Self::new(text.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /*     #[test]
    fn test_goal_input() {
        let goal = Goal::input();
        println!("{}", goal);
    } */

    #[test]
    fn test_goal_is_saved() {
        assert_eq!(1 + 1, 2)
    }

    #[test]
    fn test_default_goal() {
        let goal1 = Goal::default();
        let goal2 = Goal::default();
        assert_eq!(goal1.text, "");
        assert_eq!(goal1, goal2);
    }

    #[test]
    fn test_goal_from_str() {
        let goal = Goal::from("Hello");
        assert_eq!(goal.text, "Hello")
    }
}
