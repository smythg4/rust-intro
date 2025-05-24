mod errors;
mod accounts;
use accounts::*;


fn main() -> serde_json::Result<()> {
    let mut my_acct = CheckingSavingsAccount::new("Stephen's Account", 12000.05, 0.5, 500.0, 25.0);
    let mut your_acct = BrokerageAccount::new("Ashley's Account", 15000.00, 1.1);

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

        // println!("   Attempting to transfer $1,000 from {} to {}", your_acct.get_name(), my_acct.get_name());
        // match your_acct.transfer(&mut my_acct, 500.0, Some("moving money!")) {
        //     Ok(amount) => println!("      Transfer successful for ${:.2}", amount),
        //     Err(e) => eprintln!("   {}", e),
        // }

        println!("   Attempting to transfer $750 from {} to {}", my_acct.get_name(), your_acct.get_name());
        match my_acct.transfer(&mut your_acct, 750.0, Some("still moving money!")) {
            Ok(amount) => println!("      Transfer successful for ${:.2}", amount),
            Err(e) => eprintln!("   {}", e),
        }

        let (cash, equity, bond) = your_acct.get_asset_alloc();
        println!("Asset allocation (Cash, Equity, Bond) = {:.2}, {:.2}, {:.2}", cash, equity, bond);
        let (cash, equity, bond) = your_acct.soft_rebalance(0.70, 0.15).expect("problem rebalancing");
        println!("Asset allocation (Cash, Equity, Bond) = {:.2}, {:.2}, {:.2}", cash, equity, bond);
        
        my_acct.accrue();
        your_acct.accrue();
    }

    my_acct.validate_balance().expect("balance validation failed");
    your_acct.validate_balance().expect("balance validation failed");

    let my_statement = your_acct.generate_statement(None, None);
    println!("{}",my_statement);

    // my_acct.reset();
    // your_acct.reset();

    // accounts.push(your_acct);
    // accounts.push(my_acct);
    // match BankAccount::write_accounts_json_to_file(&accounts, filepath) {
    //     Ok(_) => println!("Successfully wrote JSON to file: {}", filepath),
    //     Err(e) => eprintln!("File write error: {}", e)
    // };

    Ok(())
}
