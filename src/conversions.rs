extern crate chrono;
extern crate csv;

use serde::{Deserialize, Serialize};

use std::fs::File;

use self::chrono::prelude::*;
use std::error::Error;

use tax::Transaction;

use std::collections::HashMap;

const WALLET_KRAKEN: &str = "Kraken";
const WALLET_BITTREX: &str = "Bittrex";

lazy_static! {
    static ref KRAKEN_PAIRS: HashMap<String, (String, String)> = {
        let mut map = HashMap::new();
        map.insert("BCHXBT".to_string(), ("BCH".to_string(), "XBT".to_string()));
        map.insert(
            "XXMRZUSD".to_string(),
            ("XMR".to_string(), "USD".to_string()),
        );
        map.insert(
            "XZECZUSD".to_string(),
            ("ZEC".to_string(), "USD".to_string()),
        );
        map.insert(
            "XXBTZUSD".to_string(),
            ("XBT".to_string(), "USD".to_string()),
        );
        map.insert(
            "XXBTZUSD".to_string(),
            ("XBT".to_string(), "USD".to_string()),
        );
        map
    };
}

fn kraken_dest_asset<'a>(pair: &'a str, buy_or_sell: &str) -> &'a str {
    if buy_or_sell == "buy" {
        &KRAKEN_PAIRS.get(pair).unwrap().0
    } else {
        &KRAKEN_PAIRS.get(pair).unwrap().1
    }
}

fn kraken_orig_asset<'a>(pair: &'a str, buy_or_sell: &str) -> &'a str {
    if buy_or_sell == "sell" {
        &KRAKEN_PAIRS.get(pair).unwrap().0
    } else {
        &KRAKEN_PAIRS.get(pair).unwrap().1
    }
}

#[derive(Debug, Deserialize)]
struct KrakenTransaction {
    txid: String,
    ordertxid: String,
    pair: String,
    #[serde(with = "kraken_date_format")]
    time: DateTime<Utc>,
    #[serde(rename = "type")]
    type_: String,
    ordertype: String,
    price: f64,
    cost: f64,
    fee: f64,
    vol: f64,
    margin: f64,
    misc: String,
    ledgers: String,
}

mod kraken_date_format {
    use super::chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S%.f";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

fn read_kraken_transactions(file_path: &str) -> Result<Vec<KrakenTransaction>, Box<Error>> {
    let file = File::open(file_path)?;

    let mut reader = csv::Reader::from_reader(file);

    let mut transactions = Vec::new();
    for transaction in reader.deserialize() {
        transactions.push(transaction?);
    }

    Ok(transactions)
}

pub fn process_kraken_transactions(file_path: &str, out_file: &str) {
    let mut transactions = read_kraken_transactions(file_path).expect("Can't read transactions");
    transactions.sort_by(|t1, t2| t1.time.cmp(&t2.time));

    let mut ptt_transactons = Vec::new();

    for kraken_transaction in transactions.iter() {
        let orig_asset = kraken_orig_asset(&kraken_transaction.pair, &kraken_transaction.type_);

        ptt_transactons.push(Transaction {
            id: kraken_transaction.txid.to_owned(),
            datetime: kraken_transaction.time,
            origin_wallet: WALLET_KRAKEN.to_string(),
            origin_asset: orig_asset.to_string(),
            origin_quantity: kraken_transaction.cost,
            destination_wallet: WALLET_KRAKEN.to_string(),
            destination_asset: kraken_dest_asset(
                &kraken_transaction.pair,
                &kraken_transaction.type_,
            )
            .to_owned(),
            destination_quantity: kraken_transaction.vol,
            usd_value: (kraken_transaction.vol * kraken_transaction.price) + kraken_transaction.fee,
            usd_fee: Some(kraken_transaction.fee),
        });
    }

    let file = File::create(out_file)
        .ok()
        .expect("Unable to create output file.");

    let mut writer = csv::Writer::from_writer(file);

    for ptt_transaction in ptt_transactons.iter() {
        writer
            .serialize(ptt_transaction)
            .ok()
            .expect("Unable to write to output file.");
    }
}

lazy_static! {
    static ref BITTREX_PAIRS: HashMap<String, (String, String)> = {
        let mut map = HashMap::new();

        map.insert(
            "BTC-PIVX".to_string(),
            ("BTC".to_string(), "PIVX".to_string()),
        );
        map.insert(
            "BTC-BITB".to_string(),
            ("BTC".to_string(), "BITB".to_string()),
        );
        map.insert(
            "BTC-DASH".to_string(),
            ("BTC".to_string(), "DASH".to_string()),
        );
        map.insert(
            "BTC-XZC".to_string(),
            ("BTC".to_string(), "XZC".to_string()),
        );
        map.insert(
            "BTC-ADA".to_string(),
            ("BTC".to_string(), "ADA".to_string()),
        );
        map
    };
}

fn bittrex_dest_asset<'a>(pair: &'a str, buy_or_sell: &str) -> &'a str {
    if buy_or_sell == "LIMIT_BUY" {
        &BITTREX_PAIRS.get(pair).unwrap().0
    } else {
        &BITTREX_PAIRS.get(pair).unwrap().1
    }
}

fn bittrex_orig_asset<'a>(pair: &'a str, buy_or_sell: &str) -> &'a str {
    if buy_or_sell == "LIMIT_SELL" {
        &BITTREX_PAIRS.get(pair).unwrap().0
    } else {
        &BITTREX_PAIRS.get(pair).unwrap().1
    }
}

#[derive(Debug, Deserialize)]
struct BittrexTransaction {
    Uuid: String,
    Exchange: String,
    #[serde(with = "bittrex_date_format")]
    TimeStamp: DateTime<Utc>,
    OrderType: String,
    Limit: f64,
    Quantity: f64,
    QuantityRemaining: f64,
    Commission: f64,
    Price: f64,
    PricePerUnit: f64,
    IsConditional: String,
    Condition: String,
    ConditionTarget: String,
    ImmediateOrCancel: String,
    #[serde(with = "bittrex_date_format")]
    Closed: DateTime<Utc>,
}

fn read_bittrex_transactions(file_path: &str) -> Result<Vec<BittrexTransaction>, Box<Error>> {
    let file = File::open(file_path)?;

    let mut reader = csv::Reader::from_reader(file);

    let mut transactions = Vec::new();
    for transaction in reader.deserialize() {
        transactions.push(transaction?);
    }

    Ok(transactions)
}

pub fn process_bittrex_transactions(file_path: &str, out_file: &str) {
    let mut transactions = read_bittrex_transactions(file_path).expect("Can't read transactions");
    transactions.sort_by(|t1, t2| t1.TimeStamp.cmp(&t2.TimeStamp));

    let mut ptt_transactons = Vec::new();

    for bittrex_transaction in transactions.iter() {
        dbg!(bittrex_transaction);

        let orig_asset = bittrex_orig_asset(
            &bittrex_transaction.Exchange,
            &bittrex_transaction.OrderType,
        );
        let parse_from_str = NaiveDateTime::parse_from_str;

        ptt_transactons.push(Transaction {
            id: bittrex_transaction.Uuid.to_owned(),
            datetime: bittrex_transaction.TimeStamp,
            origin_wallet: WALLET_BITTREX.to_string(),
            origin_asset: orig_asset.to_string(),
            origin_quantity: bittrex_transaction.Price,
            destination_wallet: WALLET_BITTREX.to_string(),
            destination_asset: bittrex_dest_asset(
                &bittrex_transaction.Exchange,
                &bittrex_transaction.OrderType,
            )
            .to_owned(),
            destination_quantity: bittrex_transaction.Price,
            usd_value: (bittrex_transaction.Quantity * bittrex_transaction.PricePerUnit) // TODO get real USD value
                + bittrex_transaction.Commission,
            usd_fee: Some(bittrex_transaction.Commission),
        });
    }

    let file = File::create(out_file)
        .ok()
        .expect("Unable to create output file.");

    let mut writer = csv::Writer::from_writer(file);

    for ptt_transaction in ptt_transactons.iter() {
        writer
            .serialize(ptt_transaction)
            .ok()
            .expect("Unable to write to output file.");
    }
}

mod bittrex_date_format {
    use super::chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%m/%d/%Y %_I:%M:%S %p";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}
