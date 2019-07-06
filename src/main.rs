#[macro_use]
extern crate serde_derive;
extern crate clap;

#[macro_use]
extern crate maplit;

mod account;
mod tax;

use clap::{App, Arg, ArgMatches};
use tax::*;

fn read_arguments<'a>() -> ArgMatches<'a> {
    App::new("Pine Tree Tax")
        .version("0.01")
        .arg(
            Arg::with_name("transactions")
                .help("The input transaction history csv file to use.")
                .required(true)
                .default_value("./transactions.csv")
                .index(1),
        )
        .get_matches()
}

fn main() {
    let cli_args = read_arguments();
    let file_path = cli_args
        .value_of("transactions")
        .expect("No input filename given.");

    calculate_capital_gains(file_path);
}
