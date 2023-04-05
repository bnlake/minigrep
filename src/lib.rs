use std::error::Error;
use std::{env, fs};

pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        // Throw away the first argument (application path)
        args.next();

        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a string to query"),
        };

        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a filepath"),
        };

        let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Config {
            query,
            file_path,
            ignore_case,
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;

    let strategy = search_strategy_factory(config.ignore_case)
        .expect("Should have returned a search strategy");
    let results = strategy.search(&config.query, &contents);

    for line in results {
        println!("{}", line);
    }

    Ok(())
}

pub trait SearchStrategy {
    fn search<'a>(&self, query: &str, contents: &'a str) -> Vec<&'a str>;
}

#[derive(Default)]
pub struct CaseInsensitiveSearch;

impl SearchStrategy for CaseInsensitiveSearch {
    fn search<'a>(&self, query: &str, contents: &'a str) -> Vec<&'a str> {
        let query = query.to_lowercase();
        let mut results = Vec::new();

        for line in contents.lines() {
            if line.to_lowercase().contains(&query) {
                results.push(line);
            }
        }

        results
    }
}

#[derive(Default)]
pub struct CaseSensitiveSearch;

impl SearchStrategy for CaseSensitiveSearch {
    fn search<'a>(&self, query: &str, contents: &'a str) -> Vec<&'a str> {
        let mut results: Vec<&str> = Vec::new();

        for line in contents.lines() {
            if line.contains(query) {
                results.push(line);
            }
        }

        results
    }
}

pub fn search_strategy_factory(ignore_case: bool) -> Option<Box<dyn SearchStrategy>> {
    match ignore_case {
        false => Some(Box::new(CaseSensitiveSearch)),
        true => Some(Box::new(CaseInsensitiveSearch)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_contents() -> &'static str {
        "Rust:
safe, fast, productive.
Pick three.
Trust me"
    }

    #[test]
    fn case_sensitive_search() {
        let query = "Rust";
        let contents = get_contents();

        let result = CaseSensitiveSearch.search(query, contents);

        assert_eq!(vec!["Rust:"], result);
    }

    #[test]
    fn case_insensitive_search() {
        let query = "RuSt";
        let contents = get_contents();

        let result = CaseInsensitiveSearch.search(query, contents);

        assert_eq!(vec!["Rust:", "Trust me"], result);
    }
}
