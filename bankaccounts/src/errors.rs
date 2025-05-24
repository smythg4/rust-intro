
use std::fmt;

#[derive(Debug)]
pub enum DepositError {
    NegativeAmount(f64),
}

#[derive(Debug)]
pub enum WithdrawalError {
    InsufficientFunds { requested: f64, available: f64 },
    NegativeAmount(f64),
}

#[derive(Debug)]
pub enum TransferError {
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