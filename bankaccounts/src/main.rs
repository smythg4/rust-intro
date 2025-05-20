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

struct BankAccount {
    name: String,
    balance: f64,
    interest_rate: f64,
    transactions: Vec<Transaction>
}

impl BankAccount {
    fn new(name: &str, balance: f64, interest_rate: f64) -> Self {
        BankAccount {
            name: name.to_string(),
            balance,
            interest_rate,
            transactions: Vec::new(),
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
            Err(DepositError::NegativeAmount)
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
            Err(WithdrawalError::NegativeAmount)
        } else if amount > self.balance {
            Err(WithdrawalError::InsufficientFunds)
        } else {
            self.balance -= amount;

            self.transactions.push(Transaction {
                transaction_type: TransactionType::Withdrawal,
                amount,
                timestamp: Utc::now(),
                description: None
            });

            Ok(amount)
        }
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
}

fn main() {
    let mut my_acct = BankAccount::new("Stephen's Account", 1200.05, 4.7);
    let mut your_acct = BankAccount::new("Ashley's Account", 15000.00, 2.1);
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

    let transaction_summary = your_acct.summarize_transactions();
    println!("Transaction Summary for {}:", your_acct.get_name());
    for (transaction_type, total) in &transaction_summary {
        println!("  {:?}: ${:.2}", transaction_type, total);
    }

    let transaction_summary = my_acct.summarize_transactions();
    println!("Transaction Summary for {}:", my_acct.get_name());
    for (transaction_type, total) in &transaction_summary {
        println!("  {:?}: ${:.2}", transaction_type, total);
    }
}
