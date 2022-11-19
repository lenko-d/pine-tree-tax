extern crate chrono;
use self::chrono::prelude::*;

pub const TAX_ACCOUNTING_METHOD_FIFO: &str = "FIFO";
pub const TAX_ACCOUNTING_METHOD_LIFO: &str = "LIFO";

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
    name: String,
    balance: f64,
    deposits: Vec<Deposit>,
}

impl Account {
    pub fn new(name: String, balance: f64) -> Account {
        let mut deposits = vec![];

        if balance > 0.0 {
            let some_date_time_in_the_past = NaiveDateTime::from_timestamp(1_000_000_000, 0);
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
        let mut withdrawnQuantities = vec![];

        let filterByDateAndRemainingQuantity= |x :&&mut Deposit| x.datetime < datetime && x.remaining_quantity > 0.0;

        let calculateWithdrawals = |x: &mut Deposit, quantity: &mut f64, balance: &mut f64, withdrawn: &mut Vec<Deposit>| {
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

        let it = self.deposits.iter_mut().filter(filterByDateAndRemainingQuantity);
        if tax_accounting_method == TAX_ACCOUNTING_METHOD_LIFO {
            for x in it.rev() {
                calculateWithdrawals(x, &mut quantity, &mut self.balance, &mut withdrawnQuantities);
            }
        } else if tax_accounting_method == TAX_ACCOUNTING_METHOD_FIFO {
            for x in it {
                calculateWithdrawals(x, &mut quantity, &mut self.balance, &mut withdrawnQuantities);
            }
        } else {
            panic!("Unsupported tax_accounting_method:{}", tax_accounting_method);
        }

        //dbg!(&withdrawnQuantities);
        withdrawnQuantities
    }
}
