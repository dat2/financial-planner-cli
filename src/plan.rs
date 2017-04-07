use std::iter::{Iterator, FromIterator};
use std::collections::HashMap;
use chrono::prelude::*;
use chrono;

use money::Money;
use accounts::*;
use iterators::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plan {
    pub accounts: Accounts,
    pub rules: HashMap<String, Rule>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Rule {
    RepeatingMoney(MoneyTransfer),
    CompoundingInterest(CompoundingInterest),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoneyTransfer {
    pub amount: Money,
    pub from: String,
    pub to: String,
    pub frequency: Frequency,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompoundingInterest {
    pub account: String,
    pub interest_rate: f64,
    pub period: Frequency,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Frequency {
    Annually,
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
        let mut iters = Vec::new();

        for rule in self.rules.values() {
            if let Rule::RepeatingMoney(ref t) = *rule {
                iters.push(RepeatingTransaction::from(t.clone()));
            }
        }

        SortedIterator::from_iter(iters.into_iter())
    }

    fn compounding(&self) -> SortedIterator<CompoundedInterest, InterestStream> {
        let mut iters = Vec::new();

        for rule in self.rules.values() {
            if let Rule::CompoundingInterest(ref c) = *rule {
                iters.push(InterestStream::from(c.clone()));
            }
        }

        SortedIterator::from_iter(iters.into_iter())
    }

    pub fn history<D: Iterator<Item = NaiveDate>>(&self,
                                                  dates: D)
                                                  -> History<SortedIterator<Transaction,
                                                                            RepeatingTransaction>,
                                                             SortedIterator<CompoundedInterest,
                                                                            InterestStream>,
                                                             D> {
        History::new((today(), self.accounts.clone()),
                     self.transactions(),
                     self.compounding(),
                     dates)
    }
}

// stream stuff
pub struct RepeatingTransaction {
    iterator: DateStream,
    amount: Amount,
    from: String,
    to: String,
}

impl RepeatingTransaction {
    fn new<T: Into<Amount>>(iterator: DateStream,
                            amount: T,
                            from: String,
                            to: String)
                            -> RepeatingTransaction {
        RepeatingTransaction {
            iterator: iterator,
            amount: amount.into(),
            from: from,
            to: to,
        }
    }
}

impl From<MoneyTransfer> for RepeatingTransaction {
    fn from(transfer: MoneyTransfer) -> RepeatingTransaction {
        RepeatingTransaction::new(DateStream::from((transfer.frequency, transfer.start_date)),
                                  transfer.amount,
                                  transfer.from,
                                  transfer.to)
    }
}

impl Iterator for RepeatingTransaction {
    type Item = Transaction;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iterator.next() {
            Some(next_date) => {
                Some(Transaction::new(Amount::from(self.amount.clone()),
                                      self.from.clone(),
                                      self.to.clone(),
                                      next_date))
            }
            None => None,
        }
    }
}

pub struct InterestStream {
    iterator: DateStream,
    interest_rate: f64,
    account: String,
}

impl InterestStream {
    fn new(iterator: DateStream, interest_rate: f64, account: String) -> InterestStream {
        InterestStream {
            iterator: iterator,
            interest_rate: interest_rate,
            account: account,
        }
    }
}

fn interest_per_period(interest: f64, period: &Frequency) -> f64 {
    match *period {
        Frequency::Annually => interest,
        Frequency::Monthly => interest / 12.0,
        Frequency::BiWeekly => interest / 26.0,
        Frequency::Once => interest,
    }
}

impl From<CompoundingInterest> for InterestStream {
    fn from(rule: CompoundingInterest) -> InterestStream {
        let interest_rate = interest_per_period(rule.interest_rate, &rule.period);
        InterestStream::new(DateStream::from((rule.period, rule.start_date)),
                            interest_rate,
                            rule.account)
    }
}

impl Iterator for InterestStream {
    type Item = CompoundedInterest;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iterator.next() {
            Some(next_date) => {
                Some(CompoundedInterest::new(next_date, self.interest_rate, self.account.clone()))
            }
            None => None,
        }
    }
}

// date streams are fun yay
pub struct DateStream {
    date: Option<NaiveDate>,
    func: fn(NaiveDate) -> Option<NaiveDate>,
}

impl DateStream {
    fn new(date: Option<NaiveDate>, func: fn(NaiveDate) -> Option<NaiveDate>) -> DateStream {
        DateStream {
            date: if date.is_none() { Some(today()) } else { date },
            func: func,
        }
    }

    pub fn yearly(date: Option<NaiveDate>) -> DateStream {
        DateStream::new(date, next_year)
    }

    pub fn monthly(date: Option<NaiveDate>) -> DateStream {
        DateStream::new(date, next_month)
    }

    pub fn biweekly(date: Option<NaiveDate>) -> DateStream {
        DateStream::new(date, next_biweek)
    }

    pub fn once(date: Option<NaiveDate>) -> DateStream {
        DateStream::new(date, once)
    }
}

impl From<(Frequency, Option<NaiveDate>)> for DateStream {
    fn from(val: (Frequency, Option<NaiveDate>)) -> DateStream {
        match val.0 {
            Frequency::Annually => DateStream::yearly(val.1),
            Frequency::Monthly => DateStream::monthly(val.1),
            Frequency::BiWeekly => DateStream::biweekly(val.1),
            Frequency::Once => DateStream::once(val.1),
        }
    }
}

impl Iterator for DateStream {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        let previous_date = self.date;
        self.date = previous_date.and_then(self.func);
        previous_date
    }
}

fn next_year(previous_date: NaiveDate) -> Option<NaiveDate> {
    Some(NaiveDate::from_ymd(previous_date.year() + 1,
                             previous_date.month(),
                             previous_date.day()))
}

fn next_month(previous_date: NaiveDate) -> Option<NaiveDate> {
    Some(previous_date + chrono::Duration::weeks(4))
}

fn next_biweek(previous_date: NaiveDate) -> Option<NaiveDate> {
    Some(previous_date + chrono::Duration::weeks(4))
}

fn once(_: NaiveDate) -> Option<NaiveDate> {
    None
}
