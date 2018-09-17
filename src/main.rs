extern crate base64;
extern crate chrono;
#[macro_use]
extern crate clap;
extern crate crypto;
extern crate csv;
#[macro_use]
extern crate failure;
extern crate futures;
#[cfg_attr(test, macro_use)] 
extern crate indoc;
extern crate itertools;
extern crate ratelimit;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate tokio_core;

use std::process;

use chrono::prelude::Utc;

mod config;
mod input;
mod output;
mod query;
mod request;
mod response;
mod url;

type Result<T> = std::result::Result<T, failure::Error>;

fn main() {
    let args = config::run();
    if let Err(err) = run(&args) {
        eprintln!("{}", format_error(&err));
        let backtrace = err.backtrace().to_string();
        if !backtrace.trim().is_empty() {
            eprintln!("{}", backtrace);
        }
        process::exit(1);
    }
}

fn format_error(err: &failure::Error) -> String {
    let mut out = "Error occurred: ".to_string();
    out.push_str(&err.to_string());
    let mut prev = err.as_fail();
    while let Some(next) = prev.cause() {
        out.push_str("\n -> ");
        out.push_str(&next.to_string());
        prev = next;
    }
    out
}

fn run(args: &config::Args) -> Result<()> {
    // Load CSV input from specified path or STDIN.
    let input_str = input::load(&args.input_path)?;

    // Build and validate queries.
    let current_time = Utc::now().naive_utc();
    let queries = input::read_csv(&input_str, &current_time)?;

    // Generate request URLs.
    let requests: Vec<_> = queries
        .into_iter()
        .map(|q| url::TaggedUrl::new(&q, &args.credentials).unwrap())
        .collect();

    // Collect responses.
    let output = request::execute_requests(&requests, 50)?;
    let output_json = serde_json::to_string(&output)?;

    // Export results.
    output::export(&args.output_path, &output_json)?;
    Ok(())
}
