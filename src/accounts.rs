use std::collections::HashMap;
use std::iter::IntoIterator;
use std::fmt;
use std::cmp::Ordering;
use chrono::prelude::*;

use money::Money;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub amount: Money,
    pub from: String,
    pub to: String,
}

impl Transaction {
    pub fn new(amount: Money, from: String, to: String) -> Transaction {
        Transaction {
            amount: amount,
            from: from,
            to: to,
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} from {} to {}", self.amount, self.from, self.to)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Moment {
    pub accounts: HashMap<String, Money>,
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

pub struct History<T: Iterator<Item = DatedTransaction> + Clone, D: Iterator<Item = NaiveDate>> {
    transactions: T,
    dates: D,
    consumed: usize,
    state: (NaiveDate, Moment),
}

impl<T, D> History<T, D>
    where T: Iterator<Item = DatedTransaction> + Clone,
          D: Iterator<Item = NaiveDate>
{
    pub fn new(state: (NaiveDate, Moment), transactions: T, dates: D) -> History<T, D> {
        History {
            transactions: transactions,
            dates: dates,
            consumed: 0,
            state: state,
        }
    }
}

impl<T, D> Iterator for History<T, D>
    where T: Iterator<Item = DatedTransaction> + Clone,
          D: Iterator<Item = NaiveDate>
{
    type Item = (NaiveDate, Moment);

    fn next(&mut self) -> Option<Self::Item> {
        match self.dates.next() {
            Some(next_date) => {
                // consume the next few transactions
                let next_transactions = self.transactions
                    .clone()
                    .skip(self.consumed)
                    .take_while(|ref dt| dt.date <= next_date)
                    .map(|dt| dt.transaction)
                    .collect::<Vec<_>>();
                self.consumed += next_transactions.len();

                // calculate next state
                self.state = (next_date,
                              next_transactions.into_iter()
                    .fold(self.state.1.clone(), Moment::push_transaction));
                Some(self.state.clone())
            }
            None => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatedTransaction {
    date: NaiveDate,
    transaction: Transaction,
}

impl DatedTransaction {
    pub fn new(date: NaiveDate, transaction: Transaction) -> DatedTransaction {
        DatedTransaction {
            date: date,
            transaction: transaction,
        }
    }
}

impl PartialOrd for DatedTransaction {
    fn partial_cmp(&self, other: &DatedTransaction) -> Option<Ordering> {
        self.date.partial_cmp(&other.date)
    }
}

impl Eq for DatedTransaction {}

impl Ord for DatedTransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.date.cmp(&other.date)
    }
}
