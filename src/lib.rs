#![allow(dead_code, unused_variables, unused_imports)]
use chrono::{Datelike, Local};
use colored::Colorize;
use rusqlite::{params, Connection, Result};
use std::{fmt, io, thread::current};

const DB_FILE: &str = "goals.db";

#[derive(Debug, Clone, PartialEq)]
pub struct CurrentWeekYear(u32, i32);

impl CurrentWeekYear {
    fn new() -> Self {
        let today = Local::now();
        let week = today.iso_week().week0();
        let year = today.year();
        Self(week, year)
    }
}

impl Default for CurrentWeekYear {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Goal {
    pub text: String,
    pub week: u32,
    pub year: i32,
    persisted: bool,
}

impl Goal {
    /// Creates a new [`Goal`] for this week.
    pub fn new(text: String, current_week_year: &CurrentWeekYear) -> Self {
        let week = current_week_year.0;
        let year = current_week_year.1;

        Self {
            text,
            week,
            year,
            persisted: false,
        }
    }

    fn from_db(text: String, week: u32, year: i32) -> Self {
        Self {
            text,
            week,
            year,
            persisted: true,
        }
    }

    pub fn save(&mut self) -> Result<()> {
        // guard against resaving
        if self.persisted {
            return Ok(());
        }
        // TODO move connection to main()
        let conn = Connection::open(DB_FILE)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS goals (
            	text TEXT NOT NULL,
            	week INTEGER NOT NULL,
            	year INTEGER NOT NULL,
                PRIMARY KEY (week, year)
			)",
            (),
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO goals (text, week, year) VALUES (?1, ?2, ?3)",
            (&self.text, &self.week, &self.year),
        )?;
        self.persisted = true;
        Ok(())
    }

    pub fn input(current_week_year: Option<&CurrentWeekYear>) -> Self {
        let mut goal_text = String::new();

        println!("{}", "Weekly goal:".bold());
        io::stdin()
            .read_line(&mut goal_text)
            .expect("Error reading console");

        goal_text = goal_text.trim().to_string();

        Self::new(
            goal_text,
            current_week_year.unwrap_or(&CurrentWeekYear::default()),
        )
    }

    fn get_current_or_input(current_week_year: &CurrentWeekYear) -> Result<Self> {
        let conn = Connection::open(DB_FILE)?;
        let mut stmt = conn.prepare("SELECT text FROM goals WHERE week = ?1 AND year = ?2")?;
        let mut rows = stmt.query(params![&current_week_year.0, &current_week_year.1])?;
        let row = rows.next()?;
        if let Some(row) = row {
            Ok(Self::from_db(
                row.get(0).unwrap_or(String::from("")),
                current_week_year.0,
                current_week_year.1,
            ))
        } else {
            Ok(Self::input(Some(current_week_year)))
        }
    }

    pub fn present(&self) {
        println!("Your goal this week: {}", self.text)
    }

    pub fn wizard() -> Self {
        let current_week_year = CurrentWeekYear::new();

        // is there a weekly goal?
        // N: input one
        let current = Self::get_current_or_input(&current_week_year).expect("Error fetching goal");
        current.present();

        // is there a daily goal for today?
        // N: input one

        // are there unfinished (ToDo) tasks?
        // Y: ask which ones should be:
        //    Moved to today's list
        //    Discarded
        //    Kept as ToDo (snoozed)

        // list existing ToDos
        // ask for new ones, until user is done

        // return aggregator:
        return current;
    }
}

impl fmt::Display for Goal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl From<&str> for Goal {
    fn from(text: &str) -> Self {
        Self::new(text.to_string(), &CurrentWeekYear::new())
    }
}

struct Task {
    text: String,
    status: TaskStatus,
}

enum TaskStatus {
    ToDo,
    Done,
    InProgress,
    Discarded,
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
