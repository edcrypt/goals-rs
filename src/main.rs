use chrono::{Datelike, Local};
use colored::Colorize;
use rusqlite::{params, Connection, Result};
use std::io;

#[derive(Debug)]
struct Goal {
    text: String,
    week: u32,
    year: i32,
}

impl Goal {
    fn save(&self) -> Result<()> {
        // TODO move connection to main()
        let conn = Connection::open_in_memory()?;

        conn.execute(
            "CREATE TABLE goals (
            	id   INTEGER PRIMARY KEY,
            	text TEXT NOT NULL,
            	week INTEGER NOT NULL,
            	year INTEGER NOT NULL
			)",
            (),
        )?;
        conn.execute(
            "INSERT INTO goals (id, text, week, year) VALUES (?1, ?2, ?3, ?4)",
            (1, &self.text, &self.week, &self.year),
        )?;
        Ok(())
    }

    fn input() -> Self {
        let mut goal_text = String::new();
        let today = Local::now();
        let week_number = today.iso_week().week0();
        let year = today.year();

        println!("{}", "Weekly goal:".bold());
        io::stdin()
            .read_line(&mut goal_text)
            .expect("Error reading console");
        Goal {
            text: goal_text,
            week: week_number,
            year: year,
        }
    }
}

fn main() -> Result<()> {
    let goal = Goal::input();

    println!("Weekly goal: {:?}", goal);

    goal.save()
}

#[cfg(test)]
mod tests {
    #[test]
    fn goal_is_saved() {
        assert_eq!(1 + 1, 2)
    }
}
