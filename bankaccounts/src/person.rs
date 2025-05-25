use crate::Account;

/// Goals for Person struct:
/// 1. Maintain a vector of accounts
/// 2. Track and limit contributions once Roth and Traditional retirement accounts are introduced
/// 3. Maintain an income model that deposits to accounts as money rolls in. Also account for exployer contributions
/// 4. Relate to an expense model (maybe in parent family struct?) and target withdrawals
/// 5. Learn about lifetimes and Box dyns for these generic structs
/// 6. Do I need an Rc< > in the accounts to annotate the owner?

pub struct Person<'a>{
    name: String,
    accounts: Vec<Box<dyn Account + 'a>>,
}

impl<'a> Person<'a> {
    pub fn new(name: &str) -> Self {
        Person {
            name: name.to_string(),
            accounts: Vec::new(),
        }
    }

    pub fn add_account<T: Account + 'a> (&mut self, account: T) {
        self.accounts.push(Box::new(account));
    }

    pub fn list_accounts (&self) {
        println!("Accounts owned by: {}", self.name);
        for acct in &self.accounts {
            println!("Account - {}", acct.get_name());
        }
    }
}