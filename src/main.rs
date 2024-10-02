use std::path::Path;

use chrono::Utc;
use clap::{command, Parser, Subcommand};

mod done {
    use core::fmt;

    use chrono::{DateTime, Duration, Utc};

    pub struct CompletedItem {
        pub completed: DateTime<Utc>,
        pub item: String,
    }

    impl fmt::Display for CompletedItem {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "[{}] {}", self.completed, self.item)
        }
    }

    impl CompletedItem {
        pub fn new(item: String, completed_on: Option<DateTime<Utc>>) -> Self {
            let completed = completed_on.unwrap_or(Utc::now());
            Self { completed, item }
        }

        pub fn now(item: String) -> Self {
            Self::new(item, None)
        }
    }

    pub fn days_ago(start_date: DateTime<Utc>, num_days: i64) -> DateTime<Utc> {
        let duration = Duration::days(num_days);
        start_date - duration
    }

    pub fn weeks_ago(start_date: DateTime<Utc>, num_weeks: i64) -> DateTime<Utc> {
        let duration = Duration::weeks(num_weeks);
        start_date - duration
    }

    pub mod db {
        use chrono::{DateTime, Utc};
        use sqlite::Connection;

        use super::CompletedItem;

        pub const DB_PATH: &str = "done.db";

        const SQL_CREATE_TABLE: &str = "
            CREATE TABLE IF NOT EXISTS CompletedItems (
                Id INTEGER PRIMARY KEY,
                CompletedOn TIMESTAMP,
                Item VARCHAR(255)
                )";

        const SQL_INSERT_ITEM: &str = "INSERT INTO CompletedItems (CompletedOn, Item) VALUES (?, ?)";

        const SQL_GET_ITEMS: &str = "SELECT Item, CompletedOn from CompletedItems WHERE CompletedOn > ?";

        pub fn new_connection(db_path: &str) -> Connection {
            return sqlite::open(db_path).expect(&format!("Should be able to open connection to db file at {}", db_path));
        }

        pub fn initialize(connection: &Connection) {
            connection.execute(SQL_CREATE_TABLE).unwrap()
        }

        pub fn insert_item(connection: &Connection, completed_item: &CompletedItem) -> Result<(), sqlite::Error> {
            let mut statement = connection.prepare(SQL_INSERT_ITEM)?;

            // Bind the parameters: 1st index is ?, 2nd index is ?
            statement.bind((1, completed_item.completed.to_rfc3339().as_str()))?;
            statement.bind((2, completed_item.item.as_str()))?;

            // Execute the statement
            statement.next()?;
            Ok(())
        }

        pub fn save(connection: &Connection, completed_items: Vec<CompletedItem>) -> Vec<sqlite::Error> {
            return completed_items
                .iter()
                .map(|ci| insert_item(connection, ci))
                .filter_map(|r| r.err())
                .collect::<Vec<_>>()
                ;
        }

        pub fn get(connection: &Connection, completed_since: DateTime<Utc>) -> Vec<CompletedItem> {
            return connection
                .prepare(SQL_GET_ITEMS)
                .unwrap()
                .into_iter()
                .bind((1, completed_since.to_rfc3339().as_str()))
                .unwrap()
                .filter_map(|row| row.ok())
                .map(
                    |row| CompletedItem {
                        completed: row.read::<&str, _>("CompletedOn").parse::<DateTime<Utc>>().expect(&format!("Should be able to parse '{}' to DateTime<Utc>", row.read::<&str, _>("CompletedOn"))),
                        item: row.read::<&str, _>("Item").to_string()
                    }
                )
                .collect()
                ;
        }
    }

    pub mod file {
        use std::{
            fs::{File, OpenOptions},
            io::{self, BufRead, Write},
            path::Path,
        };
        use chrono::{DateTime, Utc};
        use regex::Regex;

        use super::CompletedItem;

        fn try_parse(done_item: &str) -> Result<CompletedItem, String> {
            let re = Regex::new(r"^\[(?<completedOn>.*)\] (?<item>.*)").unwrap();
            let Some(captures) = re.captures(done_item) else {
                return Err(format!(
                    "Unable to parse completed on timestamp and item from {}",
                    done_item
                ));
            };
            let item = captures["item"].to_string();
            let completed = captures["completedOn"].parse::<DateTime<Utc>>();
            let parsed = completed.map_or_else(
                |e| {
                    Err(format!(
                        "Error parsing completed timestamp '{}': {}",
                        &captures["completedOn"], e
                    ))
                },
                |completed| Ok(CompletedItem { completed, item }),
            );
            return parsed;
        }

        fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
        where
            P: AsRef<Path>,
        {
            let file = File::open(filename)?;
            Ok(io::BufReader::new(file).lines())
        }

        pub fn get(path: &Path, completed_since: DateTime<Utc>) -> Vec<CompletedItem> {
            let items = read_lines(path)
                .into_iter()
                .flat_map(|lines| lines)
                .filter_map(|line| line.ok())
                .filter_map(|line| try_parse(&line).ok())
                .filter(|ci| ci.completed > completed_since)
                .collect();
            items
        }

        fn append_lines(path: &Path, lines: Vec<String>) -> io::Result<()> {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true) // Create the file if it doesn't exist
                .open(path)?; // Return an error if unable to open

            // Write the lines to the file with newline character
            for line in lines {
                writeln!(file, "{}", line)?;
            }

            Ok(())
        }

        pub fn save(path: &Path, completed_items: Vec<CompletedItem>) {
            let lines = completed_items
                .iter()
                .map(|ci| ci.to_string())
                .collect::<Vec<_>>();
            append_lines(path, lines).expect(&format!(
                "Should be able to write to file at path: '{:?}'",
                path
            ))
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    A { tasks: Vec<String> },
    D { days: i64 },
    W { weeks: i64 },
}

#[derive(Parser)]
#[command(name = "done")]
#[command(about = "A simple command-line todo manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    // let path = Path::new("./done.txt");
    let connection = done::db::new_connection(done::db::DB_PATH);
    done::db::initialize(&connection);
    let cli = Cli::parse();
    match &cli.command {
        Commands::A { tasks } => {
            let items = tasks
                .iter()
                .map(|i| done::CompletedItem::now(i.to_string()))
                .collect();
            done::db::save(&connection, items);
        }
        Commands::D { days } => {
            let completed_since = done::days_ago(Utc::now(), *days);
            done::db::get(&connection, completed_since)
                .iter()
                .map(|x| x.to_string())
                .for_each(|item| println!("{}", item));
        }
        Commands::W { weeks } => {
            let completed_since = done::weeks_ago(Utc::now(), *weeks);
            done::db::get(&connection, completed_since)
                .iter()
                .map(|x| x.to_string())
                .for_each(|item| println!("{}", item));
        }
    }
}
