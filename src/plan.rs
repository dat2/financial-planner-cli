use std::iter::Iterator;
use std::collections::HashMap;
use chrono::prelude::*;
use chrono;

use money::Money;

#[derive(Debug, Serialize, Deserialize)]
pub struct Plan {
    pub assets: HashMap<String, Asset>,
    pub liabilities: HashMap<String, Liability>,
    pub income: HashMap<String, IncomeSource>,
    pub rules: Option<HashMap<String, Rule>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    pub roi: f64,
    pub amount: Money,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Liability {
    Loan(LoanLiability),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoanLiability {
    pub amount: Money,
    pub interest: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IncomeSource {
    Monthly(MonthlyIncome),
    BiWeekly(BiWeeklyIncome),
    Once(OneTimeIncome),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlyIncome {
    pub amount: Money,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BiWeeklyIncome {
    pub amount: Money,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OneTimeIncome {
    pub amount: Money,
    pub date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Rule {
    Deposit(Deposit),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Deposit {
    pub amount: Money,
    pub from: String,
    pub to: String,
}

impl IncomeSource {
    pub fn amount(&self) -> Money {
        use IncomeSource::*;

        match *self {
            Monthly(ref m) => m.amount.clone(),
            BiWeekly(ref w) => w.amount.clone(),
            Once(ref o) => o.amount.clone(),
        }
    }

    pub fn start_date(&self) -> NaiveDate {
        use IncomeSource::*;

        let local_now = Local::now();
        let naive_today = NaiveDate::from_ymd(local_now.year(), local_now.month(), local_now.day());

        match *self {
            Monthly(ref m) => m.start_date.unwrap_or(naive_today),
            BiWeekly(ref w) => w.start_date.unwrap_or(naive_today),
            Once(ref o) => o.date,
        }
    }

    pub fn end_date(&self) -> Option<NaiveDate> {
        use IncomeSource::*;
        match *self {
            Monthly(ref m) => m.end_date,
            BiWeekly(ref w) => w.end_date,
            Once(ref o) => Some(o.date),
        }
    }

    pub fn step(&self) -> chrono::Duration {
        use IncomeSource::*;

        match *self {
            Monthly(_) => chrono::Duration::weeks(4),
            BiWeekly(_) => chrono::Duration::weeks(2),
            Once(_) => chrono::Duration::weeks(0),
        }
    }

    pub fn deposit_stream(&self, amount: Money) -> Option<IncomeStream> {
        if amount <= self.amount() {
            Some(IncomeStream::new(amount, self.start_date(), self.end_date(), self.step()))
        } else {
            None
        }
    }
}


#[derive(Debug, Clone)]
pub struct DateRange {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

impl DateRange {
    pub fn year(year: usize) -> DateRange {
        let start_date = NaiveDate::from_ymd(year as i32, 1, 1);
        let end_date = NaiveDate::from_ymd(year as i32, 12, 31);
        DateRange {
            start_date: start_date,
            end_date: end_date,
        }
    }

    pub fn next_year(&self) -> DateRange {
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

    pub fn sum(&self, income_stream: IncomeStream) -> Money {
        income_stream.skip_while(|&(date, _)| date < self.start_date)
            .take_while(|&(date, _)| date < self.end_date)
            .map(|(_, amount)| amount)
            .sum()
    }

    pub fn years() -> DateRangeStream {
        DateRangeStream::new(DateRange::year(Local::now().year() as usize),
                             DateRange::next_year)
    }
}

pub struct DateRangeStream {
    pub previous_date_range: DateRange,
    pub get_next: fn(&DateRange) -> DateRange,
}

impl DateRangeStream {
    pub fn new(date_range: DateRange, get_next: fn(&DateRange) -> DateRange) -> DateRangeStream {
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

pub struct IncomeStream {
    pub amount: Money,
    pub previous_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub step: chrono::Duration,
}

impl IncomeStream {
    pub fn new(amount: Money,
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
    type Item = (NaiveDate, Money);

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

            Some((copy, self.amount.clone()))
        } else {
            None
        }
    }
}

impl Liability {
    pub fn amount(&self) -> Money {
        use self::Liability::*;

        match *self {
            Loan(ref l) => l.amount.clone(),
        }
    }

    pub fn interest(&self) -> f64 {
        use self::Liability::*;

        match *self {
            Loan(ref l) => l.interest,
        }
    }
}
