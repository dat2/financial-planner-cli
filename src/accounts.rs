use std::collections::HashMap;
use std::iter::IntoIterator;
use chrono::prelude::*;

use money::Money;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub amount: Money,
    pub from: String,
    pub to: String
}

impl Transaction {
    pub fn new(amount: Money, from: String, to: String) -> Transaction {
        Transaction {
            amount: amount,
            from: from,
            to: to
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Moment {
    pub accounts: HashMap<String, Money>
}

impl Moment {
    pub fn new(accounts: HashMap<String, Money>) -> Moment {
        Moment { accounts: accounts }
    }

    pub fn push_transaction(self, transaction: Transaction) -> Self {
        let mut next = self.accounts.clone();

        {
            let mut from = next.entry(transaction.from).or_insert(Money::from(0));
            *from -= transaction.amount.clone();
        }

        {
            let mut to = next.entry(transaction.to).or_insert(Money::from(0));
            *to += transaction.amount.clone();
        }

        Moment::new(next)
    }
}

pub struct History<T: Iterator<Item = (NaiveDate, Transaction)> + Clone, D: Iterator<Item = NaiveDate>> {
    transactions: T,
    dates: D,
    consumed: usize,
    state: (NaiveDate, Moment)
}

impl<T, D> History<T, D>
    where T: Iterator<Item = (NaiveDate, Transaction)> + Clone,
          D: Iterator<Item = NaiveDate>
{
    pub fn new(state: (NaiveDate, Moment), dates: D, transactions: T) -> History<T, D> {
        History {
            transactions: transactions,
            dates: dates,
            consumed: 0,
            state: state
        }
    }
}

impl<T, D> Iterator for History<T, D>
    where T: Iterator<Item = (NaiveDate, Transaction)> + Clone,
          D: Iterator<Item = NaiveDate>
{
    type Item = (NaiveDate, Moment);

    fn next(&mut self) -> Option<Self::Item> {
        match self.dates.next() {
            Some(next_date) => {
                let next_transactions = self.transactions
                    .clone()
                    .skip(self.consumed)
                    .take_while(|&(d,_)| d <= next_date)
                    .map(|(_, t)| t)
                    .collect::<Vec<_>>();

                self.consumed = next_transactions.len();

                let previous_moment = self.state.1.clone();
                self.state = (next_date, next_transactions.into_iter().fold(previous_moment.clone(), Moment::push_transaction));

                Some((next_date, previous_moment))
            },
            None => None
        }
    }
}
