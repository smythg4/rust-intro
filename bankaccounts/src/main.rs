mod errors;
mod accounts;
mod tests;
mod person;
use accounts::*;
use person::Person;

fn main() -> serde_json::Result<()> {
    let checking = CheckingSavingsAccount::new("Test Checking", 1000.0, 0.5, 0.0, 0.0);
    println!("New account created - {}", checking.get_name());
    let brokerage = BrokerageAccount::new("Test Brokerage", 10000.0, 1.1);
    let mut person: Person = Person::new("Jimmy");
    person.add_account(checking);
    person.add_account(brokerage);
    person.list_accounts();

    return Ok(())
}
