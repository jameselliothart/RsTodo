
mod done {
    use core::fmt;

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

    pub fn days_ago(start_date: DateTime<Utc>, num_days: i64) -> DateTime<Utc> {
        let duration = Duration::days(num_days);
        start_date - duration
    }

    pub fn weeks_ago(start_date: DateTime<Utc>, num_weeks: i64) -> DateTime<Utc> {
        let duration = Duration::weeks(num_weeks);
        start_date - duration
    }

fn main() {
    println!("Hello, world!");
}
