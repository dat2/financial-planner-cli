use std::iter::{Iterator, FromIterator};
use std::collections::HashMap;
use chrono::prelude::*;
use chrono;

use money::Money;
use accounts::*;
use iterators::*;
use expression::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plan {
    pub accounts: Accounts,
    pub rules: HashMap<String, Rule>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Rule {
    Repeating(Transfer),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transfer {
    pub amount: Money,
    pub from: String,
    pub to: String,
    pub frequency: Frequency,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Frequency {
    Monthly,
    BiWeekly,
    Once,
}

fn today() -> NaiveDate {
    let local = Local::now();
    NaiveDate::from_ymd(local.year(), local.month(), local.day())
}

impl Plan {
    fn transactions(&self) -> SortedIterator<Transaction, RepeatingTransaction> {
        let mut result = Vec::new();

        for (_, rule) in &self.rules {
            match *rule {
                Rule::Repeating(ref t) => {
                    result.push(self.repeating_transfer(t.clone()));
                }
            }
        }

        SortedIterator::from_iter(result.into_iter())
    }

    fn repeating_transfer(&self, transfer: Transfer) -> RepeatingTransaction {
        RepeatingTransaction::new(transfer.frequency,
                                  transfer.amount,
                                  transfer.from,
                                  transfer.to,
                                  transfer.start_date.unwrap_or(today()))
    }

    pub fn history<D: Iterator<Item = NaiveDate>>
        (&self,
         dates: D)
         -> History<SortedIterator<Transaction, RepeatingTransaction>, D> {
        History::new((today(), self.accounts.clone()), self.transactions(), dates)
    }
}

// stream stuff
#[derive(Clone)]
pub struct RepeatingTransaction {
    frequency: Frequency,
    amount: Money,
    from: String,
    to: String,
    state: Option<NaiveDate>,
}

impl RepeatingTransaction {
    fn new(frequency: Frequency,
           amount: Money,
           from: String,
           to: String,
           state: NaiveDate)
           -> RepeatingTransaction {
        RepeatingTransaction {
            frequency: frequency,
            amount: amount,
            from: from,
            to: to,
            state: Some(state),
        }
    }
}

impl Iterator for RepeatingTransaction {
    type Item = Transaction;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            Some(previous_date) => {
                let next_date = match self.frequency {
                    Frequency::Monthly => Some(previous_date + chrono::Duration::weeks(4)),
                    Frequency::BiWeekly => Some(previous_date + chrono::Duration::weeks(2)),
                    Frequency::Once => None,
                };

                self.state = next_date;
                Some(Transaction::new(self.amount.clone(), self.from.clone(), self.to.clone(), previous_date))
            }
            None => None,
        }
    }
}

pub struct DateStream {
    date: NaiveDate,
    func: fn(NaiveDate) -> NaiveDate
}

impl DateStream {
    fn new(func: fn(NaiveDate) -> NaiveDate) -> DateStream {
        DateStream { date: today(), func: func }
    }

    fn next_year(previous_date: NaiveDate) -> NaiveDate {
        NaiveDate::from_ymd(previous_date.year() + 1,
                                        previous_date.month(),
                                        previous_date.day())
    }

    pub fn years() -> DateStream {
        DateStream::new(DateStream::next_year)
    }
}

impl Iterator for DateStream {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        let previous_date = self.date;
        self.date = (self.func)(previous_date);
        Some(previous_date)
    }
}
