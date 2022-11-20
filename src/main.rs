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

use std::error::Error;
use std::fs::File;
use chrono::Duration;
use clap::{App, Arg, ArgMatches};
use conversions::*;
use tax::*;
use account::{TAX_ACCOUNTING_METHOD_LIFO};

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
                .help("tax accounting method: FIFO (First-In-First-Out) or LIFO (Last-In-First-Out)")
                .short("a")
                .required(false)
                .takes_value(true)
                .value_name("LIFO_OR_FIFO"),
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
            Arg::with_name("p")
                .short("p")
                .help("Output the positions in the accounts."),
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
        let output_positions = cli_args.occurrences_of("p");

        let mut transactions = read_transactions(input_file).expect("Can't read transactions");
        let tax_events = calculate_capital_gains(transactions, tax_accounting_method,  output_positions);
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
    let mut transactions = vec![];
    for transaction in reader.deserialize() {
        transactions.push(transaction?);
    }
    Ok(transactions)
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
    use account::TAX_ACCOUNTING_METHOD_FIFO;
    use super::*;

    lazy_static!(
        static ref DATE_TIME0 :DateTime<Utc> = Utc.with_ymd_and_hms(2017, 1, 1, 0, 1, 1).unwrap();
        static ref DATE_TIME1 :DateTime<Utc> = Utc.with_ymd_and_hms(2017, 2, 1, 0, 1, 1).unwrap();
        static ref DATE_TIME2 :DateTime<Utc> = Utc.with_ymd_and_hms(2017, 3, 1, 0, 1, 1).unwrap();
    );

    #[test]
    fn fifo_accounting_gains() {
        let t0 = Transaction{
            id: "0".to_string(),
            datetime: *DATE_TIME0,
            origin_wallet: "Bank".to_string(),
            origin_asset: "USD".to_string(),
            origin_quantity: 7000.0,
            destination_wallet: "Coinbase".to_string(),
            destination_asset: "BTC".to_string(),
            destination_quantity: 10.0,
            usd_value: 7000.0,
            usd_fee: None
        };
        let t1 = Transaction{
            id: "1".to_string(),
            datetime: *DATE_TIME1,
            origin_wallet: "External".to_string(),
            origin_asset: "BTC".to_string(),
            origin_quantity: 1.0,
            destination_wallet: "Coinbase".to_string(),
            destination_asset: "ETH".to_string(),
            destination_quantity: 30.0,
            usd_value: 3000.0,
            usd_fee: None
        };
        let t2 = Transaction{
            id: "2".to_string(),
            datetime: *DATE_TIME2,
            origin_wallet: "Conbase".to_string(),
            origin_asset: "ETH".to_string(),
            origin_quantity: 30.0,
            destination_wallet: "Coinbase".to_string(),
            destination_asset: "ADA".to_string(),
            destination_quantity: 10000.0,
            usd_value: 6000.0,
            usd_fee: None
        };

        let transactions = vec![t0,t1,t2];
        let tax_events = calculate_capital_gains(transactions, TAX_ACCOUNTING_METHOD_FIFO,  0);

        assert_eq!(tax_events.get(0).unwrap().gain, 2300.0);
        assert_eq!(tax_events.get(1).unwrap().gain, 3000.0);
    }

    #[test]
    fn lifo_accounting_gains() {
        let t0 = Transaction{
            id: "0".to_string(),
            datetime: *DATE_TIME0,
            origin_wallet: "Bank".to_string(),
            origin_asset: "USD".to_string(),
            origin_quantity: 7000.0,
            destination_wallet: "Coinbase".to_string(),
            destination_asset: "BTC".to_string(),
            destination_quantity: 10.0,
            usd_value: 7000.0,
            usd_fee: None
        };
        let t1 = Transaction{
            id: "1".to_string(),
            datetime: *DATE_TIME1,
            origin_wallet: "External".to_string(),
            origin_asset: "BTC".to_string(),
            origin_quantity: 1.0,
            destination_wallet: "Coinbase".to_string(),
            destination_asset: "ETH".to_string(),
            destination_quantity: 30.0,
            usd_value: 3000.0,
            usd_fee: None
        };
        let t2 = Transaction{
            id: "2".to_string(),
            datetime: *DATE_TIME2,
            origin_wallet: "Conbase".to_string(),
            origin_asset: "ETH".to_string(),
            origin_quantity: 30.0,
            destination_wallet: "Coinbase".to_string(),
            destination_asset: "ADA".to_string(),
            destination_quantity: 10000.0,
            usd_value: 6000.0,
            usd_fee: None
        };

        let transactions = vec![t0,t1,t2];
        let tax_events = calculate_capital_gains(transactions, TAX_ACCOUNTING_METHOD_LIFO,  0);
        assert_eq!(tax_events.get(0).unwrap().gain, 2300.0);
        assert_eq!(tax_events.get(1).unwrap().gain, 3000.0);
    }
}
