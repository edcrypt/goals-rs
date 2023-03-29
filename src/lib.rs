#![allow(dead_code, unused_variables, unused_imports)]
use chrono::{Datelike, Local};
use colored::Colorize;
use inquire::{Confirm, Text};
use rusqlite::{params, Connection, Result};
use std::{
    fmt,
    io::{self, Write},
    thread::current,
};

const DB_FILE: &str = "goals.db";

pub fn wizard() {
    println!("{}", "Goals Wizard".bold());
    let today = DayWeekYear::new();

    // is there a weekly goal?
    // N: input one
    let mut goal = WeeklyGoal::get_current_or_input(&today).expect("Error fetching goal");
    goal.save().expect("Error saving goal");
    goal.present();

    // is there a daily goal for today?
    // N: input one
    let mut objective =
        DailyObjective::get_current_or_input(&today).expect("Error fetching objective");
    objective.save().expect("Error saving objective");
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
}

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
pub struct WeeklyGoal {
    pub text: String,
    pub week: u32,
    pub year: i32, // in case we travel back in time?
    persisted: bool,
}

impl WeeklyGoal {
    /// Creates a new [`Goal`] for this week.
    pub fn new(text: String, date: &DayWeekYear) -> Self {
        let week = date.week;
        let year = date.year;

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

    fn create_table(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS weekly_goals (
                id INTEGER PRIMARY KEY,
            	text TEXT NOT NULL,
            	week INTEGER DEFAULT NULL,
            	year INTEGER NOT NULL,
                UNIQUE (week, year)
			)",
            (),
        )?;
        Ok(())
    }

    fn save(&mut self) -> Result<()> {
        // guard against resaving
        if self.persisted {
            return Ok(());
        }
        // TODO move connection to main()
        let conn = Connection::open(DB_FILE)?;

        Self::create_table(&conn)?;

        conn.execute(
            "INSERT OR REPLACE INTO weekly_goals (text, week, year) VALUES (?1, ?2, ?3)",
            (&self.text, &self.week, &self.year),
        )?;
        self.persisted = true;
        Ok(())
    }

    fn input(date: Option<&DayWeekYear>) -> Self {
        let mut goal_text = Text::new("What is your goal for this week?")
            .with_help_message("What do you want to achieve?")
            .prompt()
            .expect("Error reading goal");

        goal_text = goal_text.trim().to_string();

        Self::new(goal_text, date.unwrap_or(&DayWeekYear::default()))
    }

    pub fn input_and_save() {
        let mut goal = Self::input(None);
        goal.present();
        let confirmation = Confirm::new("Is this correct?")
            .with_default(false)
            .with_help_message("Confirm to store your goal for this week")
            .prompt();

        match confirmation {
            Ok(true) => goal.save().expect("Error saving goal"),
            Ok(false) => println!("Cancelled"),
            Err(_) => println!("Error reading answer, try again later"),
        }
    }

    fn get_current_or_input(today: &DayWeekYear) -> Result<Self> {
        let conn = Connection::open(DB_FILE)?;

        Self::create_table(&conn)?;

        let mut stmt =
            conn.prepare("SELECT text FROM weekly_goals WHERE week = ?1 AND year = ?2")?;
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

    fn present(&self) {
        println!(
            "Your {} this week (#{}) is: {}",
            "Goal".blue().bold(),
            self.week.to_string().bold(),
            self.text.purple().bold()
        )
    }
}

impl fmt::Display for WeeklyGoal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl From<&str> for WeeklyGoal {
    fn from(text: &str) -> Self {
        Self::new(text.to_string(), &DayWeekYear::new())
    }
}

/// The objective for today, in order to achieve a weekly goal
#[derive(Debug, Clone, PartialEq)]
pub struct DailyObjective {
    pub text: String,
    pub day: u32,
    pub year: i32,
    persisted: bool,
}

// TODO: use traits to reduce duplication with `Goal`
// <T> Target
//  |__WeeklyGoal
//  |__DailyObjective
//  |__Task
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

    fn get_current(today: &DayWeekYear) -> Result<Option<Self>> {
        let conn = Connection::open(DB_FILE)?;
        Self::create_table(&conn)?;

        let mut stmt =
            conn.prepare("SELECT text FROM daily_objectives WHERE day =?1 AND year =?2")?;
        let mut rows = stmt.query(params![&today.day, &today.year])?;
        let row = rows.next()?;
        Ok(row.map(|row| {
            Self::from_db(
                row.get(0).unwrap_or(String::from("")),
                today.day,
                today.year,
            )
        }))
    }

    fn get_current_or_input(today: &DayWeekYear) -> Result<Self> {
        let current = Self::get_current(today)?;
        if let Some(current) = current {
            Ok(current)
        } else {
            Ok(Self::input(Some(today), None))
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

    pub fn create_table(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS daily_objectives (
                id INTEGER PRIMARY KEY,
            	text TEXT NOT NULL,
            	year INTEGER NOT NULL,
                day INTEGER DEFAULT NULL,
                UNIQUE (year, day)
			)",
            (),
        )?;
        Ok(())
    }

    fn save(&mut self) -> Result<()> {
        if self.persisted {
            return Ok(());
        }
        let conn = Connection::open(DB_FILE)?;
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO daily_objectives (day, year, text) VALUES (?1, ?2, ?3)",
        )?;
        stmt.execute(params![&self.day, &self.year, &self.text])?;
        self.persisted = true;
        Ok(())
    }

    fn input(date: Option<&DayWeekYear>, current: Option<Self>) -> Self {
        let mut prompt = Text::new("Today's objective:")
            .with_help_message("What to you want to do to get closer to this week's goal?");

        let text = if let Some(current) = current {
            current.text
        } else {
            String::new()
        };

        if !text.is_empty() {
            prompt = prompt.with_default(&text)
        };

        let mut answer = prompt.prompt().expect("Error reading objective");

        answer = answer.trim().to_string();

        Self::new(answer, date.unwrap_or(&DayWeekYear::default()))
    }

    pub fn input_and_save() {
        let today = DayWeekYear::new();

        // The daily objective should be related to the weekly goal
        WeeklyGoal::get_current_or_input(&today)
            .expect("Error getting weekly goal")
            .present();

        let current = Self::get_current(&today).expect("Error getting current objective");

        let mut objective = Self::input(Some(&today), current);
        objective.present();

        // TODO: skip resaving if the objective is default
        let confirmation = Confirm::new("Is this correct?")
            .with_default(false)
            .with_help_message("Confirm to store today's objective")
            .prompt();

        match confirmation {
            Ok(true) => objective.save().expect("Error saving objective"),
            Ok(false) => println!("Cancelled"),
            Err(_) => println!("Error reading answer, try again later"),
        }
    }

    fn present(&self) {
        println!(
            "Your {} today (#{}) is: {}",
            "Objetive".blue().bold(),
            self.day.to_string().bold(),
            self.text.red().bold()
        )
    }
}

#[derive(Debug, Default)]
pub struct Task {
    text: String,
    status: TaskStatus,
    persisted: bool,
}

#[derive(Debug)]
enum TaskStatus {
    ToDo,
    Done,
    Snoozed,
    Discarded,
    // TODO: track progress
    // InProgress,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::ToDo
    }
}

impl Task {
    fn new(text: String, status: TaskStatus) -> Self {
        Self {
            text,
            status,
            persisted: false,
        }
    }

    fn save(&mut self) {
        todo!()
    }

    fn create_table(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                text VARCHAR(255),
                status TEXT NOT NULL DEFAULT 'T'
            )",
            [],
        )?;
        Ok(())
    }

    fn get_todos() -> Result<Option<Vec<Self>>> {
        let conn = Connection::open(DB_FILE)?;

        Self::create_table(&conn)?;

        let mut stmt = conn.prepare("SELECT text FROM tasks WHERE status =?1")?;

        let tasks_iter = stmt.query_map(["T"], |row| {
            Ok(Self::from_db(
                row.get(0).unwrap_or(String::from("")),
                TaskStatus::ToDo,
            ))
        })?;
        let tasks = tasks_iter.collect::<Result<Vec<_>>>()?;
        if tasks.is_empty() {
            Ok(None)
        } else {
            Ok(Some(tasks))
        }
    }

    fn present_unfinished() -> Option<Vec<Task>> {
        let todos = Self::get_todos().expect("Error getting tasks");
        match todos {
            Some(ts) => {
                println!("You have the following unfinished tasks:");
                for task in &ts {
                    println!(" - {}", task.text);
                }
                Some(ts)
            }
            None => None,
        }
    }

    fn reprioritize(unfinished_tasks: &Option<Vec<Self>>) -> Vec<Self> {
        todo!()
    }

    fn input_new_tasks(today: &DayWeekYear, todo_tasks: &mut Vec<Self>) {
        todo_tasks.push(Self::default());
        todo!()
    }

    fn input(today: &DayWeekYear) {
        todo!()
    }

    fn from_db(text: String, status: TaskStatus) -> Self {
        Self {
            text,
            status,
            persisted: false,
        }
    }

    pub fn list() -> Vec<Self> {
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
        let goal1 = WeeklyGoal::default();
        let goal2 = WeeklyGoal::default();
        assert_eq!(goal1.text, "");
        assert_eq!(goal1, goal2);
    }

    #[test]
    fn test_goal_from_str() {
        let goal = WeeklyGoal::from("Hello");
        assert_eq!(goal.text, "Hello")
    }
}
