use std::collections::HashMap;
use std::cmp::Ordering;
use std::fmt;
use std::iter::Peekable;
use chrono::prelude::*;
use rugflo::Float;

use money::Money;
use errors::*;
use expression::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Accounts {
    Tree(HashMap<String, Accounts>),
    Leaf(Account),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Account {
    Simple(SimpleAccount),
    Derived(DerivedAccount),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimpleAccount {
    pub amount: Money,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DerivedAccount {
    pub expression: Expr,
}

fn eval(expr: &Expr, root: &Accounts) -> Result<Money> {
    match *expr {
        Expr::Id(ref name) => root.get(name).map(Accounts::sum),
        Expr::Add(ref left, ref right) => {
            eval(left, root)
                .and_then(|left_val| eval(right, root).map(|right_val| left_val + right_val))
        }
        Expr::Sub(ref left, ref right) => {
            eval(left, root)
                .and_then(|left_val| eval(right, root).map(|right_val| left_val - right_val))
        }
    }
}

impl Account {
    pub fn amount(&self) -> Money {
        match *self {
            Account::Simple(ref s) => s.amount.clone(),
            Account::Derived(_) => Money::zero(),
        }
    }
}

impl Accounts {
    pub fn root() -> Accounts {
        Accounts::Tree(HashMap::new())
    }

    pub fn leaf(&self) -> Result<&Account> {
        match *self {
            Accounts::Tree(_) => Err(ErrorKind::UnwrapNode.into()),
            Accounts::Leaf(ref l) => Ok(l),
        }
    }

    pub fn get(&self, path: &str) -> Result<&Accounts> {
        match self {
            &Accounts::Tree(ref m) => {
                if let Some(index) = path.find(':') {
                    let (account, sub_account) = path.split_at(index);
                    trace!("get [{}] split ([{}],[{}])", path, account, sub_account);
                    m.get(account)
                        .ok_or_else(|| ErrorKind::InvalidAccountName(String::from(path)).into())
                        .and_then(|a| a.get(&sub_account[1..]))
                } else {
                    m.get(path)
                        .ok_or_else(|| ErrorKind::InvalidAccountName(String::from(path)).into())
                }
            }
            a => {
                if path.len() == 0 {
                    Ok(a)
                } else {
                    Err(ErrorKind::InvalidAccountName(String::from(path)).into())
                }
            }
        }
    }

    pub fn get_mut(&mut self, path: &str) -> Result<&mut Accounts> {
        match self {
            &mut Accounts::Tree(ref mut m) => {
                if let Some(index) = path.find(':') {
                    let (account, sub_account) = path.split_at(index);
                    trace!("get_mut [{}] split ([{}],[{}])", path, account, sub_account);
                    m.get_mut(account)
                        .ok_or_else(|| ErrorKind::InvalidAccountName(String::from(path)).into())
                        .and_then(|a| a.get_mut(&sub_account[1..]))
                } else {
                    m.get_mut(path)
                        .ok_or_else(|| ErrorKind::InvalidAccountName(String::from(path)).into())
                }
            }
            a => {
                if path.len() == 0 {
                    Ok(a)
                } else {
                    Err(ErrorKind::InvalidAccountName(String::from(path)).into())
                }
            }
        }
    }

    pub fn sum(&self) -> Money {
        let result = match *self {
            Accounts::Tree(ref m) => {
                let mut result = Money::from(0);
                for (_, account) in m {
                    result += account.sum();
                }
                result
            }
            Accounts::Leaf(ref a) => a.amount(),
        };
        trace!("sum({:?}):({})", self, result);
        result
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

    pub fn create_account(&mut self, path: String, account: Account) -> Result<()> {
        match *self {
            Accounts::Tree(ref mut m) => {
                if let Some(index) = path.find(':') {
                    let (path, sub_path) = path.split_at(index);
                    m.entry(String::from(path))
                        .or_insert_with(Accounts::root)
                        .create_account(String::from(&sub_path[1..]), account)?;
                    Ok(())
                } else if m.contains_key(&path) {
                    Err(ErrorKind::AlreadyExists(path).into())
                } else {
                    m.insert(path, Accounts::Leaf(account));
                    Ok(())
                }
            }
            _ => {
                if path.len() == 0 {
                    Ok(())
                } else {
                    Err(ErrorKind::InvalidAccountName(path).into())
                }
            }
        }
    }

    pub fn deposit(&mut self, path: String, amount: Money) -> Result<()> {
        trace!("deposit ({}) into [{}]", amount, path);
        match *self {
            Accounts::Tree(ref mut m) => {
                if let Some(index) = path.find(':') {
                    let (path, sub_path) = path.split_at(index);
                    m.entry(String::from(path))
                        .or_insert_with(Accounts::root)
                        .deposit(String::from(&sub_path[1..]), amount)?;
                    Ok(())
                } else if m.contains_key(&path) {
                    m.get_mut(&path).unwrap()
                        .deposit(path.clone(), amount)?;
                    Ok(())
                } else {
                Ok(())
                }
            }
            Accounts::Leaf(Account::Simple(ref mut s)) => {
                s.amount += amount;
                Ok(())
            }
            Accounts::Leaf(Account::Derived(_)) => {
                Err(ErrorKind::InvalidDeposit(path, amount.to_string()).into())
            }
        }
    }

    pub fn withdraw(&mut self, path: String, amount: Money) -> Result<()> {
        self.deposit(path, -amount)
    }

    pub fn validate(&self) -> Result<()> {
        if let Accounts::Tree(ref m) = *self {
            for (path, account) in m {
                if let Some(_) = path.find(':') {
                    return Err(ErrorKind::InvalidAccountName(path.clone()).into());
                }
                account.validate()?;
            }
        }
        // TODO validate that derived accounts makes sense
        Ok(())
    }

    pub fn apply(&mut self, transaction: Transaction) -> Result<()> {
        trace!("apply: {}", transaction);
        let from_amount = transaction.from_amount(self)?;

        if self.get(&transaction.from).is_err() {
            self.create_account(transaction.from.clone(), Account::Simple(SimpleAccount { amount: Money::from(0) }))?;
        }
        if self.get(&transaction.to).is_err() {
            self.create_account(transaction.to.clone(), Account::Simple(SimpleAccount { amount: Money::from(0) }))?;
        }

        self.withdraw(transaction.from, from_amount.clone())?;
        self.deposit(transaction.to, from_amount)?;
        Ok(())
    }

    pub fn eval(&self) -> Result<HashMap<String, Money>> {
        let mut result = HashMap::new();
        for name in self.paths() {
            let account = self.get(&name)?.leaf()?;
            result.insert(name,
                          match *account {
                              Account::Simple(ref s) => s.amount.clone(),
                              Account::Derived(ref d) => eval(&d.expression, self)?,
                          });
        }
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub amount: Amount,
    pub from: String,
    pub to: String,
    pub date: NaiveDate,
}

impl Transaction {
    pub fn new(amount: Amount, from: String, to: String, date: NaiveDate) -> Transaction {
        Transaction {
            amount: amount.into(),
            from: from,
            to: to,
            date: date,
        }
    }

    pub fn from_amount(&self, accounts: &Accounts) -> Result<Money> {
        self.amount.eval(accounts, &self.from)
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
        write!(f, "[{}] sending ({}) to [{}] on {{{}}}", self.from, self.amount, self.to, self.date)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Amount {
    Money(Money),
    Percent(f64),
}

impl From<Money> for Amount {
    fn from(money: Money) -> Amount {
        Amount::Money(money)
    }
}

impl From<f64> for Amount {
    fn from(percent: f64) -> Amount {
        Amount::Percent(percent)
    }
}

impl Amount {
    fn eval(&self, accounts: &Accounts, from: &str) -> Result<Money> {
        let evaluated = accounts.eval()?;

        let result = match *self {
            Amount::Money(ref m) => Ok(m.clone()),
            Amount::Percent(p) => {
                evaluated.get(from)
                    .cloned()
                    .ok_or_else(|| ErrorKind::InvalidAccountName(String::from(from)).into())
                    .map(|account| account.mul_percent(Float::from((p, 64))))
            }
        };
        trace!("eval({:?},{}):({:?})", accounts, from, result);
        result
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Amount::Money(ref m) => m.fmt(f),
            Amount::Percent(ref p) => p.fmt(f),
        }
    }
}

pub struct History<T: Iterator<Item = Transaction> + Clone, D: Iterator<Item = NaiveDate>> {
    transactions: Peekable<T>,
    dates: D,
    state: (NaiveDate, Accounts),
}

impl<T, D> History<T, D>
    where T: Iterator<Item = Transaction> + Clone,
          D: Iterator<Item = NaiveDate>
{
    pub fn new(state: (NaiveDate, Accounts), transactions: T, dates: D) -> History<T, D> {
        History {
            transactions: transactions.peekable(),
            dates: dates,
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

                self.state.0 = next_date;

                loop {
                    let consume = if let Some(transaction) = self.transactions.peek() {
                        transaction.date <= next_date
                    } else {
                        false
                    };
                    if consume {
                        self.state.1.apply(self.transactions.next().unwrap()).unwrap();
                    } else {
                        break;
                    }
                }

                Some(self.state.clone())
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum() {
        // TODO
    }
}
