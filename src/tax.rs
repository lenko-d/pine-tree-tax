extern crate chrono;
extern crate csv;

use self::chrono::prelude::*;
use self::chrono::Duration;
use std::error::Error;
use std::fs::File;

use serde::{Deserialize, Serialize};

use account::Account;
use account::Deposit;

const WALLET_EXTERNAL: &str = "External";
const WALLET_NA: &str = "N/A";

const CAPITAL_GAIN_TYPE_LONG: &str = "long";
const CAPITAL_GAIN_TYPE_SHORT: &str = "short";

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
struct TaxEvent {
    quantity: f64,
    asset: String,
    buy_date: DateTime<Utc>,
    sell_date: DateTime<Utc>,
    cost_basis: f64,
    proceeds: f64,
    gain: f64,
}

pub fn calculate_capital_gains(file_path: &str, output_file: &str, output_positions: u64) {
    let mut transactions = read_transactions(file_path).expect("Can't read transactions");
    transactions.sort_by(|t1, t2| t1.datetime.cmp(&t2.datetime));

    let mut accounts = hashmap! {
        "USD".to_string() => Account::new("USD".to_string(), 100000.0),
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

    let mut tax_events: Vec<TaxEvent> = Vec::new();

    for transaction in transactions.iter() {
        if transaction.origin_asset == transaction.destination_asset
            && transaction.origin_wallet != WALLET_EXTERNAL
            && transaction.destination_wallet != WALLET_EXTERNAL
        {
            continue;
        }
        let mut deposits: Vec<Deposit> = Vec::new();
        if transaction.origin_wallet != WALLET_NA {
            if let Some(account) = accounts.get_mut(&transaction.origin_asset) {
                deposits = account.withdraw(transaction.datetime, transaction.origin_quantity)
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

    if output_positions > 0 {
        dbg!(accounts);
    }

    //dbg!(&tax_events);

    save_to_file(
        &tax_events,
        &(output_file.to_owned() + "_long_gains.csv"),
        CAPITAL_GAIN_TYPE_LONG,
    );
    save_to_file(
        &tax_events,
        &(output_file.to_owned() + "_short_gains.csv"),
        CAPITAL_GAIN_TYPE_SHORT,
    );
}

fn save_to_file(tax_events: &Vec<TaxEvent>, out_file: &str, filter_by: &str) {
    let file = File::create(out_file)
        .ok()
        .expect("Unable to create output file.");

    let mut writer = csv::Writer::from_writer(file);

    for tax_event in tax_events.iter() {
        let time_elapsed = tax_event
            .sell_date
            .signed_duration_since(tax_event.buy_date);

        if (time_elapsed >= Duration::days(365) && filter_by == CAPITAL_GAIN_TYPE_LONG)
            || (time_elapsed < Duration::days(365) && filter_by == CAPITAL_GAIN_TYPE_SHORT)
        {
            writer
                .serialize(tax_event)
                .ok()
                .expect("Unable to write to output file.");
        }
    }
}

pub fn read_transactions(file_path: &str) -> Result<Vec<Transaction>, Box<Error>> {
    let file = File::open(file_path)?;
    let mut reader = csv::Reader::from_reader(file);
    let mut transactions = Vec::new();
    for transaction in reader.deserialize() {
        transactions.push(transaction?);
    }
    Ok(transactions)
}

fn round_to_dollars(num: f64) -> f64 {
    return (100.0 * num).round() / 100.0;
}
