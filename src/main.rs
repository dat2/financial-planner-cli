#[macro_use]
extern crate clap;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate chrono;
extern crate prettytable;
extern crate rugflo;

mod money;
mod plan;
mod accounts;
mod iterators;

use std::fs::File;
use clap::{Arg, App, SubCommand};
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;

use plan::*;

fn print_forecast(plan: Plan, years: usize) {
    let mut table = Table::new();

    for (date, moment) in plan.history(YearStream::new().take(years)) {
        let mut result = Vec::new();

        result.push(Cell::new(&format!("{}", date)));

        for (name, value) in moment.accounts {
            result.push(Cell::new(&format!("{}: {}", name, value)));
        }

        table.add_row(Row::new(result));
    }

    table.printstd();
}

fn main() {

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

    let input_file = File::open(matches.value_of("INPUT").unwrap_or("input.yaml")).unwrap();
    let plan: Plan = serde_yaml::from_reader(input_file).unwrap();

    if let Some(matches) = matches.subcommand_matches("forecast") {
        let years = value_t!(matches, "years", usize).unwrap_or(25);
        print_forecast(plan, years);
    }
}
