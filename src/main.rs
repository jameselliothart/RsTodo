
mod done {
    use core::fmt;
    use std::{
        fs::{File, OpenOptions},
        io::{self, BufRead, Write},
        path::Path,
    };

    use chrono::{DateTime, Duration, Utc};
    use regex::Regex;

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

    pub fn days_ago(start_date: DateTime<Utc>, num_days: i64) -> DateTime<Utc> {
        let duration = Duration::days(num_days);
        start_date - duration
    }

    pub fn weeks_ago(start_date: DateTime<Utc>, num_weeks: i64) -> DateTime<Utc> {
        let duration = Duration::weeks(num_weeks);
        start_date - duration
    }

    pub fn get(path: &Path, completed_since: DateTime<Utc>) -> Vec<CompletedItem> {
        if let Ok(lines) = read_lines(path) {
            let items = lines
                .flat_map(|item| item.map(|i| try_parse(&i)))
                .filter(|r| r.as_ref().is_ok_and(|ci| ci.completed > completed_since))
                .map(|r| r.unwrap())
                .collect::<Vec<_>>();
            return items;
        }
        return Vec::new();
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

fn main() {
    println!("Hello, world!");
}
