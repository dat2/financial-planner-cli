use std::iter::{Iterator, FromIterator};
use std::collections::HashMap;
use chrono::prelude::*;
use chrono;

use money::Money;
use accounts::*;
use iterators::*;
use errors::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plan {
    pub accounts: Accounts,
    pub income: HashMap<String, Income>,
    pub rules: HashMap<String, Rule>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Income {
    pub amount: Money,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Rule {
    Transfer(Transfer),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transfer {
    pub amount: Money,
    pub from: String,
    pub to: String,
}

fn today() -> NaiveDate {
    let local = Local::now();
    NaiveDate::from_ymd(local.year(), local.month(), local.day())
}

impl Plan {
    fn initial_state(&self) -> Moment {
        let mut result = HashMap::new();

        // for (name, account) in &self.accounts {
        //     result.insert(name.clone(), account.amount.clone());
        // }

        Moment::new(result)
    }

    fn transactions(&self) -> SortedIterator<DatedTransaction, RepeatingTransaction> {
        let mut result = Vec::new();

        for (_, rule) in &self.rules {
            match *rule {
                Rule::Transfer(ref d) => {
                    // TODO don't unwrap
                    result.push(self.transfer_income(d.amount.clone(), d.from.clone(), d.to.clone())
                            .unwrap());
                }
            }
        }

        SortedIterator::from_iter(result.into_iter())
    }

    fn transfer_income(&self,
                       amount: Money,
                       from: String,
                       to: String)
                       -> Option<RepeatingTransaction> {
        match self.income.get(&from) {
            Some(ref income) => {
                if amount <= income.amount {
                    Some(RepeatingTransaction::new(income.frequency.clone(),
                                                   amount,
                                                   from,
                                                   to,
                                                   income.start_date.unwrap_or(today())))
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn history<D: Iterator<Item = NaiveDate>>
        (&self,
         dates: D)
         -> History<SortedIterator<DatedTransaction, RepeatingTransaction>, D> {
        History::new((today(), self.initial_state()), self.transactions(), dates)
    }
}

// stream stuff
#[derive(Clone)]
pub struct RepeatingTransaction {
    frequency: Frequency,
    transaction: Transaction,
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
            transaction: Transaction::new(amount, from, to),
            state: Some(state),
        }
    }
}

impl Iterator for RepeatingTransaction {
    type Item = DatedTransaction;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            Some(previous_date) => {
                let next_date = match self.frequency {
                    Frequency::Monthly => Some(previous_date + chrono::Duration::weeks(4)),
                    Frequency::BiWeekly => Some(previous_date + chrono::Duration::weeks(2)),
                    Frequency::Once => None,
                };

                self.state = next_date;
                Some(DatedTransaction::new(previous_date, self.transaction.clone()))
            }
            None => None,
        }
    }
}

pub struct YearStream {
    date: NaiveDate,
}

impl YearStream {
    pub fn new() -> YearStream {
        YearStream { date: today() }
    }
}

impl Iterator for YearStream {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        let previous_date = self.date;
        self.date = NaiveDate::from_ymd(previous_date.year() + 1,
                                        previous_date.month(),
                                        previous_date.day());
        Some(previous_date)
    }
}
