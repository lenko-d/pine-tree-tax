extern crate chrono;
use self::chrono::prelude::*;

pub const TAX_ACCOUNTING_METHOD_FIFO: &str = "FIFO";
pub const TAX_ACCOUNTING_METHOD_LIFO: &str = "LIFO";
pub const TAX_ACCOUNTING_METHOD_HIFO: &str = "HIFO";

#[derive(Debug)]
pub struct Deposit {
    pub datetime: DateTime<Utc>,
    pub quantity: f64,
    pub usd_value: f64,
    pub remaining_quantity: f64,
}

impl Deposit {
    pub fn new(datetime: DateTime<Utc>, quantity: f64, usd_value: f64) -> Deposit {
        Deposit {
            datetime,
            quantity,
            usd_value,
            remaining_quantity: quantity,
        }
    }

    fn claim(&mut self, quantity: f64) {
        if quantity > self.remaining_quantity {
            panic!("Not enough quantity remaining");
        } else {
            self.remaining_quantity -= quantity
        }
    }
}

#[derive(Debug)]
pub struct Account {
    pub name: String,
    pub balance: f64,
    pub deposits: Vec<Deposit>,
}

impl Account {
    pub fn new(name: String, balance: f64) -> Account {
        let mut deposits = vec![];

        if balance > 0.0 {
            let some_date_time_in_the_past = NaiveDateTime::from_timestamp(1_000_000, 0);
            let existing_account_datetime =
                DateTime::<Utc>::from_utc(some_date_time_in_the_past, Utc);

            assert_eq!(name, "USD");
            deposits.push(Deposit::new(existing_account_datetime, balance, balance));
        }

        Account {
            name,
            balance,
            deposits,
        }
    }

    pub fn deposit(&mut self, datetime: DateTime<Utc>, quantity: f64, usd_value: f64) {
        self.deposits
            .push(Deposit::new(datetime, quantity, usd_value));

        self.balance += quantity;
    }

    pub fn withdraw(&mut self, datetime: DateTime<Utc>, mut quantity: f64, tax_accounting_method: &str) -> Vec<Deposit> {
        let mut withdrawn_quantities = vec![];

        let filter_by_date_and_remaining_quantity = |x :&&mut Deposit| x.datetime < datetime && x.remaining_quantity > 0.0;

        let calculate_withdrawals = |x: &mut Deposit, quantity: &mut f64, balance: &mut f64, withdrawn: &mut Vec<Deposit>| {
            if *quantity <= 0.0 {
                return ;
            };

            let sold_quantity = x.remaining_quantity.min(*quantity);
            let deposit = Deposit::new(
                x.datetime,
                sold_quantity,
                x.usd_value * (sold_quantity / x.quantity),
            );
            withdrawn.push(deposit);
            x.claim(sold_quantity);
            *quantity -= sold_quantity;
            *balance -= sold_quantity;
        };

        let it = self.deposits.iter_mut().filter(filter_by_date_and_remaining_quantity);
        if tax_accounting_method == TAX_ACCOUNTING_METHOD_LIFO {
            for d in it.rev() {
                calculate_withdrawals(d, &mut quantity, &mut self.balance, &mut withdrawn_quantities);
            }
        } else if tax_accounting_method == TAX_ACCOUNTING_METHOD_FIFO {
            for d in it {
                calculate_withdrawals(d, &mut quantity, &mut self.balance, &mut withdrawn_quantities);
            }
        } else if tax_accounting_method == TAX_ACCOUNTING_METHOD_HIFO {
            let mut filtered_and_sorted_by_highest_cost_basis = it.collect::<Vec<&mut Deposit>>();
            filtered_and_sorted_by_highest_cost_basis.sort_by(|a,b| b.usd_value.partial_cmp(&a.usd_value).unwrap() );
            for d in filtered_and_sorted_by_highest_cost_basis.iter_mut() {
                calculate_withdrawals(*d, &mut quantity, &mut self.balance, &mut withdrawn_quantities);
            }
        } else {
            panic!("Unsupported tax_accounting_method:{}", tax_accounting_method);
        }

        withdrawn_quantities
    }
}
