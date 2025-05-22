use chrono::{DateTime, Utc};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
//use serde_json::Result;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
enum TransactionType {
    Deposit,
    Withdrawal,
    Interest,
    Fee,
    Tax,
    Sale,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    transaction_type: TransactionType,
    amount: f64,
    timestamp: DateTime<Utc>,
    description: Option<String>
}

#[derive(Debug)]
enum DepositError {
    NegativeAmount(f64),
}

#[derive(Debug)]
enum WithdrawalError {
    InsufficientFunds { requested: f64, available: f64 },
    NegativeAmount(f64),
}

#[derive(Debug)]
enum TransferError {
    InsufficientFunds { requested: f64, available: f64 },
    NegativeAmount(f64),
    DepositFailed,
}

impl fmt::Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferError::InsufficientFunds { requested, available } => 
                write!(f, "transfer failed - insufficient funds: requested ${:.2}, available ${:.2}", requested, available),
            TransferError::NegativeAmount(amt) => 
                write!(f, "cannot transfer negative amount: ${:.2}", amt),
            TransferError::DepositFailed => 
                write!(f, "transfer failed during deposit phase"),
        }
    }
}

impl fmt::Display for WithdrawalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WithdrawalError::InsufficientFunds { requested, available } => 
                write!(f, "insufficient funds: requested ${:.2}, available ${:.2}", requested, available),
            WithdrawalError::NegativeAmount(amt) => 
                write!(f, "cannot withdraw negative amount: ${:.2}", amt),
        }
    }
}

impl fmt::Display for DepositError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DepositError::NegativeAmount(amt) => 
                write!(f, "cannot deposit a negative amount: ${:.2}", amt),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
struct BankAccount {
    name: String,
    account_type: AccountType,
    starting_balance: f64,
    balance: f64,
    interest_rate: f64,
    transactions: Vec<Transaction>,
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

    fn calc_cost_basis(&self) -> f64 {
        let mut cost_basis = self.starting_balance;
        println!("Initial Cost Basis: ${:.2}", cost_basis);
        for trans in &self.transactions {
            match trans.transaction_type {
                TransactionType::Deposit => cost_basis += trans.amount,
                TransactionType::Fee => cost_basis -= trans.amount,
                TransactionType::Tax => cost_basis -= trans.amount,
                TransactionType::Withdrawal => cost_basis -= trans.amount,
                _ => (),
            };
        }
        println!("Final Cost Basis: ${:.2}", cost_basis);

        cost_basis
    }

    fn calc_capital_gains(&self, transaction: &Transaction) -> f64 {
        println!("{:?}", transaction);
        let cost_basis = self.calc_cost_basis();
        if cost_basis > self.balance {
            return 0.0;
        }
        let cb_ratio = (self.balance - cost_basis)/self.starting_balance;
        match transaction.transaction_type {
            TransactionType::Withdrawal | TransactionType::Fee => transaction.amount * cb_ratio * 0.15,
            _ => 0.0
        }
    }

    fn accrue(&mut self) -> f64 {
        let interest_amount = self.balance * (self.interest_rate / 100.0);
        if interest_amount > 0.0 {
            self.balance += interest_amount;
            self.transactions.push(Transaction {
                transaction_type: TransactionType::Interest,
                amount: interest_amount,
                timestamp: Utc::now(),
                description: Some(format!("Interest at {}%", self.interest_rate)),
        });
        }

        self.balance
    }

    fn deposit(&mut self, amount: f64, note: Option<&str>) -> Result<f64,DepositError> {
        if amount < 0.0 {
            return Err(DepositError::NegativeAmount(amount));
        } else {

            let note = match note { Some(n) => Some(n.to_string()), None => None };
            self.balance += amount;

            self.transactions.push(Transaction {
                transaction_type: TransactionType::Deposit,
                amount,
                timestamp: Utc::now(),
                description: note,
            });

            Ok(amount)
        }
    }

    fn withdraw(&mut self, amount: f64, note: Option<&str>) -> Result<f64,WithdrawalError> {
        if amount < 0.0 {
            return Err(WithdrawalError::NegativeAmount(amount));
        }
        
        match &mut self.account_type {
            AccountType::Checking { overdraft_limit, overdraft_fee } => {
                if amount > self.balance + *overdraft_limit {
                    return Err(WithdrawalError::InsufficientFunds{requested: amount, available: self.balance});
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
                        return Err(WithdrawalError::InsufficientFunds{ requested: amount+penalty, available: self.balance});
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
                        return Err(WithdrawalError::InsufficientFunds{requested: amount, available: self.balance});
                    }
                }
            },
        }
        let note = match note { Some(n) => Some(n.to_string()), None => None };
        self.balance -= amount;

        self.transactions.push(Transaction {
            transaction_type: TransactionType::Withdrawal,
            amount,
            timestamp: Utc::now(),
            description: note,
        });

        Ok(amount)
    }

    fn transfer(&mut self, other: &mut Self, amount: f64) -> Result<f64,TransferError> {
        if amount < 0.0 {
            return Err(TransferError::NegativeAmount(amount));
        }

        let note = format!("transfer to {}", other.get_name());
        let withdrawn_amount = self.withdraw(amount, Some(&note)).map_err(|err| match err {
            WithdrawalError::InsufficientFunds { requested, available } => 
                TransferError::InsufficientFunds { requested, available },
            WithdrawalError::NegativeAmount(amt) => TransferError::NegativeAmount(amt),
        })?;

        let note = format!("transfer from {}", self.get_name());
        match other.deposit(withdrawn_amount, Some(&note)) {
            Ok(deposited_amount) => Ok(deposited_amount),
            Err(_) => {
                let _ = self.deposit(withdrawn_amount, None).unwrap();
                Err(TransferError::DepositFailed)
            }
        }
    }

    fn load_accounts_from_json(filepath: &str) -> serde_json::Result<Vec<BankAccount>> {
        let json_data = fs::read_to_string(filepath)
            .map_err(serde_json::Error::io)?;
        let accounts: Vec<BankAccount> = serde_json::from_str(&json_data)?;

        Ok(accounts)
    }

    fn write_json_to_file(&self, filepath: &str) -> serde_json::Result<()> {
        let json_data = serde_json::to_string_pretty(&self)?;
        let mut file = File::create(filepath)
            .map_err(serde_json::Error::io)?;

        file.write_all(json_data.as_bytes())
            .map_err(serde_json::Error::io)?;

        Ok(())
    }

    fn write_accounts_json_to_file(accounts: &[BankAccount], filepath: &str) -> serde_json::Result<()> {
        let json_data = serde_json::to_string_pretty(accounts)?;
        let mut file = File::create(filepath)
            .map_err(serde_json::Error::io)?;

        file.write_all(json_data.as_bytes())
            .map_err(serde_json::Error::io)?;

        Ok(())
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
                TransactionType::Tax => running_balance -= transaction.amount,
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

    fn reset(&mut self) {
        self.transactions = Vec::<Transaction>::new();
        self.balance = self.starting_balance;
    }
}

fn main() -> serde_json::Result<()> {
    //let mut my_acct = BankAccount::new_checking("Stephen's Account", 1200.05, 0.5, 1000.0, 25.0);
    //let mut your_acct = BankAccount::new_cd("Ashley's Account", 15000.00, 4.1, 36, 0.10);
    
    let filepath = "output.json";
    let mut accounts =  BankAccount::load_accounts_from_json(filepath)?;

    let mut my_acct = accounts.pop().unwrap();
    let mut your_acct = accounts.pop().unwrap();

    for i in 1..=10 {
        println!("{} - {}: ${:.2}", i, my_acct.get_name(), my_acct.get_balance());
        println!("{} - {}: ${:.2}", i, your_acct.get_name(), your_acct.get_balance());

        println!("   Attempting to deposit $10 to {}...", my_acct.get_name());
        match my_acct.deposit(10.0, None) {
            Ok(amount) => println!("      Deposit successful for ${:.2}", amount),
            Err(e) => eprintln!("      Error: {}", e),
        }

        println!("   Attempting to withdraw $150 from {}...", my_acct.get_name());
        match my_acct.withdraw(150.0, None) {
            Ok(amount) => println!("      Withdrawal successful for ${:.2}", amount),
            Err(e) => eprintln!("      Error: {}",e),//eprintln!("      Error: Cannot withdraw a negative amount ({})", amt),
        }

        println!("   Attempting to transfer $1,000 from {} to {}", your_acct.get_name(), my_acct.get_name());
        match your_acct.transfer(&mut my_acct, 1000.0) {
            Ok(amount) => println!("      Transfer successful for ${:.2}", amount),
            Err(e) => eprintln!("   {}", e),
        }

        println!("   Attempting to transfer $750 from {} to {}", my_acct.get_name(), your_acct.get_name());
        match my_acct.transfer(&mut your_acct, 750.0) {
            Ok(amount) => println!("      Transfer successful for ${:.2}", amount),
            Err(e) => eprintln!("   {}", e),
        }

        my_acct.accrue();
        your_acct.accrue();
    }

    let my_statement = my_acct.generate_statement(None, None);
    println!("{}",my_statement);

    my_acct.reset();
    your_acct.reset();

    accounts.push(your_acct);
    accounts.push(my_acct);
    match BankAccount::write_accounts_json_to_file(&accounts, filepath) {
        Ok(_) => println!("Successfully wrote JSON to file: {}", filepath),
        Err(e) => eprintln!("File write error: {}", e)
    };

    Ok(())
}
