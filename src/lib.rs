#![allow(dead_code, unused_variables, unused_imports)]
use chrono::{Datelike, Local};
use colored::Colorize;
use rusqlite::{params, Connection, Result};
use std::{fmt, io, thread::current};

const DB_FILE: &str = "goals.db";

#[derive(Debug, Clone, PartialEq)]
pub struct DayWeekYear {
    day: u32,
    week: u32,
    year: i32,
}

impl DayWeekYear {
    fn new() -> Self {
        let date = Local::now();
        let day = date.ordinal();
        let week = date.iso_week().week();
        let year = date.year();
        Self { day, week, year }
    }
}

impl Default for DayWeekYear {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Goal {
    pub text: String,
    pub week: u32,
    pub year: i32, // in case we travel back in time?
    persisted: bool,
}

impl Goal {
    /// Creates a new [`Goal`] for this week.
    pub fn new(text: String, current_week_year: &DayWeekYear) -> Self {
        let week = current_week_year.week;
        let year = current_week_year.year;

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
                day INTEGER,
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

    pub fn input(current_week_year: Option<&DayWeekYear>) -> Self {
        let mut goal_text = String::new();

        println!("{}", "Weekly goal:".bold());
        io::stdin()
            .read_line(&mut goal_text)
            .expect("Error reading console");

        goal_text = goal_text.trim().to_string();

        Self::new(
            goal_text,
            current_week_year.unwrap_or(&DayWeekYear::default()),
        )
    }

    fn get_current_or_input(today: &DayWeekYear) -> Result<Self> {
        let conn = Connection::open(DB_FILE)?;
        // TODO: create table if needed
        let mut stmt = conn.prepare("SELECT text FROM goals WHERE week = ?1 AND year = ?2")?;
        let mut rows = stmt.query(params![&today.week, &today.year])?;
        let row = rows.next()?;
        if let Some(row) = row {
            Ok(Self::from_db(
                row.get(0).unwrap_or(String::from("")),
                today.week,
                today.year,
            ))
        } else {
            Ok(Self::input(Some(today)))
        }
    }

    pub fn present(&self) {
        println!("Your goal this week (#{}) is {}", self.week, self.text)
    }

    pub fn wizard() -> Self {
        let today = DayWeekYear::new();

        // is there a weekly goal?
        // N: input one
        let goal = Self::get_current_or_input(&today).expect("Error fetching goal");
        goal.present();

        // is there a daily goal for today?
        // N: input one
        let objective =
            DailyObjective::get_current_or_input(&today).expect("Error fetching objective");
        objective.present();
        // are there unfinished (ToDo) tasks?
        // Y: ask which ones should be:
        //    Moved to today's list
        //    Discarded
        //    Kept as ToDo (snoozed)
        let unfinished_tasks = Task::present_unfinished();
        let mut todo_tasks = Task::reprioritize(&unfinished_tasks);
        Task::input_new_tasks(&today, &mut todo_tasks);

        // list existing ToDos
        // ask for new ones, until user is done

        // return aggregator:
        return goal;
    }
}

impl fmt::Display for Goal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl From<&str> for Goal {
    fn from(text: &str) -> Self {
        Self::new(text.to_string(), &DayWeekYear::new())
    }
}

/// The objective for today, in order to achieve a weekly goal
#[derive(Debug, Clone, PartialEq)]
struct DailyObjective {
    pub text: String,
    pub day: u32,
    pub year: i32,
    persisted: bool,
}

// TODO: use traits to reduce duplication with `Goal`
impl DailyObjective {
    fn new(text: String, current_day_year: &DayWeekYear) -> Self {
        let day = current_day_year.day;
        let year = current_day_year.year;
        Self {
            text,
            day,
            year,
            persisted: false,
        }
    }

    fn get_current_or_input(today: &DayWeekYear) -> Result<Self> {
        let conn = Connection::open(DB_FILE)?;
        let mut stmt = conn.prepare("SELECT text FROM goals WHERE day =?1 AND year =?2")?;
        let mut rows = stmt.query(params![&today.day, &today.year])?;
        let row = rows.next()?;
        if let Some(row) = row {
            Ok(Self::from_db(
                row.get(0).unwrap_or(String::from("")),
                today.day,
                today.year,
            ))
        } else {
            Ok(Self::input(Some(today)))
        }
    }

    fn from_db(text: String, day: u32, year: i32) -> Self {
        Self {
            text,
            day,
            year,
            persisted: true,
        }
    }

    fn input(date: Option<&DayWeekYear>) -> Self {
        let mut text = String::new();

        println!("{}", "Today's objective:".bold());
        io::stdin()
            .read_line(&mut text)
            .expect("Error reading console");

        text = text.trim().to_string();

        Self::new(text, date.unwrap_or(&DayWeekYear::default()))
    }

    fn present(&self) {
        println!("Your objetive today (#{}) is {}", self.day, self.text)
    }
}

struct Task {
    text: String,
    status: TaskStatus,
    persisted: bool,
}

enum TaskStatus {
    ToDo,
    Done,
    InProgress,
    Discarded,
}

impl Task {
    fn new(text: String, status: TaskStatus) -> Self {
        Self {
            text,
            status,
            persisted: false,
        }
    }

    fn present_unfinished() -> Option<Vec<Self>> {
        todo!()
    }

    fn reprioritize(unfinished_tasks: &Option<Vec<Self>>) -> Vec<Self> {
        todo!()
    }

    fn input_new_tasks(today: &DayWeekYear, todo_tasks: &mut Vec<Self>) {
        todo_tasks.push(Self::input(today));
        todo!()
    }

    fn input(today: &DayWeekYear) -> Self {
        todo!()
    }

    fn from_db() -> Self {
        todo!()
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
