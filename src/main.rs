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
    amount: f64,
    end_date: Option<DateTime<Local>>
}

#[derive(Debug, Deserialize)]
struct BiWeekly {
    amount: f64,
    end_date: Option<DateTime<Local>>
}

#[derive(Debug, Deserialize)]
struct OneTime {
    amount: f64,
    date: DateTime<Local>
}

#[derive(Debug, Deserialize)]
enum IncomeSource {
    Monthly(Monthly),
    BiWeekly(BiWeekly),
    Once(OneTime)
}

#[derive(Debug, Deserialize)]
struct Investment {
    roi: f64,
    amount: f64
}

#[derive(Debug, Deserialize)]
struct Deposit {
    amount: f64,
    from: String,
    to: String
}

#[derive(Debug, Deserialize)]
enum Rule {
    Deposit(Deposit)
}

#[derive(Debug, Deserialize)]
struct Plan {
    income: HashMap<String, IncomeSource>,
    investments: HashMap<String, Investment>,
    rules: Option<HashMap<String, Rule>>
}

// TODO complain if deposit_amount > income
// TODO care about the year / end date
fn calculate_yearly_deposit_amount(income: &IncomeSource, deposit_amount: f64) -> f64 {
    match *income {
        IncomeSource::Monthly(_) => {
            deposit_amount * 12.0
        },
        IncomeSource::BiWeekly(_) => {
            deposit_amount * 26.0
        },
        IncomeSource::Once(_) => {
            deposit_amount
        }
    }
}

fn print_forecast(input: Plan, years: u32) {
    let mut table = Table::new();

    let mut header_row = Vec::new();
    header_row.push(Cell::new("Year"));
    for (name, _) in &input.investments {
        header_row.push(Cell::new(name));
    }
    table.add_row(Row::new(header_row));

    let mut previous_year_values = HashMap::new();

    for year in 0..(years + 1) {

        let mut year_row = Vec::new();
        year_row.push(Cell::new(&year.to_string()));
        for (name, investment) in &input.investments {
            let mut previous_year_value = previous_year_values.entry(name).or_insert(investment.amount);
            year_row.push(Cell::new(&format!("${:.2}", previous_year_value)));

            // add return
            *previous_year_value *= 1.0 + investment.roi;

            // add rules
            if let Some(ref rules) = input.rules {
                for (_, rule) in rules {
                    match *rule {
                        Rule::Deposit(ref d) if d.to == *name => {
                            *previous_year_value += calculate_yearly_deposit_amount(input.income.get(&d.from).unwrap(), d.amount);
                        },
                        _ => { }
                    }
                }
            }
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
    let input: Plan = serde_yaml::from_reader(input_file).unwrap();

    if let Some(matches) = matches.subcommand_matches("forecast") {
        let years = value_t!(matches, "years", u32).unwrap_or(25);
        print_forecast(input, years);
    }
}
