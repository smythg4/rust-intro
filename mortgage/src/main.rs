use std::fmt;
use chrono::{Months,Utc, prelude::*};

#[derive(Clone)]
struct Mortgage {
    origin_date: chrono::DateTime<Utc>,
    principal: f64,
    annual_rate: f64,
    term_years: u32,
    additional_payment: f64,
    historical_payments: Vec<Payment>,
}

#[derive(Clone)]
struct Payment {
    payment_date: chrono::DateTime<Utc>,
    payment_number: u32,
    payment_amount: f64,
    principal_payment: f64,
    interest_payment: f64,
    remaining_principal: f64,
}

struct AmortizationSchedule {
    payments: Vec<Payment>,
    total_interest_paid: f64,
    total_paid: f64,
}

impl Mortgage {
    fn new(origin_date: chrono::DateTime<Utc>, principal: f64, annual_rate: f64, term_years: u32) -> Self {
        Mortgage {
            origin_date,
            principal,
            annual_rate,
            term_years,
            additional_payment: 0.0,
            historical_payments: Vec::new(),
        }
    }

    fn with_additional_payment(mut self, additional_payment: f64) -> Self {
        self.additional_payment = additional_payment;
        self
    }

    fn refinance(mut self, new_rate: f64) -> Self {
        self.annual_rate = new_rate;
        self
    }

    fn monthly_rate(&self) -> f64 {
        self.annual_rate / 100.0 / 12.0
    }

    fn total_payments(&self) -> u32 {
        self.term_years * 12
    }

    fn monthly_payment(&self) -> f64 {
        let r = self.monthly_rate();
        let n = self.total_payments() as f64;

        let monthly_payment = self.principal * r * (1.0+r).powf(n) / ((1.0 + r).powf(n) - 1.0);

        (monthly_payment * 100.0).round() / 100.0
    }

    fn generate_history(&mut self, today: chrono::DateTime<Utc>) {
        let mut payment_date = self.origin_date;
        let mut payment_number = 1;
        let mut remaining_principal = self.principal;

        while payment_date < today {
            let interest_payment = remaining_principal * self.monthly_rate();
            let mut payment_amount = self.monthly_payment();

            if payment_amount > remaining_principal {
                payment_amount = remaining_principal;
            }

            let principal_payment = payment_amount - interest_payment;

            remaining_principal -= principal_payment;

            let payment = Payment {
                payment_date,
                payment_number,
                payment_amount,
                principal_payment,
                interest_payment,
                remaining_principal,
            };

            self.historical_payments.push(payment);

            payment_date = payment_date.checked_add_months(Months::new(1)).unwrap();
            payment_number += 1;
        }
    }

    fn generate_amortization_schedule(&self) -> AmortizationSchedule {
        let mut payments = Vec::new();
        let mut remaining_principal = self.principal;
        let base_monthly_payment = self.monthly_payment();
        let mut payment_number = 1;
        let mut total_interest = 0.0;
        let mut current_date = Utc::now();

        // first apply each of the historical payments
        for payment in &self.historical_payments {
            payment_number += 1;
            current_date = payment.payment_date.clone();
            payments.push(payment.clone());
            remaining_principal = payment.remaining_principal;
            total_interest += payment.interest_payment;
        }
        // add one month to the current date
        current_date = current_date.checked_add_months(Months::new(1)).unwrap();

        // now apply future payments ( to account for any additional payment amount applied after the history was generated )

        while remaining_principal > 0.0 {
            let interest_payment = remaining_principal * self.monthly_rate();

            let mut payment_amount = base_monthly_payment + self.additional_payment;

            if payment_amount > remaining_principal + interest_payment {
                payment_amount = remaining_principal + interest_payment;
            }

            let principal_payment = payment_amount - interest_payment;

            remaining_principal -= principal_payment;

            if remaining_principal < 0.01 {
                remaining_principal = 0.0;
            }

            let payment = Payment {
                payment_date: current_date,
                payment_number,
                payment_amount,
                principal_payment,
                interest_payment,
                remaining_principal,
            };

            payments.push(payment);

            payment_number += 1;
            total_interest += interest_payment;
            current_date = current_date.checked_add_months(Months::new(1)).unwrap();

            if payment_number > 1200 {
                // if something is horribly wrong
                break;
            }
        }

        let total_paid = self.principal + total_interest;

        AmortizationSchedule {
            payments,
            total_interest_paid: total_interest,
            total_paid,
        }
    }

}

impl fmt::Display for AmortizationSchedule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Amortization Schedule")?;
        writeln!(f, "--------------------------------------------------------------------------------")?;
        writeln!(f, "{:>4} | {:>12}  | {:>12} | {:>12} | {:>12} | {:>12}",
            "Pmt#", "Payment Date", "Payment", "Principal", "Interest", "Remaining")?;
        writeln!(f, "--------------------------------------------------------------------------------")?;

        for payment in &self.payments {
            writeln!(f,"{:4} | {}   | ${:11.2} | ${:11.2} | ${:11.2} | ${:11.2}",
            payment.payment_number,
            payment.payment_date.format("%Y-%b-%d"),
            payment.payment_amount,
            payment.principal_payment,
            payment.interest_payment,
            payment.remaining_principal)?;
        }
        writeln!(f, "--------------------------------------------------------------------------------")?;
        writeln!(f, "Total Payments: ${:.2}", self.total_paid)?;
        writeln!(f, "Total Interest Paid: ${:.2}", self.total_interest_paid)?;
        writeln!(f, "Number of Payments: {:.0}", self.payments.len())?;

        Ok(())
    }
}

#[derive(Clone)]
struct Scenario {
    additional_payment: f64,
    total_payments: usize,
    total_interest: f64,
    payoff_date: DateTime<Utc>,
    interest_savings: f64,
    savings_ratio: f64,
}

fn compare_payment(mort: Mortgage, pay_inc: f64) {
    let mut results = Vec::new();

    let mut this_mort = mort;
    let baseline_amort = this_mort.generate_amortization_schedule();
    let baseline_interest = baseline_amort.total_interest_paid;

    for i in 0..=10 {
        let payment = pay_inc * i as f64;
        this_mort = this_mort.with_additional_payment(payment);
        let amort = this_mort.generate_amortization_schedule();
        let payments = amort.payments.len();
        let interest_paid = amort.total_interest_paid;
        let payoff_date = amort.payments.get(payments-1).unwrap().payment_date;
        results.push( Scenario {
            additional_payment: payment,
            total_payments: payments,
            total_interest: interest_paid,
            payoff_date,
            interest_savings: baseline_interest - amort.total_interest_paid,
            savings_ratio: (baseline_interest - amort.total_interest_paid) / payment,
        });
    }
    //results.sort_unstable_by_key(|item| item.savings_ratio as i64);
    for result in results {
        println!("With additional payments of ${:.2}", result.additional_payment);
        println!("   Total Payments: {}", result.total_payments);
        println!("   Total Interest: ${:.2}", result.total_interest);
        println!("   Payoff Date: {}", result.payoff_date.format("%Y-%b-%d"));
        println!("   Interest savings: ${:.2}", result.interest_savings);
        println!("   Savings Ratio: ${:.2} per $1 per month", result.savings_ratio);
    }
    
}

fn main() {
    let origin_date = Utc.with_ymd_and_hms(2023, 8, 1, 0, 0, 0).unwrap();
    let mut mort = Mortgage::new(origin_date, 479000.0, 5.5, 30);
    println!("New Mortgage created on origin date: {}", origin_date.format("%Y-%b-%d"));
    mort.generate_history(Utc::now());

     mort = mort.with_additional_payment(200.0);

     let amort = mort.generate_amortization_schedule();
     println!("{}",amort);

    // compare_payment(mort, 50.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_additional_payments() {
        let origin_date = Utc.with_ymd_and_hms(2023, 8, 1, 0, 0, 0).unwrap();
        let mort1 = Mortgage::new(origin_date, 479000.0, 5.5, 30);
        

        let amort1 = mort1.generate_amortization_schedule();
        let payment1 = amort1.total_paid;

        let mort2 = mort1.with_additional_payment(200.0);
        let amort2 = mort2.generate_amortization_schedule();


        let payment2 = amort2.total_paid;

        assert!(payment1 > payment2);
    }

    #[test]
    fn test_refinance_lower() {
        let origin_date = Utc.with_ymd_and_hms(2023, 8, 1, 0, 0, 0).unwrap();
        let mort1 = Mortgage::new(origin_date, 479000.0, 5.5, 30);
        

        let amort1 = mort1.generate_amortization_schedule();
        let payment1 = amort1.total_paid;

        let mort2 = mort1.refinance(2.5);
        let amort2 = mort2.generate_amortization_schedule();


        let payment2 = amort2.total_paid;

        assert!(payment1 > payment2);
    }

    #[test]
    fn test_refinance_higher() {
        let origin_date = Utc.with_ymd_and_hms(2023, 8, 1, 0, 0, 0).unwrap();
        let mort1 = Mortgage::new(origin_date, 479000.0, 5.5, 30);
        

        let amort1 = mort1.generate_amortization_schedule();
        let payment1 = amort1.total_paid;

        let mort2 = mort1.refinance(7.5);
        let amort2 = mort2.generate_amortization_schedule();


        let payment2 = amort2.total_paid;

        assert!(payment1 < payment2);
    }
}