#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    fn test_liquidate() {
        let mut acct = BrokerageAccount::new("Gonna Sell it all!", 10000.0, 1.5);
        acct.buy(100.0,50.0, AssetClass::Equity).unwrap();
        let cash_left = acct.get_cash_balance()*0.999;
        acct.buy(10.0, cash_left/10.0, AssetClass::Bond).unwrap();
        acct.validate_balance().unwrap();

        let (c, e, b) = acct.get_asset_alloc();
        assert!(c > 0.0 && e > 0.0 && b > 0.0);

        // make sure you got some cash from the sale
        let proceeds = acct.liquidate().unwrap_or(0.0);
        assert!(proceeds > 0.0);

        // make sure it's effectively all cash now
        let (c, e, b) = acct.get_asset_alloc();
        assert!(c > 0.95);
        assert!(e < 0.01 && b < 0.01);

        println!("New Balances - Cash: ${:.2}, Total ${:.2}", acct.get_cash_balance(), acct.get_balance());
        acct.validate_balance().unwrap();       
    }   

    #[test]
    fn test_hard_rebalance() {
        let mut acct = BrokerageAccount::new("Rebalancing Act", 1000.0, 1.1);

        //initially should be all cash (1.0,0.0,0.0)
        assert_eq!((1.0,0.0,0.0), acct.get_asset_alloc());

        // accrue interest, should have no impact on asset allocation.
        acct.accrue();
        assert_eq!((1.0,0.0,0.0), acct.get_asset_alloc());

        // buy some stocks to affect allocation
        acct.buy(10.0, 5.0, AssetClass::Equity).unwrap();
        let (_,e,b) = acct.get_asset_alloc();
        assert!(e > 0.0);
        assert_eq!(b,0.0);

        // buy some bonds to affect allocation
        acct.buy(10.0, 5.0, AssetClass::Bond).unwrap();
        let (_,_,b) = acct.get_asset_alloc();
        assert!(b > 0.0);

        // trigger rebalance and check allocations
        let (c,e,b) = acct.hard_rebalance(0.70, 0.10).unwrap();
        println!("Cash: {:.2}%, Equities: {:.2}%, Bonds: {:.2}%", c*100.0, e*100.0, b*100.0);

        assert!( (c - 0.10).abs() < 0.01 );
        assert!( (e - 0.70).abs() < 0.05 );
        assert!( (b - 0.20).abs() < 0.05 );
    }

    #[test]
    fn test_soft_rebalance() {
        let mut acct = BrokerageAccount::new("Rebalancing Act", 1000.0, 1.1);

        //initially should be all cash (1.0,0.0,0.0)
        assert_eq!((1.0,0.0,0.0), acct.get_asset_alloc());

        // accrue interest, should have no impact on asset allocation.
        acct.accrue();
        assert_eq!((1.0,0.0,0.0), acct.get_asset_alloc());

        // buy some stocks to affect allocation
        acct.buy(10.0, 5.0, AssetClass::Equity).unwrap();
        let (_,e,b) = acct.get_asset_alloc();
        assert!(e > 0.0);
        assert_eq!(b,0.0);

        // buy some bonds to affect allocation
        acct.buy(10.0, 5.0, AssetClass::Bond).unwrap();
        let (_,_,b) = acct.get_asset_alloc();
        assert!(b > 0.0);

        // trigger rebalance and check allocations
        let (c,e,b) = acct.soft_rebalance(0.5, 0.10).unwrap();
        println!("Cash: {:.2}%, Equities: {:.2}%, Bonds: {:.2}%", c*100.0, e*100.0, b*100.0);

        assert!( (c - 0.10).abs() < 0.01 );
        assert!( (e - 0.50).abs() < 0.05 );
        assert!( (b - 0.40).abs() < 0.05 );
    }

    #[test]
    fn test_sell_from_brokerage_without_enough_cash() {
        let starting_bal = 1000.0;
        let mut acct = BrokerageAccount::new("Brokerage Test", starting_bal, 1.0);
        let shares = 20.0;
        let price = 25.0;
        println!("Cash Bal: ${:.2}. Total Bal: ${:.2}", acct.get_cash_balance(), acct.get_balance());

        // buy some stocks. total balance shouldn't change. equity value should increase. cash value should decrease.
        match acct.buy(shares, price, AssetClass::Equity) {
            Ok(amount) => println!("Success buying {} shares at {}. Amount: {}", shares, price, amount),
            Err(e) => eprintln!("{}",e),
        }
        println!("Cash Bal: ${:.2}. Total Bal: ${:.2}", acct.get_cash_balance(), acct.get_balance());
        //cash value decreases?
        assert_eq!(acct.get_cash_balance(), 1000.0-shares*price);
        //equity value equals purchase price?
        let equity_val: f64 = acct.get_assets_of_type(AssetClass::Equity).iter().map(|a|a.get_value()).sum();
        assert_eq!(equity_val, shares*price);
        //cost basis should equal value
        let equity_cb: f64 = acct.get_assets_of_type(AssetClass::Equity).iter().map(|a|a.get_cost_basis()).sum();
        assert_eq!(equity_cb, equity_val);
        //validate balance
        acct.validate_balance().unwrap();

        //withdraw an amount greater than known cash balance. should trigger sale of stocks, but no taxable event
        match acct.withdraw(550.00, Some("should trigger sale of stocks")) {
            Ok(amount) => println!("Success withdrawing {}", amount),
            Err(e) => eprintln!("{}",e),
        }

        println!("Cash Bal: ${:.2}. Total Bal: ${:.2}", acct.get_cash_balance(), acct.get_balance());
        // equity value should go down (converted equity to cash)
        let equity_val_af: f64 = acct.get_assets_of_type(AssetClass::Equity).iter().map(|a|a.get_value()).sum();
        assert!(equity_val_af < equity_val);
        //total balance should go down by precisely amount withdrawn
        assert_eq!(acct.get_balance(), starting_bal-550.0);
        // should have 0 tax transactions
        let num_tax_transactions = acct.generate_transactions().iter().filter(|t| t.transaction_type == TransactionType::Tax).count();
        assert_eq!(0, num_tax_transactions);
        //validate balances and don't panic!
        acct.validate_balance().unwrap();

        // accrue some interest and gains. now balance should exceed cost basis
        acct.accrue();
        acct.accrue();
        // ensure value exceeds cost basis
        let equity_val: f64 = acct.get_assets_of_type(AssetClass::Equity).iter().map(|a| a.get_value()).sum();
        let equity_cb: f64 = acct.get_assets_of_type(AssetClass::Equity).iter().map(|a| a.get_cost_basis()).sum();
        assert!(equity_val > equity_cb);
        // validate balances and don't panic!
        acct.validate_balance().unwrap();

        // sell more stocks. this time it should trigger a taxable event (since we have unrealized gains)
        match acct.withdraw(100.00, Some("should trigger sale of stocks")) {
            Ok(amount) => println!("Success withdrawing {}", amount),
            Err(e) => eprintln!("Failed to withdraw {} - {}",100.0,e),
        }
        println!("Cash Bal: ${:.2}. Total Bal: ${:.2}", acct.get_cash_balance(), acct.get_balance());

        //check that it actually created a tax transaction
        let num_tax_transactions = acct.generate_transactions().iter()
                                        .filter(|t|t.transaction_type==TransactionType::Tax)
                                        .count();
        assert_eq!(num_tax_transactions,1);
        // validate balances and don't panic!
        acct.validate_balance().unwrap();

        // try to withdraw more than we have. transaction should fail, but assets should be liquidated
        let bal_bf = acct.get_balance();
        let cash_bf = acct.get_cash_balance();
        let tax_bf: f64 = acct.generate_transactions().iter()
                    .filter(|t|t.transaction_type==TransactionType::Tax)
                    .map(|t| t.amount)
                    .sum();
                
        let result = match acct.withdraw(1000.00, Some("should trigger sale of stocks")) {
            Ok(amount) => {
                println!("Success withdrawing {}", amount);
                Ok(())},
            Err(e) => {
                eprintln!("Failed to withdraw {} - {}",1000.0,e);
            Err("FAIL".to_string())},
        };
        println!("Cash Bal: ${:.2}. Total Bal: ${:.2}", acct.get_cash_balance(), acct.get_balance());

        // make sure it's all cash now
        assert_eq!(acct.get_balance(), acct.get_cash_balance());

        let tax_af: f64 = acct.generate_transactions().iter()
                    .filter(|t|t.transaction_type==TransactionType::Tax)
                    .map(|t| t.amount)
                    .sum();
        // make sure the total balance only changed by the amount of taxes paid
        assert_eq!(bal_bf, acct.get_balance()+(tax_af-tax_bf));
        // make sure the cash balance changed
        assert_ne!(cash_bf, acct.get_cash_balance());

        // make sure the withdrawal return an error
        assert_eq!(Err("FAIL".to_string()), result);

    }

    #[test]
    fn test_validate_balances() {
        for j in 0..20 {
            let eq_ratio = (j as f64) / 20.0;

            println!("TESTING with {:.2}% equity.", eq_ratio*100.0);

            let mut my_acct = CheckingSavingsAccount::new("Stephen's Account", 12000.05, 0.5, 500.0, 25.0);
            let mut your_acct = BrokerageAccount::new("Ashley's Account", 15000.00, 1.1);


            for i in 1..=50 {

                println!("Validating balances at start of loop (main)");
                your_acct.validate_balance().expect("validation failed");

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

                println!("   Attempting to transfer $500 from {} to {}", your_acct.get_name(), my_acct.get_name());
                match your_acct.transfer(&mut my_acct, 500.0, Some("moving money!")) {
                    Ok(amount) => println!("      Transfer successful for ${:.2}", amount),
                    Err(e) => eprintln!("   {}", e),
                }

                println!("   Attempting to transfer $750 from {} to {}", my_acct.get_name(), your_acct.get_name());
                match my_acct.transfer(&mut your_acct, 750.0, Some("still moving money!")) {
                    Ok(amount) => println!("      Transfer successful for ${:.2}", amount),
                    Err(e) => eprintln!("   {}", e),
                }

                println!("Validating balances at end of loop (main - pre stock sale)");
                your_acct.validate_balance().expect("validation failed");

                println!("   Attempting to sell $1000.0 worth of stock from {}", your_acct.get_name());
                match your_acct.sell(1000.0, AssetClass::Bond) {
                    Ok(amount) => println!("     Stock sale successful for ${:2}", amount),
                    Err(e) => eprintln!("--    {}", e),
                }

                println!("Validating balances at end of loop (main - pre balance)");
                your_acct.validate_balance().expect("validation failed");

                println!("   Attempting to soft rebalance {} ", your_acct.get_name());
                let (cash, equity, bond) = your_acct.get_asset_alloc();
                println!("Asset allocation - before (Cash, Equity, Bond) = {:.2}, {:.2}, {:.2}", cash, equity, bond);

                let (cash, equity, bond) = match your_acct.soft_rebalance(eq_ratio, 0.15) {
                    Ok((c, e, b)) => (c, e, b),
                    Err(e) => { eprintln!(" Issue with reblance   {}", e);
                                                (0.0,0.0,0.0)},
                };
                //let (cash, equity, bond) = your_acct.get_asset_alloc();
                println!("Asset allocation - after (Cash, Equity, Bond) = {:.2}, {:.2}, {:.2}", cash, equity, bond);
                
                println!("Validating balances at end of loop (main - pre post balance buy)");
                your_acct.validate_balance().expect("validation failed");

                println!("   Attempting to buy $500.0 worth of stock from {}", your_acct.get_name());
                match your_acct.buy(5.0, 100.0, AssetClass::Equity) {
                    Ok(amount) => println!("     Stock purchase successful for ${:2}", amount),
                    Err(e) => eprintln!("    {}", e),
                };
                println!("   Attempting to buy $500.0 worth of bonds from {}", your_acct.get_name());
                match your_acct.buy(5.0, 100.0, AssetClass::Bond) {
                    Ok(amount) => println!("     Bond purchase successful for ${:2}", amount),
                    Err(e) => eprintln!("    {}", e),
                };

                println!("Validating balances at end of loop (main - pre accrue)");
                your_acct.validate_balance().expect("validation failed");

                my_acct.accrue();
                your_acct.accrue();

                println!("   Attempting to hard rebalance {} ", your_acct.get_name());
                let (cash, equity, bond) = your_acct.get_asset_alloc();
                println!("Asset allocation - before (Cash, Equity, Bond) = {:.2}, {:.2}, {:.2}", cash, equity, bond);

                let (cash, equity, bond) = match your_acct.hard_rebalance(eq_ratio, 0.15) {
                    Ok((c, e, b)) => (c, e, b),
                    Err(e) => { eprintln!(" Issue with reblance   {}", e);
                                                (0.0,0.0,0.0)},
                };
                //let (cash, equity, bond) = your_acct.get_asset_alloc();
                println!("Asset allocation - after (Cash, Equity, Bond) = {:.2}, {:.2}, {:.2}", cash, equity, bond);

                println!("Validating balances at end of loop (main - post accrue)");
                your_acct.validate_balance().expect("validation failed");
            }

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
        }
    }

    #[test]
    #[should_panic]
    fn test_checking_overdraft() {
        let mut acct = CheckingSavingsAccount::new("checking", 100.0, 0.5, 0.0, 10.0);

        acct.withdraw(1500.0, Some("This should panic!")).unwrap();
    }

    #[test]
    fn test_checking_interest() {
        let mut acct = CheckingSavingsAccount::new("checking", 100.0, 0.5, 0.0, 10.0);

        acct.accrue();

        assert_eq!(acct.get_balance(), 100.5);
    }
}