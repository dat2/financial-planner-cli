#[macro_use]
extern crate clap;
extern crate num;
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

#[derive(Debug, Serialize, Deserialize)]
struct Plan {
    assets: HashMap<String, Asset>,
    liabilities: HashMap<String, Liability>,
    income: HashMap<String, IncomeSource>,
    rules: Option<HashMap<String, Rule>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Asset {
    roi: f64,
    amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
enum Liability {
    Loan(LoanLiability),
}

#[derive(Debug, Serialize, Deserialize)]
struct LoanLiability {
    amount: f64,
    interest: f64,
}

#[derive(Debug, Serialize, Deserialize)]
enum IncomeSource {
    Monthly(MonthlyIncome),
    BiWeekly(BiWeeklyIncome),
    Once(OneTimeIncome),
}

#[derive(Debug, Serialize, Deserialize)]
struct MonthlyIncome {
    amount: f64,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BiWeeklyIncome {
    amount: f64,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OneTimeIncome {
    amount: f64,
    date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize)]
enum Rule {
    Deposit(Deposit),
}

#[derive(Debug, Serialize, Deserialize)]
struct Deposit {
    amount: f64,
    from: String,
    to: String,
}

impl IncomeSource {
    fn amount(&self) -> f64 {
        use IncomeSource::*;

        match *self {
            Monthly(ref m) => m.amount,
            BiWeekly(ref w) => w.amount,
            Once(ref o) => o.amount,
        }
    }

    fn start_date(&self) -> NaiveDate {
        use IncomeSource::*;

        let local_now = Local::now();
        let naive_today = NaiveDate::from_ymd(local_now.year(), local_now.month(), local_now.day());

        match *self {
            Monthly(ref m) => m.start_date.unwrap_or(naive_today),
            BiWeekly(ref w) => w.start_date.unwrap_or(naive_today),
            Once(ref o) => o.date,
        }
    }

    fn end_date(&self) -> Option<NaiveDate> {
        use IncomeSource::*;
        match *self {
            Monthly(ref m) => m.end_date,
            BiWeekly(ref w) => w.end_date,
            Once(ref o) => Some(o.date),
        }
    }

    fn step(&self) -> chrono::Duration {
        use IncomeSource::*;

        match *self {
            Monthly(_) => chrono::Duration::weeks(4),
            BiWeekly(_) => chrono::Duration::weeks(2),
            Once(_) => chrono::Duration::weeks(0),
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


#[derive(Debug, Clone)]
struct DateRange {
    start_date: NaiveDate,
    end_date: NaiveDate,
}

impl DateRange {
    fn year(year: usize) -> DateRange {
        let start_date = NaiveDate::from_ymd(year as i32, 1, 1);
        let end_date = NaiveDate::from_ymd(year as i32, 12, 31);
        DateRange {
            start_date: start_date,
            end_date: end_date,
        }
    }

    fn next_year(&self) -> DateRange {
        let start_date = NaiveDate::from_ymd(self.start_date.year() + 1,
                                             self.start_date.month(),
                                             self.start_date.day());
        let end_date = NaiveDate::from_ymd(self.end_date.year() + 1,
                                           self.end_date.month(),
                                           self.end_date.day());
        DateRange {
            start_date: start_date,
            end_date: end_date,
        }
    }

    fn sum(&self, income_stream: IncomeStream) -> f64 {
        income_stream.skip_while(|&(date, _)| date < self.start_date)
            .take_while(|&(date, _)| date < self.end_date)
            .map(|(_, amount)| amount)
            .sum()
    }

    fn years() -> DateRangeStream {
        DateRangeStream::new(DateRange::year(Local::now().year() as usize),
                             DateRange::next_year)
    }
}

struct DateRangeStream {
    previous_date_range: DateRange,
    get_next: fn(&DateRange) -> DateRange
}

impl DateRangeStream {
    fn new(date_range: DateRange, get_next: fn(&DateRange) -> DateRange) -> DateRangeStream {
        DateRangeStream {
            previous_date_range: date_range,
            get_next: get_next,
        }
    }
}

impl Iterator for DateRangeStream {
    type Item = DateRange;

    fn next(&mut self) -> Option<Self::Item> {
        let date_range = self.previous_date_range.clone();
        self.previous_date_range = (self.get_next)(&self.previous_date_range);
        Some(date_range)
    }
}

struct IncomeStream {
    amount: f64,
    previous_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    step: chrono::Duration,
}

impl IncomeStream {
    fn new(amount: f64,
           start_date: NaiveDate,
           end_date: Option<NaiveDate>,
           step: chrono::Duration)
           -> IncomeStream {
        IncomeStream {
            amount: amount,
            previous_date: Some(start_date),
            end_date: end_date,
            step: step,
        }
    }
}

impl Iterator for IncomeStream {
    type Item = (NaiveDate, f64);

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

impl Liability {
    fn amount(&self) -> f64 {
        use Liability::*;

        match *self {
            Loan(ref l) => l.amount
        }
    }

    fn interest(&self) -> f64 {
        use Liability::*;

        match *self {
            Loan(ref l) => l.interest
        }
    }
}

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

        let mut net = 0.0_f64;

        // add assets
        for (name, asset) in &plan.assets {
            let mut previous_year_value = previous_year_values.entry(name).or_insert(asset.amount);
            year_row.push(Cell::new(&format!("${:.2}", previous_year_value)));
            net += *previous_year_value;

            // add return
            *previous_year_value *= 1.0 + asset.roi;

            // use rules
            if let Some(ref rules) = plan.rules {
                for (_, rule) in rules {
                    match *rule {
                        Rule::Deposit(ref d) if d.to == *name => {
                            *previous_year_value += range.sum(plan.income
                                .get(&d.from)
                                .unwrap()
                                .deposit_stream(d.amount)
                                .unwrap());
                        }
                        _ => {}
                    }
                }
            }
        }

        // add liabilities
        for (name, liability) in &plan.liabilities {
            let mut previous_year_value = previous_year_values.entry(name).or_insert(liability.amount());
            year_row.push(Cell::new(&format!("-${:.2}", previous_year_value)));
            net -= *previous_year_value;

            // add interest
            *previous_year_value *= 1.0 + liability.interest();

            // TODO use liability rules
        }

        year_row.push(Cell::new(&format!("${:.2}", net)));

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
