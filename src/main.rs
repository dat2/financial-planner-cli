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
use std::iter::Iterator;
use std::collections::HashMap;
use chrono::prelude::*;
use clap::{Arg, App, SubCommand};
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;

#[derive(Debug, Deserialize)]
struct Plan {
    income: HashMap<String, IncomeSource>,
    investments: HashMap<String, Investment>,
    rules: Option<HashMap<String, Rule>>
}

#[derive(Debug, Deserialize)]
enum IncomeSource {
    Monthly(MonthlyIncome),
    BiWeekly(BiWeeklyIncome),
    Once(OneTimeIncome)
}

#[derive(Debug, Deserialize)]
struct MonthlyIncome {
    amount: f64,
    start_date: Option<DateTime<UTC>>,
    end_date: Option<DateTime<UTC>>
}

#[derive(Debug, Deserialize)]
struct BiWeeklyIncome {
    amount: f64,
    start_date: Option<DateTime<UTC>>,
    end_date: Option<DateTime<UTC>>
}

#[derive(Debug, Deserialize)]
struct OneTimeIncome {
    amount: f64,
    date: DateTime<UTC>
}

#[derive(Debug, Deserialize)]
struct Investment {
    roi: f64,
    amount: f64
}

#[derive(Debug, Deserialize)]
enum Rule {
    Deposit(Deposit)
}

#[derive(Debug, Deserialize)]
struct Deposit {
    amount: f64,
    from: String,
    to: String
}


impl IncomeSource {
    fn amount(&self) -> f64 {
        use IncomeSource::*;

        match *self {
            Monthly(ref m) => m.amount,
            BiWeekly(ref w) => w.amount,
            Once(ref o) => o.amount
        }
    }

    fn start_date(&self) -> DateTime<UTC> {
        use IncomeSource::*;
        match *self {
            Monthly(ref m) => m.start_date.unwrap_or(UTC::now()),
            BiWeekly(ref w) => w.start_date.unwrap_or(UTC::now()),
            Once(ref o) => o.date
        }
    }

    fn end_date(&self) -> Option<DateTime<UTC>> {
        use IncomeSource::*;
        match *self {
            Monthly(ref m) => m.end_date,
            BiWeekly(ref w) => w.end_date,
            Once(ref o) => Some(o.date)
        }
    }

    fn step(&self) -> chrono::Duration {
        use IncomeSource::*;

        match *self {
            Monthly(_) => chrono::Duration::weeks(4),
            BiWeekly(_) => chrono::Duration::weeks(2),
            Once(_) => chrono::Duration::weeks(0)
        }
    }

    fn deposit_stream(&self, amount: f64) -> Option<IncomeStream> {
        if amount <= self.amount() {
            Some(IncomeStream::new(amount, self.start_date(), self.end_date(), self.step()))
        } else {
            None
        }
    }
}

struct IncomeStream {
    amount: f64,
    previous_date: Option<DateTime<UTC>>,
    end_date: Option<DateTime<UTC>>,
    step: chrono::Duration,
}

impl IncomeStream {
    fn new(amount: f64, start_date: DateTime<UTC>, end_date: Option<DateTime<UTC>>, step: chrono::Duration) -> IncomeStream {
        IncomeStream {
            amount: amount,
            previous_date: Some(start_date),
            end_date: end_date,
            step: step
        }
    }
}

impl Iterator for IncomeStream {
    type Item = (DateTime<UTC>, f64);

    fn next(&mut self) -> Option<Self::Item> {

        if let Some(previous) = self.previous_date {
            let copy = previous.clone();

            self.previous_date = Some(previous + self.step);

            if let Some(next_date) = self.previous_date {
                if let Some(end_date) = self.end_date {
                    if next_date >= end_date {
                        self.previous_date = None;
                    }
                }
            }

            Some((copy, self.amount))
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct DateRange {
    start_date: DateTime<UTC>,
    end_date: DateTime<UTC>
}

impl DateRange {

    fn year(year: i32) -> DateRange {
        let start_date = UTC.ymd(year, 1, 1).and_hms(0, 0, 0);
        let end_date = UTC.ymd(year, 12, 31).and_hms(0, 0, 0);
        DateRange { start_date: start_date, end_date: end_date }
    }

    fn sum(&self, income_stream: IncomeStream) -> f64 {
        income_stream
            .skip_while(|&(date,_)| date < self.start_date)
            .take_while(|&(date,_)| date < self.end_date)
            .map(|(_,amount)| amount)
            .sum()
    }

}

fn print_forecast(plan: Plan, years: i32) {
    let mut table = Table::new();

    let mut header_row = Vec::new();
    header_row.push(Cell::new("Year"));
    for (name, _) in &plan.investments {
        header_row.push(Cell::new(name));
    }
    table.add_row(Row::new(header_row));

    let mut previous_year_values = HashMap::new();

    for year in 0..(years + 1) {

        let range = DateRange::year(UTC::now().year() + year);

        let mut year_row = Vec::new();
        year_row.push(Cell::new(&year.to_string()));
        for (name, investment) in &plan.investments {
            let mut previous_year_value = previous_year_values.entry(name).or_insert(investment.amount);
            year_row.push(Cell::new(&format!("${:.2}", previous_year_value)));

            // add return
            *previous_year_value *= 1.0 + investment.roi;

            // add rules
            if let Some(ref rules) = plan.rules {
                for (_, rule) in rules {
                    match *rule {
                        Rule::Deposit(ref d) if d.to == *name => {
                            *previous_year_value += range.sum(plan.income.get(&d.from).unwrap().deposit_stream(d.amount).unwrap());
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
    let plan: Plan = serde_yaml::from_reader(input_file).unwrap();

    if let Some(matches) = matches.subcommand_matches("forecast") {
        let years = value_t!(matches, "years", i32).unwrap_or(25);
        print_forecast(plan, years);
    }
}
