use chrono::{DateTime, Utc};
use std::{collections::HashMap, mem};
use serde::{Deserialize, Serialize};

use crate::errors::{DepositError, WithdrawalError, TransferError};

pub trait Account {
    fn deposit(&mut self, amount: f64, note: Option<&str>) -> Result<f64, DepositError>;
    fn withdraw(&mut self, amount: f64, note: Option<&str>) -> Result<f64, WithdrawalError>;
    fn transfer(&mut self, other: &mut dyn Account, amount: f64, note: Option<&str>) -> Result<f64, TransferError>;
    fn accrue(&mut self) -> f64;
    fn get_balance(&self) -> f64;
    fn get_cash_balance(&self) -> f64 {
        self.get_balance()
    }
    fn get_name(&self) -> &str;
    fn get_starting_balance(&self) -> f64;
    fn generate_transactions(&self) -> &Vec<Transaction>;
    fn generate_statement(&self, start_date: Option<DateTime<Utc>>, end_date: Option<DateTime<Utc>>) -> String {
        let mut statement = format!("Statement for: {}\n", self.get_name());
        statement.push_str("Date                  | Type           | Amount      | Balance      | Description\n");
        statement.push_str("----------------------|----------------|-------------|--------------|------------\n");
        
        let mut running_balance = self.get_starting_balance();

        for transaction in self.generate_transactions() {
            // Skip if before start_date or after end_date
            if (start_date.is_some() && transaction.timestamp < start_date.unwrap()) ||
            (end_date.is_some() && transaction.timestamp > end_date.unwrap()) {
                continue;
            }
            match transaction.transaction_type {
                TransactionType::Deposit => running_balance += transaction.amount,
                TransactionType::Interest => running_balance += transaction.amount,
                TransactionType::UnrealizedGain => running_balance += transaction.amount,
                TransactionType::Withdrawal => running_balance -= transaction.amount,
                TransactionType::Fee => running_balance -= transaction.amount,
                TransactionType::Tax => running_balance -= transaction.amount,
                _ => (),
            }
            statement.push_str(&format!("{}   | {:14} | ${:10.2} | ${:10.2}  | {}\n",
                transaction.timestamp.format("%Y-%m-%d %H:%M:%S"),
                format!("{:?}", transaction.transaction_type),
                transaction.amount,
                running_balance,
                transaction.description.as_deref().unwrap_or("")
            ));
        }
        
        statement.push_str(&format!("\nCurrent Balance: ${:.2}", self.get_balance()));
        statement
    }

    fn summarize_transactions(&self) -> HashMap<TransactionType, f64> {
        let mut summary = HashMap::new();

        for transaction in self.generate_transactions() {
            *summary.entry(transaction.transaction_type.clone()).or_insert(0.0) += transaction.amount;
        }

        summary
    }

    fn validate_balance(&self) -> Result<(), String> {
        let my_start_bal = self.get_starting_balance();

        let summary = self.summarize_transactions();
        let total_deposits = *summary.get(&TransactionType::Deposit).unwrap_or(&0.0);
        let total_withdrawals = *summary.get(&TransactionType::Withdrawal).unwrap_or(&0.0);
        let total_interest = *summary.get(&TransactionType::Interest).unwrap_or(&0.0);
        let total_fees = *summary.get(&TransactionType::Fee).unwrap_or(&0.0);
        let total_tax = *summary.get(&TransactionType::Tax).unwrap_or(&0.0);
        let total_ur_gains = *summary.get(&TransactionType::UnrealizedGain).unwrap_or(&0.0);

        let total_in = total_deposits+total_interest+total_ur_gains;
        let total_out = total_withdrawals+total_tax+total_fees;
        let expected_bal = my_start_bal+total_in-total_out;

        if (self.get_balance() - expected_bal).abs() < 0.01 {
            Ok(())
        } else {
            Err(format!(
                "Balance mismatch: actual ${:.2}, expected ${:.2}",
                self.get_balance(), expected_bal
            ))
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Interest,
    UnrealizedGain,
    Fee,
    Tax,
    Sale,
    Purchase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_type: TransactionType,
    pub amount: f64,
    timestamp: DateTime<Utc>,
    description: Option<String>
}


#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash, Copy)]
pub enum AssetClass {
    Equity,
    Bond,
    Other,
}

#[derive(Debug, Deserialize, Serialize,  Clone)]
pub struct Asset {
    symbol: String,
    shares: f64,
    cost_basis: f64,
    current_price: f64,
    asset_class: AssetClass,
}

impl Asset {
    fn get_value(&self) -> f64 {
        self.shares * self.current_price
    }

    fn get_rate_of_return(&self) -> f64 {
        match self.asset_class {
            AssetClass::Equity => 10.0,
            AssetClass::Bond => 3.5,
            AssetClass::Other => 0.0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CheckingSavingsAccount {
    name: String,
    starting_balance: f64,
    balance: f64,
    interest_rate: f64,
    overdraft_limit: f64,
    overdraft_fee: f64,
    transactions: Vec<Transaction>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct CDAccount {
    name: String,
    starting_balance: f64,
    balance: f64,
    interest_rate: f64,
    maturity_date: DateTime<Utc>,
    early_withdrawal_penalty: f64,
    transactions: Vec<Transaction>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct BrokerageAccount {
    name: String,
    starting_balance: f64,
    cash_balance: f64,
    cash_interest: f64,
    assets: Vec<Asset>,
    transactions: Vec<Transaction>,
}

impl BrokerageAccount {
    pub fn new(name: &str, starting_balance: f64, cash_interest: f64) -> Self {
        BrokerageAccount {
            name: name.to_string(),
            starting_balance,
            cash_interest,
            cash_balance: starting_balance,
            assets: Vec::new(),
            transactions: Vec::new(),
        }
    }

    fn has_enough_cash(&self, amount: f64) -> bool {
        (self.cash_balance - amount).abs() > 0.01
    }

    pub fn get_asset_alloc(&self) -> (f64, f64, f64) {
        let total_bal = self.get_balance();
        let cash_bal = self.cash_balance;
        let bond_bal = self.assets.iter().filter(|x|x.asset_class == AssetClass::Bond)
        .fold(0.0, |acc, asset| {
            acc + (asset.shares * asset.current_price)
        });
        let equity_bal = self.assets.iter().filter(|x|x.asset_class == AssetClass::Equity)
        .fold(0.0, |acc, asset| {
            acc + (asset.shares * asset.current_price)
        });

        (cash_bal/total_bal, equity_bal/total_bal, bond_bal/total_bal)
    }

    pub fn buy(&mut self, shares: f64, price: f64, asset_class: AssetClass) -> Result<f64,WithdrawalError> {
        let amount = price * shares;
        if !self.has_enough_cash(amount) {
            return Err(WithdrawalError::InsufficientFunds { requested: amount, available: self.cash_balance });
        }

        self.cash_balance -= amount;
        let new_asset = Asset{
            symbol: "GEN".to_string(),
            shares,
            cost_basis: amount,
            current_price: price,
            asset_class,
        };
        
        self.transactions.push( Transaction {
                transaction_type: TransactionType::Purchase,
                amount: amount,
                timestamp: Utc::now(),
                description: Some(format!("Purchased {:.2} shares of {} at ${:.2}. Allocation: {:?}",
                                    new_asset.shares, new_asset.symbol, new_asset.current_price, self.get_asset_alloc())),
            });

        self.assets.push(new_asset);
        
        Ok(amount)
    }

    fn calc_cap_gains_tax(&self, assets_to_sell: &Vec::<(Asset, f64)>) -> f64 {
        let mut cap_gains = 0.0;
        for (asset, shares) in assets_to_sell {
            let per_share_cb = asset.cost_basis / asset.shares;
            cap_gains += shares * (asset.current_price - per_share_cb);
        }
        (cap_gains * 0.15).max(0.0)
    }

    pub fn sell(&mut self, amount: f64, class: AssetClass) -> Result<f64,WithdrawalError> {
        if amount < 0.0 {
            return Err(WithdrawalError::NegativeAmount(amount));
        }
        let mut cash_raised = 0.0;
        let mut assets_to_sell = Vec::new();

        //let mut assets = self.get_assets_of_type(class);

        while cash_raised < amount && !self.assets.is_empty() {
            if let Some(mut asset) = self.assets.pop() {
                let asset_value = asset.get_value();
                if cash_raised + asset_value <= amount {
                    let shares = asset.shares;
                    cash_raised += asset_value;
                    assets_to_sell.push((asset, shares));
                } else {
                    let remaining_needed = amount - cash_raised;
                    let shares_to_sell = remaining_needed / asset.current_price;
                    let shares_to_keep = asset.shares - shares_to_sell;

                    cash_raised += remaining_needed;
                    assets_to_sell.push((asset.clone(), shares_to_sell));

                    asset.shares = shares_to_keep;
                    asset.cost_basis = asset.cost_basis * (shares_to_keep / (shares_to_keep+shares_to_sell));
                    self.assets.push(asset);
                    break;
                }
            }
        }
        if cash_raised < amount {
            // not enough money raised, put the shares back
            for (asset, _) in assets_to_sell {
                self.assets.push(asset);
            }
            return Err(WithdrawalError::InsufficientFunds { requested: amount, available: cash_raised } );
        }

        //calculate capital gains tax
        let tax = self.calc_cap_gains_tax(&assets_to_sell);
        //withdraw cash to pay capital gains tax
        if self.cash_balance < tax {
            return Err(WithdrawalError::InsufficientFunds {
                requested: tax,
                available: self.cash_balance,
            });
        }
        if tax >= 0.0 {
            self.cash_balance -= tax;
            self.transactions.push( Transaction {
                transaction_type: TransactionType::Tax,
                amount: tax,
                timestamp: Utc::now(),
                description: Some(format!("Capital gains tax paid on asset sale"))
            });
        }

        self.cash_balance += amount;
        for (asset, shares_sold) in assets_to_sell {
            let proceeds = shares_sold * asset.current_price;
            self.transactions.push( Transaction {
                transaction_type: TransactionType::Sale,
                amount: proceeds,
                timestamp: Utc::now(),
                description: Some(format!("Sold {:.2} shares of {} at ${:.2}",
                                    shares_sold, asset.symbol, asset.current_price)),
            });
        }

        Ok(amount)
    }

/* From Claude
fn extract_assets_of_type(&mut self, class: AssetClass) -> Vec<Asset> {
    let mut extracted = Vec::new();
    let mut remaining = Vec::new();
    
    for asset in self.assets.drain(..) {
        if mem::discriminant(&asset.asset_class) == mem::discriminant(&class) {
            extracted.push(asset);
        } else {
            remaining.push(asset);
        }
    }
    
    self.assets = remaining;
    extracted
} */

    fn get_assets_of_type(& self, class: AssetClass) -> Vec<&Asset> {
        self.assets.iter().filter(|asset| mem::discriminant(&asset.asset_class) == mem::discriminant(&class))
            .collect()
    }

    pub fn soft_rebalance(&mut self, target_equity_alloc: f64, target_cash_alloc: f64) -> Result<(f64, f64, f64) ,WithdrawalError> {
        // uses cash in the account to purchase shares of equity or bonds to achieve goal portfolio balance
        // returns a tuple with the new allocation percentages (cash, equity, bond)

        // convert these into error return cases
        assert!(target_equity_alloc <= 1.0);
        assert!(target_equity_alloc >= 0.0);

        let total_balance = self.get_balance(); // does this actually get me a performance "boost" by "caching"?
        let (_, equity_alloc, bond_alloc) = self.get_asset_alloc();
        let asset_balance = total_balance * (1.0-target_cash_alloc); // retain cash buffer
        if asset_balance <= 0.0 {
            return Ok(self.get_asset_alloc());
        }


        let target_equity_bal = target_equity_alloc * asset_balance;
        let target_bond_bal = (1.0-target_equity_alloc) * asset_balance;

        let act_equity_bal = equity_alloc*total_balance;
        let act_bond_bal = bond_alloc*total_balance;

        let mut need_for_equity = (target_equity_bal - act_equity_bal).max(0.0);
        let mut need_for_bond = (target_bond_bal - act_bond_bal).max(0.0);

        if self.cash_balance < need_for_bond + need_for_equity {
            let ratio = self.cash_balance / (need_for_bond + need_for_equity);
            need_for_bond *= ratio;
            need_for_equity *= ratio;
        }

        if need_for_equity > 0.0 {self.buy(1.0, need_for_equity, AssetClass::Equity)?;};
        if need_for_bond > 0.0 {self.buy(1.0, need_for_bond, AssetClass::Bond)?;};

        Ok(self.get_asset_alloc())
    }
}

impl CheckingSavingsAccount {
    pub fn new(name: &str, balance: f64, interest_rate: f64, overdraft_limit: f64, overdraft_fee: f64) -> Self {
        CheckingSavingsAccount {
            name: name.to_string(),
            starting_balance: balance,
            balance,
            interest_rate,
            overdraft_limit,
            overdraft_fee,
            transactions: Vec::new(),
        }
    }

/*     fn load_accounts_from_json(filepath: &str) -> serde_json::Result<Vec<BankAccount>> {
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
    } */

    // pub fn reset(&mut self) {
    //     self.transactions = Vec::<Transaction>::new();
    //     self.balance = self.starting_balance;
    // }
}

impl Account for BrokerageAccount {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_balance(&self) -> f64 {
        self.assets.iter().fold(self.cash_balance, |acc, asset| {
            acc + (asset.shares * asset.current_price)
        })
    }

    fn get_cash_balance(&self) -> f64 {
        self.cash_balance
    }

    fn get_starting_balance(&self) -> f64 {
        self.starting_balance
    }

    fn generate_transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    fn deposit(&mut self, amount: f64, note: Option<&str>) -> Result<f64,DepositError> {
        if amount < 0.0 {
            return Err(DepositError::NegativeAmount(amount));
        } else {

            let note = match note { Some(n) => Some(n.to_string()), None => None };
            self.cash_balance += amount;

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

        let cash_note = note.map(|n| format!("Cash withdrawal. {}", n));
        
        if !self.has_enough_cash(amount){
            let target = amount - self.cash_balance;
            self.sell(target, AssetClass::Bond)?;
        }
        
        if self.cash_balance < amount {
            return Err(WithdrawalError::InsufficientFunds {
                requested: amount,
                available: self.cash_balance,
            });
        }
        self.cash_balance -= amount;

        self.transactions.push(Transaction {
            transaction_type: TransactionType::Withdrawal,
            amount,
            timestamp: Utc::now(),
            description: cash_note,
        });

        Ok(amount)
    }

    fn transfer(&mut self, other: &mut dyn Account, amount: f64, note: Option<&str>) -> Result<f64,TransferError> {
        if amount < 0.0 {
            return Err(TransferError::NegativeAmount(amount));
        }

        let withdraw_note = match note {
            Some(note) => format!("{}. transfer to {}", note, other.get_name()),
            None => format!("transfer to {}", other.get_name())
        };

        let withdrawn_amount = self.withdraw(amount, Some(&withdraw_note)).map_err(|err| match err {
            WithdrawalError::InsufficientFunds { requested, available } => 
                TransferError::InsufficientFunds { requested, available },
            WithdrawalError::NegativeAmount(amt) => TransferError::NegativeAmount(amt),
        })?;

        let deposit_note = match note {
            Some(note) => format!("{}. transfer from {}", note, self.get_name()),
            None => format!("transfer from {}", self.get_name())
        };

        match other.deposit(withdrawn_amount, Some(&deposit_note)) {
            Ok(deposited_amount) => Ok(deposited_amount),
            Err(_) => {
                let _ = self.deposit(withdrawn_amount, None).unwrap();
                Err(TransferError::DepositFailed)
            }
        }
    }

    fn accrue(&mut self) -> f64 {
        let interest_amount = self.cash_balance * (self.cash_interest / 100.0);
        if interest_amount > 0.0 {
            self.cash_balance += interest_amount;
            self.transactions.push(Transaction {
                transaction_type: TransactionType::Interest,
                amount: interest_amount,
                timestamp: Utc::now(),
                description: Some(format!("Brokerage interest at {}%", self.cash_interest)),
            });
        }
        // Calculate period gains from asset price changes

        let period_gains: f64 = self.assets.iter_mut().map(|asset| {
            let old_value = asset.current_price * asset.shares;
            asset.current_price *= 1.0 + (asset.get_rate_of_return() / 100.0);
            let new_value = asset.current_price * asset.shares;
            new_value - old_value // This period's gain only
        }).sum();

        // Record period gains as a transaction (optional)
        if period_gains > 0.0 {
            let rate = period_gains/(self.get_balance()-self.cash_balance);
            self.transactions.push(Transaction {
                transaction_type: TransactionType::UnrealizedGain,
                amount: period_gains,
                timestamp: Utc::now(),
                description: Some(format!("Brokerage gains of {:.2}%", rate*100.0)),
            });
        }
        interest_amount + period_gains
    }
}

impl Account for CheckingSavingsAccount {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_balance(&self) -> f64 {
        self.balance
    }

    fn get_starting_balance(&self) -> f64 {
        self.starting_balance
    }

    fn generate_transactions(&self) -> &Vec<Transaction> {
        &self.transactions
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
            interest_amount
        } else {
            0.0
        }
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

        if amount > self.balance + self.overdraft_limit {
            return Err(WithdrawalError::InsufficientFunds{requested: amount, available: self.balance});
        }

        if amount > self.balance {
            self.balance -= self.overdraft_fee;
            self.transactions.push(Transaction {
                transaction_type: TransactionType::Fee,
                amount: self.overdraft_fee,
                timestamp: Utc::now(),
                description: Some(format!("Overdraft fee")),
            });
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

    fn transfer(&mut self, other: &mut dyn Account, amount: f64, note: Option<&str>) -> Result<f64,TransferError> {
        if amount < 0.0 {
            return Err(TransferError::NegativeAmount(amount));
        }

        let withdraw_note = match note {
            Some(note) => format!("{}. transfer to {}", note, other.get_name()),
            None => format!("transfer to {}", other.get_name())
        };

        let withdrawn_amount = self.withdraw(amount, Some(&withdraw_note)).map_err(|err| match err {
            WithdrawalError::InsufficientFunds { requested, available } => 
                TransferError::InsufficientFunds { requested, available },
            WithdrawalError::NegativeAmount(amt) => TransferError::NegativeAmount(amt),
        })?;

        let deposit_note = match note {
            Some(note) => format!("{}. transfer from {}", note, self.get_name()),
            None => format!("transfer from {}", self.get_name())
        };

        match other.deposit(withdrawn_amount, Some(&deposit_note)) {
            Ok(deposited_amount) => Ok(deposited_amount),
            Err(_) => {
                let _ = self.deposit(withdrawn_amount, None).unwrap();
                Err(TransferError::DepositFailed)
            }
        }
    }
}