extern crate chrono;
use self::chrono::prelude::*;

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
            datetime: datetime,
            quantity: quantity,
            usd_value: usd_value,
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
        let mut deposits = Vec::new();

        if balance > 0.0 {
            let some_date_time_in_the_past = NaiveDateTime::from_timestamp(1_000_000_000, 0);
            let existing_account_datetime =
                DateTime::<Utc>::from_utc(some_date_time_in_the_past, Utc);

            assert!(name == "USD");
            deposits.push(Deposit::new(existing_account_datetime, balance, balance));
        }

        Account {
            name,
            balance,
            deposits: deposits,
        }
    }

    pub fn deposit(&mut self, datetime: DateTime<Utc>, quantity: f64, usd_value: f64) {
        self.deposits
            .push(Deposit::new(datetime, quantity, usd_value));

        self.balance += quantity;
    }

    pub fn withdraw(&mut self, datetime: DateTime<Utc>, mut quantity: f64) -> Vec<Deposit> {
        let mut deposits: Vec<Deposit> = Vec::new();

        for candidate_to_withdraw_index in 0..self.deposits.len() {
            let mut candidate = self.deposits.get_mut(candidate_to_withdraw_index).unwrap();

            if candidate.datetime > datetime {
                panic!("Candidate is in the future!")
            };

            if candidate.remaining_quantity <= 0.0 {
                continue;
            };

            let sold_quantity = candidate.remaining_quantity.min(quantity);
            let deposit = Deposit::new(
                candidate.datetime,
                sold_quantity,
                candidate.usd_value * (sold_quantity / candidate.quantity),
            );
            deposits.push(deposit);
            candidate.claim(sold_quantity);
            quantity -= sold_quantity;
            self.balance -= sold_quantity;

            if quantity <= 0.0 {
                break;
            };
        }

        //dbg!(&deposits);
        return deposits;
    }
}
