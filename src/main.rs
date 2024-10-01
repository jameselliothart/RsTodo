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

/*
// let result =
// try
//     s
//     |> fun s -> Regex.Match(s, "^\[(?<completedOn>.*)\] (?<item>.*)")
//     |> fun m -> if m.Success then Some m.Groups else None
//     |> Option.map (
//         fun g -> (
//                     (g |> Seq.filter (fun x -> x.Name = "completedOn") |> Seq.exactlyOne).Value,
//                     (g |> Seq.filter (fun x -> x.Name = "item") |> Seq.exactlyOne).Value
//                 )
//         )
//     |> Option.map (fun (date, item) -> (DateTime.TryParse(date), item))
//     |> Option.map (fun ((success, date), item) -> if success then Some (create date item) else None)
// with
// | :? ArgumentException -> None
// match result with
// | Some(Some(completedItem)) -> Some completedItem
// | _ -> None

// mod todo {
//     struct Todo(usize, String);

//     pub enum TodoList {
//         Todos(Vec<Todo>),
//         Nothing,
//     }

//     pub enum TodoEvent {
//         TodoAddedEvent(String),
//         TodosRemainingEvent(TodoList),
//         TodosCompletedEvent(TodoList),
//         TodosPurgedEvent(TodoList),
//     }

//     pub fn create(todos: &[&str]) -> TodoList {
//         if todos.is_empty() {return TodoList::Nothing}
//         let todos_vec = todos
//             .iter()
//             .enumerate()
//             .map(|(index, item)| Todo(index, item.to_string()))
//             .collect();
//         TodoList::Todos(todos_vec)
//     }

//     pub fn value(Todo(_, item): Todo) -> String {item}
//     pub fn index(Todo(index, _): Todo) -> usize {index}

// }
*/

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
    let path = Path::new("./done.txt");
    let cli = Cli::parse();
    match &cli.command {
        Commands::A { tasks } => {
            let items = tasks
                .iter()
                .map(|i| done::CompletedItem::now(i.to_string()))
                .collect();
            done::file::save(path, items)
        }
        Commands::D { days } => {
            let completed_since = done::days_ago(Utc::now(), *days);
            done::file::get(path, completed_since)
                .iter()
                .map(|x| x.to_string())
                .for_each(|item| println!("{}", item));
        }
        Commands::W { weeks } => {
            let completed_since = done::weeks_ago(Utc::now(), *weeks);
            done::file::get(path, completed_since)
                .iter()
                .map(|x| x.to_string())
                .for_each(|item| println!("{}", item));
        }
    }
}
