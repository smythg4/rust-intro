use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum TransactionType {
    Deposit,
    Withdrawal,
    Interest,
    Fee
}

#[derive(Debug, Clone)]
struct Transaction {
    transaction_type: TransactionType,
    amount: f64,
    timestamp: DateTime<Utc>,
    description: Option<String>
}

#[derive(Debug)]
enum DepositError {
    NegativeAmount,
}

#[derive(Debug)]
enum WithdrawalError {
    InsufficientFunds,
    NegativeAmount,
}

#[derive(Debug)]
enum TransferError {
    InsufficientFunds,
    NegativeAmount,
    DepositFailed,
}

enum AccountType {
    Checking {
        overdraft_limit: f64,
        overdraft_fee: f64,
    },
    CD {
        maturity_date: DateTime<Utc>,
        early_withdrawal_fee: f64,
    },
}

struct BankAccount {
    name: String,
    starting_balance: f64,
    balance: f64,
    interest_rate: f64,
    transactions: Vec<Transaction>,
    account_type: AccountType,
}

impl BankAccount {
    fn new_checking(name: &str, balance: f64, interest_rate: f64, overdraft_limit: f64, overdraft_fee: f64) -> Self {
        BankAccount {
            name: name.to_string(),
            starting_balance: balance,
            balance,
            interest_rate,
            transactions: Vec::new(),
            account_type: AccountType::Checking{
                overdraft_limit,
                overdraft_fee,
            },
        }
    }

    fn new_cd(name: &str, balance: f64, interest_rate: f64, term_months: u32, early_withdrawal_fee: f64) -> Self {
        let maturity_date = Utc::now() + chrono::Duration::days(term_months as i64 * 30);
        BankAccount {
            name: name.to_string(),
            starting_balance: balance,
            balance,
            interest_rate,
            transactions: Vec::new(),
            account_type: AccountType::CD {
                maturity_date,
                early_withdrawal_fee,
            },
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_balance(&self) -> f64 {
        self.balance
    }

    fn accrue(&mut self) -> f64 {
        let interest_amount = self.balance * (self.interest_rate / 100.0);
        self.balance += interest_amount;

        self.transactions.push(Transaction {
            transaction_type: TransactionType::Interest,
            amount: interest_amount,
            timestamp: Utc::now(),
            description: Some(format!("Interest at {}%", self.interest_rate)),
        });

        self.balance
    }

    fn deposit(&mut self, amount: f64) -> Result<f64,DepositError> {
        if amount < 0.0 {
            return Err(DepositError::NegativeAmount);
        } else {
            self.balance += amount;

            self.transactions.push(Transaction {
                transaction_type: TransactionType::Deposit,
                amount,
                timestamp: Utc::now(),
                description: None
            });

            Ok(amount)
        }
    }

    fn withdraw(&mut self, amount: f64) -> Result<f64,WithdrawalError> {
        if amount < 0.0 {
            return Err(WithdrawalError::NegativeAmount);
        }

        match &mut self.account_type {
            AccountType::Checking { overdraft_limit, overdraft_fee } => {
                if amount > self.balance + *overdraft_limit {
                    return Err(WithdrawalError::InsufficientFunds);
                }
                if amount > self.balance {
                    self.balance -= *overdraft_fee;
                    self.transactions.push(Transaction {
                        transaction_type: TransactionType::Fee,
                        amount: *overdraft_fee,
                        timestamp: Utc::now(),
                        description: Some(format!("Overdraft fee")),
                    });
                }
            },
            AccountType::CD {maturity_date, early_withdrawal_fee} => {
                if Utc::now() < *maturity_date {
                    let penalty = amount * *early_withdrawal_fee;

                    if amount + penalty > self.balance {
                        return Err(WithdrawalError::InsufficientFunds);
                    }

                    self.balance -= penalty;

                    self.transactions.push( Transaction {
                        transaction_type: TransactionType::Fee,
                        amount: penalty,
                        timestamp: Utc::now(),
                        description: Some(format!("Early withdrawal fee {:.1}% of ${:.2}", *early_withdrawal_fee*100.0, amount)),
                    });
                } else {
                    if amount > self.balance {
                        return Err(WithdrawalError::InsufficientFunds);
                    }
                }
            },
        }

        self.balance -= amount;

        self.transactions.push(Transaction {
            transaction_type: TransactionType::Withdrawal,
            amount,
            timestamp: Utc::now(),
            description: None,
        });

        Ok(amount)
    }

    fn transfer(&mut self, other: &mut Self, amount: f64) -> Result<f64,TransferError> {
        if amount < 0.0 {
            return Err(TransferError::NegativeAmount);
        }

        let withdrawn_amount = self.withdraw(amount).map_err(|err| match err {
            WithdrawalError::InsufficientFunds => TransferError::InsufficientFunds,
            WithdrawalError::NegativeAmount => TransferError::NegativeAmount,
        })?;

        match other.deposit(withdrawn_amount) {
            Ok(deposited_amount) => Ok(deposited_amount),
            Err(_) => {
                let _ = self.deposit(withdrawn_amount).unwrap();
                Err(TransferError::DepositFailed)
            }
        }
    }

    fn summarize_transactions(&self) -> HashMap<TransactionType, f64> {
        let mut summary = HashMap::new();

        for transaction in &self.transactions {
            *summary.entry(transaction.transaction_type.clone()).or_insert(0.0) += transaction.amount;
        }

        summary
    }

    fn generate_statement(&self, start_date: Option<DateTime<Utc>>, end_date: Option<DateTime<Utc>>) -> String {
        let mut statement = format!("Statement for: {}\n", self.name);
        statement.push_str("Date                  | Type       | Amount      | Balance      | Description\n");
        statement.push_str("----------------------|------------|-------------|--------------|------------\n");
        
        let mut running_balance = self.starting_balance;

        for transaction in &self.transactions {
            // Skip if before start_date or after end_date
            if (start_date.is_some() && transaction.timestamp < start_date.unwrap()) ||
            (end_date.is_some() && transaction.timestamp > end_date.unwrap()) {
                continue;
            }
            match transaction.transaction_type {
                TransactionType::Deposit => running_balance += transaction.amount,
                TransactionType::Interest => running_balance += transaction.amount,
                TransactionType::Withdrawal => running_balance -= transaction.amount,
                TransactionType::Fee => running_balance -= transaction.amount,
            }
            statement.push_str(&format!("{}   | {:10} | ${:10.2} | ${:10.2}  | {}\n",
                transaction.timestamp.format("%Y-%m-%d %H:%M:%S"),
                format!("{:?}", transaction.transaction_type),
                transaction.amount,
                running_balance,
                transaction.description.as_deref().unwrap_or("")
            ));
        }
        
        statement.push_str(&format!("\nCurrent Balance: ${:.2}", self.balance));
        statement
    }
}

fn main() {
    let mut my_acct = BankAccount::new_checking("Stephen's Account", 1200.05, 0.5, 1000.0, 25.0);
    let mut your_acct = BankAccount::new_cd("Ashley's Account", 15000.00, 4.1, 36, 0.10);
    for i in 1..=10 {
        println!("{} - {}: ${:.2}", i, my_acct.get_name(), my_acct.get_balance());
        println!("{} - {}: ${:.2}", i, your_acct.get_name(), your_acct.get_balance());

        println!("   Attempting to deposit $10 to {}...", my_acct.get_name());
        match my_acct.deposit(10.0) {
            Ok(amount) => println!("      Deposit successful for ${:.2}", amount),
            Err(DepositError::NegativeAmount) => println!("      Error: Cannot deposit a negative amount"),
        }

        println!("   Attempting to withdraw $150 from {}...", my_acct.get_name());
        match my_acct.withdraw(150.0) {
            Ok(amount) => println!("      Withdrawal successful for ${:.2}", amount),
            Err(WithdrawalError::NegativeAmount) => println!("      Error: Cannot withdraw a negative amount"),
            Err(WithdrawalError::InsufficientFunds) => println!("      Error: Insufficient funds available")
        }

        println!("   Attempting to transfer $1,000 from {} to {}", your_acct.get_name(), my_acct.get_name());
        match your_acct.transfer(&mut my_acct, 1000.0) {
            Ok(amount) => println!("      Transfer successful for ${:.2}", amount),
            Err(TransferError::NegativeAmount) => println!("      Error: Cannot withdraw a negative amount"),
            Err(TransferError::InsufficientFunds) => println!("      Error: Insufficient funds available"),
            Err(TransferError::DepositFailed) => println!("      Error: Deposit failed"),
        }

        my_acct.accrue();
        your_acct.accrue();
    }

    let my_statement = your_acct.generate_statement(None, None);
    println!("{}",my_statement);
}
