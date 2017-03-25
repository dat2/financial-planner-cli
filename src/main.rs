#[macro_use]
extern crate clap;
extern crate ramp;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate chrono;
extern crate prettytable;

use std::fs::File;
use chrono::prelude::*;
use clap::{Arg, App, SubCommand};
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct Monthly {
    name: String,
    amount: f64,
    end_date: Option<DateTime<Local>>
}

#[derive(Debug, Deserialize)]
struct BiWeekly {
    name: String,
    amount: f64,
    end_date: Option<DateTime<Local>>
}

#[derive(Debug, Deserialize)]
struct OneTime {
    name: String,
    amount: f64,
    date: DateTime<Local>
}

#[derive(Debug, Deserialize)]
enum IncomeStream {
    Monthly(Monthly),
    BiWeekly(BiWeekly),
    OneTime(OneTime)
}

#[derive(Debug, Deserialize)]
struct Investment {
    name: String,
    roi: f64,
    amount: f64
}

#[derive(Debug, Deserialize)]
struct Input {
    income: Vec<IncomeStream>,
    investments: Vec<Investment>
}

fn print_forecast(input: Input, years: u32) {
    let mut table = Table::new();

    let mut header_row = Vec::new();
    header_row.push(Cell::new("Year"));
    for investment in &input.investments {
        header_row.push(Cell::new(&investment.name));
    }
    table.add_row(Row::new(header_row));

    let mut previous_year_values = HashMap::new();

    for year in 0..(years + 1) {

        let mut year_row = Vec::new();
        year_row.push(Cell::new(&year.to_string()));
        for investment in &input.investments {
            let mut previous_year_value = previous_year_values.entry(&investment.name).or_insert(investment.amount);
            year_row.push(Cell::new(&format!("${:.2}", previous_year_value)));
            *previous_year_value *= 1.0 + investment.roi;
        }
        table.add_row(Row::new(year_row));

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
            .about("Calculate investment values over <n> years.")
            .arg(Arg::with_name("years")
                .help("Sets the number of years to calculate forward.")
                .index(1)))
        .get_matches();

    let input_file = File::open(matches.value_of("INPUT").unwrap_or("input.yaml")).unwrap();
    let input: Input = serde_yaml::from_reader(input_file).unwrap();

    if let Some(matches) = matches.subcommand_matches("forecast") {
        let years = value_t!(matches, "years", u32).unwrap_or(25);
        print_forecast(input, years);
    }
}
