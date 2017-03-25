extern crate clap;
extern crate ramp;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use clap::{Arg, App, SubCommand};
use ramp::rational::Rational;
use std::fs::File;

#[derive(Debug, Serialize, Deserialize)]
struct IncomeStream {
    monthly: f64,
}

#[derive(Debug, Serialize, Deserialize)]
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
