extern crate clap;

#[macro_use]
extern crate maplit;

#[macro_use]
extern crate lazy_static;

extern crate serde;
extern crate chrono;


mod account;
mod conversions;
mod tax;

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::hash::Hash;
use std::io;
use chrono::Duration;
use clap::{App, Arg, ArgMatches};
use conversions::*;
use tax::*;
use account::{Account, TAX_ACCOUNTING_METHOD_LIFO};

fn read_arguments<'a>() -> ArgMatches<'a> {
    App::new("Pine Tree Tax")
        .version("0.01")
        .arg(
            Arg::with_name("INPUT_FILE")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("tax-accounting-method")
                .help("tax accounting method: FIFO (First-In-First-Out), LIFO (Last-In-First-Out) or HIFO (High-In-First-Out)")
                .short("m")
                .required(false)
                .takes_value(true)
                .value_name("LIFO_FIFO_OR_HIFO"),
        )
        .arg(
            Arg::with_name("convert-from")
                .short("c")
                .long("convert-from")
                .required(false)
                .help("Convert from another format.")
                .takes_value(true)
                .value_name("FILE_FORMAT"),
        )
        .arg(
            Arg::with_name("a")
                .short("a")
                .long("save-accounts")
                .help("Save the accounts in a .csv file."),
        )
        .arg(
            Arg::with_name("e")
                .short("e")
                .long("save-transactions-tax-events")
                .help("Save the transactions and the tax events in a .csv file."),
        )
        .arg(
            Arg::with_name("output-file")
                .help("Output file.")
                .required(false)
                .value_name("OUTPUT_FILE"),
        )
        .get_matches()
}

fn main() {
    let cli_args = read_arguments();
    let input_file = cli_args.value_of("INPUT_FILE").unwrap();

    if let Some(convert_from_another_format) = cli_args.value_of("convert-from") {
        let output_file = cli_args
            .value_of("output-file")
            .unwrap_or("_transactions.csv");

        if convert_from_another_format == "kraken" {
            process_kraken_transactions(
                input_file,
                &(convert_from_another_format.to_owned() + output_file),
            );
        }

        if convert_from_another_format == "bittrex" {
            process_bittrex_transactions(
                input_file,
                &(convert_from_another_format.to_owned() + output_file),
            );
        }
    } else {
        let tax_accounting_method = cli_args.value_of("tax-accounting-method").unwrap_or(TAX_ACCOUNTING_METHOD_LIFO);
        let output_file = cli_args.value_of("output-file").unwrap_or("transactions");
        let output_accounts = cli_args.occurrences_of("a");
        let output_transactions_and_tax_events = cli_args.occurrences_of("e");

        let mut transactions = read_transactions(input_file).expect("read transactions");
        let (tax_events, accounts) = calculate_capital_gains(&mut transactions, tax_accounting_method);
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

        if output_transactions_and_tax_events > 0 {
            save_transactions_and_tax_events_to_file(&transactions, &tax_events, &accounts, "transactions_and_tax_events.csv").expect("save transactions and tax events file");
        }

        if output_accounts > 0 {
            save_accounts_to_file(accounts).expect("save accounts file");
        }
    }
}

pub fn read_transactions(file_path: &str) -> Result<Vec<Transaction>, Box<Error>> {
    let file = File::open(file_path)?;
    let mut reader = csv::Reader::from_reader(file);
    let mut transactions = vec![];
    for transaction in reader.deserialize() {
        transactions.push(transaction?);
    }
    Ok(transactions)
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

fn save_accounts_to_file(accounts: HashMap<String,Account>) -> Result<(), Box<dyn Error>>{
    let out_file = "accounts.csv";
    let file = File::create(out_file)
        .ok()
        .expect("Unable to create output file.");

    let mut wtr = csv::Writer::from_writer(file);

    wtr.write_record(&["Account", "Balance", "Deposit datetime", "Deposit USD value", "Deposit quantity", "Deposit remaining quantity"])?;
    for (_currency, mut acct) in accounts {
        acct.deposits.sort_by(|a,b| a.datetime.cmp(&b.datetime));
        for dep in acct.deposits {
            wtr.write_record(vec![acct.name.clone(), acct.balance.to_string(), dep.datetime.to_string(), dep.usd_value.to_string(), dep.quantity.to_string(), dep.remaining_quantity.to_string()])?;
        }
    }

    wtr.flush()?;
    Ok(())
}

fn save_transactions_and_tax_events_to_file(transactions: &Vec<Transaction>, tax_events: &Vec<TaxEvent>, accounts: &HashMap<String, Account>, out_file: &str) -> Result<(), Box<dyn Error>>{
    let file = File::create(out_file)
        .ok()
        .expect("the output file to be created.");

    let mut wtr = csv::Writer::from_writer(file);

    wtr.write_record(&["id","datetime","origin_wallet","origin_asset","origin_quantity","destination_wallet","destination_asset","destination_quantity","remaining_quantity","usd_value","usd_fee","buy_date","cost_basis","gain"])?;
    for transaction in transactions {
        let acct = accounts.get(transaction.destination_asset.as_str()).unwrap();
        let mut remaining_quantity= transaction.destination_quantity;
        for dep in &acct.deposits {
            if dep.datetime == transaction.datetime{
                remaining_quantity = dep.remaining_quantity;
            }
        }

        let mut buy_date = "".to_string();
        let mut cost_basis = 0.0;
        let mut gain = 0.0;
        for tax_event in tax_events {
            if tax_event.sell_date == transaction.datetime {
                buy_date = tax_event.buy_date.to_string();
                cost_basis = tax_event.cost_basis;
                gain = tax_event.gain;
            }
        }
        wtr.write_record(vec![transaction.id.clone(), transaction.datetime.to_string(), transaction.origin_wallet.clone(),transaction.origin_asset.clone(),
                              transaction.origin_quantity.to_string(),
                              transaction.destination_wallet.clone(), transaction.destination_asset.clone(), transaction.destination_quantity.to_string(),
                              remaining_quantity.to_string(),
                              transaction.usd_value.to_string(),
                              transaction.usd_fee.unwrap_or_default().to_string(),buy_date, cost_basis.to_string(), gain.to_string()])?;
    }

    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Utc};
    use account::{TAX_ACCOUNTING_METHOD_FIFO, TAX_ACCOUNTING_METHOD_HIFO};
    use super::*;

    lazy_static!(
        static ref DATE_TIME0 :DateTime<Utc> = Utc.with_ymd_and_hms(2017, 1, 1, 0, 1, 1).unwrap();
        static ref DATE_TIME1 :DateTime<Utc> = Utc.with_ymd_and_hms(2017, 2, 1, 0, 1, 1).unwrap();
        static ref DATE_TIME2 :DateTime<Utc> = Utc.with_ymd_and_hms(2017, 3, 1, 0, 1, 1).unwrap();
    );

    fn test_transactions_eth_buy2_sell1() -> Vec<Transaction> {
        let t0 = Transaction{
            id: "0".to_string(),
            datetime: *DATE_TIME0,
            origin_wallet: WALLET_BANK.to_string(),
            origin_asset: "USD".to_string(),
            origin_quantity: 2250.0,
            destination_wallet: WALLET_KRAKEN.to_string(),
            destination_asset: "ETH".to_string(),
            destination_quantity: 1.0,
            usd_value: 2250.0,
            usd_fee: None
        };
        let t1 = Transaction{
            id: "1".to_string(),
            datetime: *DATE_TIME1,
            origin_wallet: WALLET_BANK.to_string(),
            origin_asset: "USD".to_string(),
            origin_quantity: 2500.0,
            destination_wallet: WALLET_KRAKEN.to_string(),
            destination_asset: "ETH".to_string(),
            destination_quantity: 1.0,
            usd_value: 2500.0,
            usd_fee: None
        };
        let t2 = Transaction{
            id: "2".to_string(),
            datetime: *DATE_TIME2,
            origin_wallet: WALLET_KRAKEN.to_string(),
            origin_asset: "ETH".to_string(),
            origin_quantity: 1.0,
            destination_wallet: WALLET_BANK.to_string(),
            destination_asset: "USD".to_string(),
            destination_quantity: 3000.0,
            usd_value: 3000.0,
            usd_fee: None
        };

        vec![t0,t1,t2]
    }

    #[test]
    fn fifo_accounting_gains() {
        let (tax_events, accounts) = calculate_capital_gains(&mut test_transactions_eth_buy2_sell1(), TAX_ACCOUNTING_METHOD_FIFO);
        assert_eq!(tax_events.get(0).unwrap().gain, 750.0);
    }

    #[test]
    fn lifo_accounting_gains() {
        let (tax_events, accounts) = calculate_capital_gains(&mut test_transactions_eth_buy2_sell1(), TAX_ACCOUNTING_METHOD_LIFO);
        assert_eq!(tax_events.get(0).unwrap().gain, 500.0);
    }

    #[test]
    fn hifo_accounting_gains() {
        let (tax_events, accounts) = calculate_capital_gains(&mut test_transactions_eth_buy2_sell1(), TAX_ACCOUNTING_METHOD_HIFO);
        assert_eq!(tax_events.get(0).unwrap().gain, 500.0);
    }

    #[test]
    fn save_transactions_and_tax_events() {
        let mut transactions = test_transactions_eth_buy2_sell1();
        let (tax_events, accounts) = calculate_capital_gains(&mut transactions, TAX_ACCOUNTING_METHOD_FIFO);

        save_transactions_and_tax_events_to_file(&transactions, &tax_events, &accounts, "transactions_and_tax_events_test.csv").expect("the file to be saved");
    }
}
