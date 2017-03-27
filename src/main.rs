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

use money::*;
use plan::*;

use std::fs::File;
use std::collections::HashMap;
use chrono::prelude::*;
use clap::{Arg, App, SubCommand};
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;

fn print_forecast(plan: Plan, years: usize) {
    let mut table = Table::new();

    let mut header_row = Vec::new();
    header_row.push(Cell::new("Year"));
    for (name, _) in &plan.assets {
        header_row.push(Cell::new(&format!("Asset:{}", name)));
    }
    for (name, _) in &plan.liabilities {
        header_row.push(Cell::new(&format!("Liability:{}", name)));
    }
    header_row.push(Cell::new("Net"));
    table.add_row(Row::new(header_row));

    let mut previous_year_values = HashMap::new();

    for range in DateRange::years().take(years + 1) {

        let mut year_row = Vec::new();
        year_row.push(Cell::new(&range.end_date.year().to_string()));

        let mut net = Money::from(0);

        // add assets
        for (name, asset) in &plan.assets {
            let mut previous_year_value = previous_year_values.entry(name).or_insert(asset.amount.clone());
            year_row.push(Cell::new(&format!("{:.2}", previous_year_value)));
            net += previous_year_value.clone();

            // add return
            previous_year_value.mul_percentage(1.0_f64 + asset.roi);

            // use rules
            if let Some(ref rules) = plan.rules {
                for (_, rule) in rules {
                    match *rule {
                        Rule::Deposit(ref d) if d.to == *name => {
                            *previous_year_value += range.sum(plan.income
                                .get(&d.from)
                                .unwrap()
                                .deposit_stream(d.amount.clone())
                                .unwrap());
                        }
                        _ => {}
                    }
                }
            }
        }

        // add liabilities
        for (name, liability) in &plan.liabilities {
            let mut previous_year_value = previous_year_values.entry(name)
                .or_insert(liability.amount());
            year_row.push(Cell::new(&format!("-{:.2}", previous_year_value)));
            net -= previous_year_value.clone();

            // add interest
            previous_year_value.mul_percentage(1.0_f64 + liability.interest());

            // TODO use liability rules
        }

        year_row.push(Cell::new(&format!("{:.2}", net)));

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
