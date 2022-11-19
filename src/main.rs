extern crate clap;

#[macro_use]
extern crate maplit;

#[macro_use]
extern crate lazy_static;

extern crate serde;

mod account;
mod conversions;
mod tax;

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

        calculate_capital_gains(tax_accounting_method, input_file, output_file, output_positions);
    }
}
