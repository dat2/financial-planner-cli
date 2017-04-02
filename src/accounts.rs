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
#[serde(untagged)]
pub enum Account {
    Simple(SimpleAccount), // TODO derivedaccount
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimpleAccount {
    pub amount: Money,
}

impl Account {
    pub fn amount(&self) -> Money {
        match *self {
            Account::Simple(ref s) => s.amount.clone(),
        }
    }

    pub fn add(&mut self, amount: Money) {
        match *self {
            Account::Simple(ref mut s) => {
                s.amount += amount;
            }
        }
    }
}

impl Accounts {

    pub fn root() -> Accounts {
        Accounts::Tree(HashMap::new())
    }

    pub fn leaf(account: Account) -> Accounts {
        Accounts::Leaf(account)
    }

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

    // TODO get_mut

    pub fn sum(&self) -> Money {
        match *self {
            Accounts::Tree(ref m) => {
                let mut result = Money::from(0);
                for (_, account) in m {
                    result += account.sum();
                }
                result
            }
            Accounts::Leaf(ref a) => a.amount(),
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

    pub fn get_account_names(&self) -> Vec<String> {
        self.clone()
            .fold_with_path(Vec::new(), |mut vec, path, _| {
                vec.push(String::from(path));
                vec
            })
    }

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
                        .unwrap_or_else(Accounts::root)
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

    // TODO create_account_unchecked
    // TODO create_account_mut
    // TODO create_account_mut_unchecked

    pub fn deposit(self, path: String, amount: Money) -> Result<Self> {
        match self {
            Accounts::Tree(mut m) => {
                if let Some(index) = path.find(':') {
                    let (path, sub_path) = path.split_at(index);
                    let new_subaccount = m.get(path).cloned()
                        .unwrap_or_else(Accounts::root)
                        .deposit(String::from(&sub_path[1..]), amount)?;
                    m.insert(String::from(path), new_subaccount);
                    Ok(Accounts::Tree(m))
                } else if m.contains_key(&path) {
                    let new_subaccount =
                        m.get(&path).cloned().unwrap().deposit(path.clone(), amount)?;
                    m.insert(String::from(path), new_subaccount);
                    Ok(Accounts::Tree(m))
                } else {
                    Accounts::Tree(m)
                        .create_account(path, Account::Simple(SimpleAccount { amount: amount }))
                }
            }
            Accounts::Leaf(mut account) => {
                account.add(amount);
                Ok(Accounts::Leaf(account))
            }
        }
    }

    // TODO deposit_unchecked
    // TODO deposit_mut
    // TODO deposit_mut_unchecked

    pub fn withdraw(self, path: String, amount: Money) -> Result<Self> {
        self.deposit(path, -amount)
    }

    // TODO withdraw_unchecked

    // TODO withdraw_mut
    // TODO withdraw_mut_unchecked

    pub fn validate(&self) -> Result<()> {
        if let Accounts::Tree(ref m) = *self {
            for (path, account) in m {
                if let Some(_) = path.find(':') {
                    return Err(ErrorKind::InvalidAccountName(path.clone()).into());
                }
                account.validate()?;
            }
        }
        Ok(())
    }

    pub fn apply(self, transaction: Transaction) -> Result<Self> {
        self.withdraw(transaction.from, transaction.amount.clone())?
            .deposit(transaction.to, transaction.amount)
    }

    pub fn apply_unchecked(self, transaction: Transaction) -> Self {
        self.withdraw(transaction.from, transaction.amount.clone()).unwrap()
            .deposit(transaction.to, transaction.amount).unwrap()
    }

    // TODO apply_mut
    // TODO apply_mut_unchecked
}

// TODO impl IntoIterator for Accounts
// TODO impl std::iter::Extend for Accounts

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub amount: Money,
    pub from: String,
    pub to: String,
    pub date: NaiveDate
}

impl Transaction {
    pub fn new(amount: Money, from: String, to: String, date: NaiveDate) -> Transaction {
        Transaction {
            amount: amount,
            from: from,
            to: to,
            date: date
        }
    }
}

impl Eq for Transaction {}

impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Transaction) -> Option<Ordering> {
        self.date.partial_cmp(&other.date)
    }
}

impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.date.cmp(&other.date)
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} from {} to {}", self.amount, self.from, self.to)
    }
}

// TODO wrap transaction iterators

pub struct History<T: Iterator<Item = Transaction> + Clone, D: Iterator<Item = NaiveDate>> {
    transactions: T,
    dates: D,
    consumed: usize,
    state: (NaiveDate, Accounts),
}

impl<T, D> History<T, D>
    where T: Iterator<Item = Transaction> + Clone,
          D: Iterator<Item = NaiveDate>
{
    pub fn new(state: (NaiveDate, Accounts), transactions: T, dates: D) -> History<T, D> {
        History {
            transactions: transactions,
            dates: dates,
            consumed: 0,
            state: state,
        }
    }
}

// this assumes that users have validated the transactions first :)
impl<T, D> Iterator for History<T, D>
    where T: Iterator<Item = Transaction> + Clone,
          D: Iterator<Item = NaiveDate>
{
    type Item = (NaiveDate, Accounts);

    fn next(&mut self) -> Option<Self::Item> {
        match self.dates.next() {
            Some(next_date) => {
                // consume the next few transactions
                let next_transactions = self.transactions
                    .clone()
                    .skip(self.consumed)
                    .take_while(|ref t| t.date <= next_date)
                    .collect::<Vec<_>>();
                self.consumed += next_transactions.len();

                // calculate next state
                self.state = (next_date,
                              next_transactions.into_iter()
                    .fold(self.state.1.clone(), Accounts::apply_unchecked));
                Some(self.state.clone())
            }
            None => None,
        }
    }
}

// TODO write tests for the accounts object
