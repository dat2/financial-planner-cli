extern crate clap;
extern crate ramp;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate chrono;

use std::fs::File;
use chrono::prelude::*;
use ramp::rational::Rational;
use clap::{Arg, App, SubCommand};

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
enum IncomeStream {
    Monthly(Monthly),
    BiWeekly(BiWeekly),
    OneTime(OneTime)
}

#[derive(Debug, Deserialize)]
struct Input {
    income: Vec<IncomeStream>,
}

fn main() {
    let matches = App::new("financial-planner")
        .version("0.1")
        .author("Nicholas D. <nickdujay@gmail.com>")
        .about("Helps you plan your financial future.")
        .arg(Arg::with_name("INPUT")
            .help("Sets the input file to use.")
            .required(true)
            .index(1))
        .get_matches();

    let input_file = File::open(matches.value_of("INPUT").unwrap()).unwrap();
    let input: Input = serde_yaml::from_reader(input_file).unwrap();

    println!("{:?}", input);
}
