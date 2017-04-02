use std::collections::HashMap;
use std::iter::IntoIterator;
use std::fmt;
use std::cmp::Ordering;
use chrono::prelude::*;

use money::Money;
use errors::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Accounts {
    Tree(HashMap<String, Accounts>),
    Leaf(Account),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub amount: Money,
}

impl Accounts {
    // path has colons
    pub fn get(&self, path: &str) -> Option<&Accounts> {
        match self {
            &Accounts::Tree(ref m) => {
                if let Some(index) = path.find(':') {
                    let (account, sub_account) = path.split_at(index);
                    m.get(account).and_then(|a| a.get(&sub_account[1..]))
                } else {
                    m.get(path)
                }
            }
            a => if path.len() == 0 { Some(a) } else { None },
        }
    }

    // sum a tree of accounts
    pub fn sum(&self) -> Money {
        match *self {
            Accounts::Tree(ref m) => {
                let mut result = Money::from(0);
                for (_, account) in m {
                    result += account.sum();
                }
                result
            }
            Accounts::Leaf(ref a) => a.amount.clone(),
        }
    }

    pub fn fold<B, F>(self, init: B, mut f: F) -> B
        where F: FnMut(B, Account) -> B
    {
        self.fold_internal(init, &mut f)
    }

    fn fold_internal<B, F>(self, init: B, f: &mut F) -> B
        where F: FnMut(B, Account) -> B
    {
        match self {
            Accounts::Tree(m) => {
                let mut result = init;
                for (_, account) in m {
                    result = account.fold_internal(result, f);
                }
                result
            }
            Accounts::Leaf(l) => f(init, l),
        }
    }

    pub fn fold_with_path<B, F>(self, init: B, mut f: F) -> B
        where F: FnMut(B, &str, Account) -> B
    {
        self.fold_with_path_internal(init, "", &mut f)
    }

    fn fold_with_path_internal<B, F>(self, init: B, path: &str, f: &mut F) -> B
        where F: FnMut(B, &str, Account) -> B
    {
        match self {
            Accounts::Tree(m) => {
                let mut result = init;
                for (name, account) in m {
                    result = account.fold_with_path_internal(result,
                                                             &format!("{}{}",
                                                                      if path.len() == 0 {
                                                                          String::new()
                                                                      } else {
                                                                          format!("{}:", path)
                                                                      },
                                                                      name),
                                                             f);
                }
                result
            }
            Accounts::Leaf(l) => f(init, path, l),
        }
    }

    // TODO paths() (including intermediate nodes)
    pub fn paths(&self) -> Vec<String> {
        self.paths_internal("")
    }

    fn paths_internal(&self, path: &str) -> Vec<String> {
        match *self {
            Accounts::Tree(ref m) => {
                let mut result = Vec::new();
                for (name, account) in m {
                    result.extend(account.paths_internal(&format!("{}{}",
                                                                  if path.len() == 0 {
                                                                      String::new()
                                                                  } else {
                                                                      format!("{}:", path)
                                                                  },
                                                                  name)));
                }
                result
            }
            _ => vec![String::from(path)],
        }
    }

    // get a list of colon separated names
    pub fn get_account_names(&self) -> Vec<String> {
        self.clone()
            .fold_with_path(Vec::new(), |mut vec, path, _| {
                vec.push(String::from(path));
                vec
            })
    }

    // get all the leaf accounts
    pub fn flatten(self) -> Vec<Account> {
        self.fold(Vec::new(), |mut vec, account| {
            vec.push(account);
            vec
        })
    }

    pub fn flatten_with_path(self) -> Vec<(String, Account)> {
        self.fold_with_path(Vec::new(), |mut vec, path, account| {
            vec.push((String::from(path), account));
            vec
        })
    }

    pub fn create_account(self, path: String, account: Account) -> Result<Accounts> {
        match self {
            Accounts::Tree(mut m) => {
                if let Some(index) = path.find(':') {
                    let (path, sub_path) = path.split_at(index);
                    let sub_account = m.get(path)
                        .cloned()
                        .unwrap_or(Accounts::Tree(HashMap::new()))
                        .create_account(String::from(&sub_path[1..]), account)?;
                    m.insert(String::from(path), sub_account);
                    Ok(Accounts::Tree(m))
                } else if m.contains_key(&path) {
                    Err(ErrorKind::AlreadyExists(path).into())
                } else {
                    m.insert(path, Accounts::Leaf(account));
                    Ok(Accounts::Tree(m))
                }
            }
            other => {
                if path.len() == 0 {
                    Ok(other)
                } else {
                    Err(ErrorKind::InvalidAccountName(path).into())
                }
            }
        }
    }

    pub fn deposit(self, path: String, amount: Money) -> Result<Self> {
        match self {
            Accounts::Tree(mut m) => {
                if let Some(index) = path.find(':') {
                    let (path, sub_path) = path.split_at(index);
                    let new_subaccount = m.get(path)
                        .cloned()
                        .ok_or(ErrorKind::InvalidAccountName(String::from(path)).into())
                        .and_then(|sub_account| sub_account.deposit(String::from(&sub_path[1..]), amount))?;
                    m.insert(String::from(path), new_subaccount);
                    Ok(Accounts::Tree(m))
                } else if m.contains_key(&path) {
                    let new_subaccount = m.get(&path).cloned().unwrap().deposit(path.clone(), amount)?;
                    m.insert(String::from(path), new_subaccount);
                    Ok(Accounts::Tree(m))
                } else {
                    Accounts::Tree(m).create_account(path, Account { amount: amount })
                }
            },
            Accounts::Leaf(mut account) => {
                account.amount += amount;
                Ok(Accounts::Leaf(account))
            }
        }
    }

    pub fn withdraw(self, path: String, amount: Money) -> Result<Self> {
        self.deposit(path, -amount)
    }

    pub fn validate(&self) -> Result<()> {
        if let Accounts::Tree(ref m) = *self {
            for (path, account) in m {
                if let Some(_) = path.find(':') {
                    return Err(ErrorKind::InvalidAccountName(path.clone()).into())
                }
                account.validate()?;
            }
        }
        Ok(())
    }

    // TODO create_account_mut

    // TODO deposit_mut

    // TODO withdraw_mut

}

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

impl fmt::Display for DatedTransaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} => {}", self.date, self.transaction)
    }
}

// TODO write tests for the accounts object
