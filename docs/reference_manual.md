# Help
```
financial-planner 0.1
Nicholas D. <nickdujay@gmail.com>
Helps you plan your financial future.

USAGE:
    financial-planner-cli [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f <INPUT>        Sets the input file to use.

SUBCOMMANDS:
    forecast    Calculate Asset values over <n> years.
    help        Prints this message or the help of the given subcommand(s)
```

# Input File
An example input file:

```yaml
accounts:
    assets:
        'Stocks':
            amount: 5000
    liabilities:
        'Credit Card Debt':
            amount: 1000
        'Other Debt':
            amount: 2000
income:
    'BiWeekly Pay Cheque':
        start_date: '2017-01-06'
        amount: 100
        frequency: BiWeekly
rules:
    'Deposit Pay Cheque into Stocks':
        amount: 100
        from: 'BiWeekly Pay Cheque'
        to: 'assets:Stocks'
```

There are three main sections, accounts, income and rules

## Accounts
These are financial accounts. Similar to ledger, you can reference accounts separated by `:`.
For example, `assets:stocks`, `liaibilities:Credit Card Debt`.

## Income
These are a special form of an account. It is an account that generates income that you can distribute
to other accounts.

## Rules
These determine what you are doing with your money. For example, you can transfer $100 from your income
to your debt, or your assets and see what happens over time by doing that. Depending on how high your
interest rate on your debt is, it may be better to pay the minimum and invest the difference.
