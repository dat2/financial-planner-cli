#![recursion_limit = "1024"]

#[macro_use]
extern crate clap;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate chrono;
extern crate prettytable;
extern crate rugflo;
#[macro_use]
extern crate error_chain;

mod money;
mod plan;
mod accounts;
mod iterators;
mod errors;

use std::fs::File;
use clap::{Arg, App, SubCommand};
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;

use plan::*;
use errors::*;

fn print_forecast(plan: Plan, years: usize) {
    let mut table = Table::new();

    let mut header = Vec::new();
    header.push(Cell::new("Date"));
    for name in plan.accounts.get_account_names() {
        header.push(Cell::new(&name));
    }
    table.add_row(Row::new(header));

    for (date, moment) in plan.history(YearStream::new().take(years)) {
        let mut result = Vec::new();

        result.push(Cell::new(&format!("{}", date)));

        for (name, account) in moment.flatten_with_path() {
            if name.find("income:").is_none() {
                result.push(Cell::new(&account.amount().to_string()));
            }
        }

        table.add_row(Row::new(result));
    }

    table.printstd();
}

fn run() -> Result<()> {
    let matches = App::new("financial-planner")
        .version("0.1")
        .author("Nicholas D. <nickdujay@gmail.com>")
        .about("Helps you plan your financial future.")
        .arg(Arg::with_name("input")
            .short("f")
            .value_name("INPUT")
            .help("Sets the input file to use.")
            .takes_value(true))
        .subcommand(SubCommand::with_name("forecast")
            .about("Calculate Asset values over <n> years.")
            .arg(Arg::with_name("years")
                .help("Sets the number of years to calculate forward.")
                .index(1)))
        .get_matches();

    let input_file = File::open(matches.value_of("input").unwrap_or("input.yaml"))?;
    let plan: Plan = serde_yaml::from_reader(input_file)?;
    plan.accounts.validate()?;

    if let Some(matches) = matches.subcommand_matches("forecast") {
        let years = value_t!(matches, "years", usize).unwrap_or(25);
        print_forecast(plan, years);
    }

    Ok(())
}

quick_main!(run);
