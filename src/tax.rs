extern crate chrono;
extern crate csv;

use std::collections::HashMap;
use self::chrono::prelude::*;
use self::chrono::Duration;
use std::error::Error;
use std::fs::File;

use serde::{Deserialize, Serialize};

use account::Account;
use account::Deposit;

const WALLET_EXTERNAL: &str = "External";
const WALLET_NA: &str = "N/A";

pub const CAPITAL_GAIN_TYPE_LONG: &str = "long";
pub const CAPITAL_GAIN_TYPE_SHORT: &str = "short";

#[derive(Debug, Deserialize, Serialize)]
pub struct Transaction {
    pub id: String,
    pub datetime: DateTime<Utc>,
    pub origin_wallet: String,
    pub origin_asset: String,
    pub origin_quantity: f64,
    pub destination_wallet: String,
    pub destination_asset: String,
    pub destination_quantity: f64,
    pub usd_value: f64,
    pub usd_fee: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct TaxEvent {
    pub quantity: f64,
    pub asset: String,
    pub buy_date: DateTime<Utc>,
    pub sell_date: DateTime<Utc>,
    pub cost_basis: f64,
    pub proceeds: f64,
    pub gain: f64,
}

pub fn calculate_capital_gains(mut transactions: Vec<Transaction>, tax_accounting_method: &str, output_positions: u64) -> (Vec<TaxEvent>, Option<HashMap<String,Account>>) {
    transactions.sort_by(|t1, t2| t1.datetime.cmp(&t2.datetime));

    let mut accounts = hashmap! {
        "USD".to_string() => Account::new("USD".to_string(), 100000000.0),
        "BTC".to_string() => Account::new("BTC".to_string(), 0.0),
        "ETH".to_string() => Account::new("ETH".to_string(), 0.0),
        "BCH".to_string() =>  Account::new("BCH".to_string(), 0.0),
        "LTC".to_string() =>  Account::new("LTC".to_string(), 0.0),
        "XRP".to_string() =>  Account::new("XRP".to_string(), 0.0),
        "XBT".to_string() =>  Account::new("XBT".to_string(), 0.0),
        "XMR".to_string() =>  Account::new("XMR".to_string(), 0.0),
        "ZEC".to_string() =>  Account::new("ZEC".to_string(), 0.0),
        "ADA".to_string() =>  Account::new("ADA".to_string(), 0.0),
        "BITB".to_string() =>  Account::new("BITB".to_string(), 0.0),
        "XZC".to_string() =>  Account::new("XZC".to_string(), 0.0),
    };

    let mut tax_events: Vec<TaxEvent> = vec![];

    for transaction in transactions.iter() {
        if transaction.origin_asset == transaction.destination_asset
            && transaction.origin_wallet != WALLET_EXTERNAL
            && transaction.destination_wallet != WALLET_EXTERNAL
        {
            continue;
        }
        let mut deposits: Vec<Deposit> = vec![];
        if transaction.origin_wallet != WALLET_NA {
            if let Some(account) = accounts.get_mut(&transaction.origin_asset) {
                deposits = account.withdraw(transaction.datetime, transaction.origin_quantity, tax_accounting_method)
            }
        }

        accounts
            .get_mut(&transaction.destination_asset)
            .unwrap()
            .deposit(
                transaction.datetime,
                transaction.destination_quantity,
                transaction.usd_value,
            );

        if transaction.origin_asset != "USD" && deposits.len() > 0 {
            for deposit in deposits.iter() {
                let proceeds = round_to_dollars(
                    transaction.usd_value * (deposit.quantity / transaction.origin_quantity),
                );
                let cost_basis = round_to_dollars(deposit.usd_value);
                tax_events.push(TaxEvent {
                    quantity: deposit.quantity,
                    asset: transaction.origin_asset.clone(),
                    buy_date: deposit.datetime,
                    sell_date: transaction.datetime,
                    cost_basis,
                    proceeds,
                    gain: round_to_dollars(proceeds - cost_basis),
                });
            }
        }
    }

    return if output_positions > 0 {
        (tax_events, Some(accounts))
    } else {
        (tax_events, None)
    }
}

fn round_to_dollars(num: f64) -> f64 {
    return (100.0 * num).round() / 100.0;
}
