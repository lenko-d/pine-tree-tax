![Pine Tree Tax](/images/pine_tree_tax_big.png)

Pine Tree Tax is a cryptocurrency capital gains calculator implemented in [Rust](https://en.wikipedia.org/wiki/Rust_(programming_language)).

Pine Tree Tax uses the double-entry bookkeeping method of accounting. Every transaction is recorded as both a debit and a credit in the general ledger. 
The totals of each should always balance. If there is a value difference between debits and credits then this indicates a recording error.

Advantages of double-entry accounting over single-entry methods:

* Helps in guaranteeing accurate financial records by revealing data entry errors.
* Provides a complete record of financial transactions.


# Cryptocurrencies according to the IRS
The IRS addressed the taxation of cryptocurrency transactions in [Notice 2014-21](https://www.irs.gov/pub/irs-drop/n-14-21.pdf). 
According to that notice cryptocurrencies are treated as property for tax purposes and taxpayers must recognize gain or loss on the exchange of 
cryptocurrency for cash or for other property. Gain or loss is also recognized every time a cryptocurrency is sold or used to purchase goods or services. 

# Types of transactions:
## Settled for cash
A taxpayer who sells a cryptocurrency position for cash must report a capital gain on Form 8949. A position held for one year or less is considered 
a short-term capital gain taxed at ordinary tax rates. A position held for more than one year is considered a long-term capital gain.

As with stock trades, capital losses offset capital gains in full, and a net capital loss is limited to $3,000 ($1,500 for married taxpayers filing 
separately) against other types of income on an individual tax return. An excess capital loss is carried forward to the subsequent tax year.

Under IRS rules, the default for stock transactions is the first-in, first-out (FIFO) method of accounting.

## Exchanged for other cryptocurrencies.
Crypto-to-crypto trades generate tax liability.

## Cryptocurrency mining.
When a taxpayer mines a cryptocurrency, the fair market value of the coins mined should be included in gross income. The amount of the income equals 
the market price of the coins on the day they were awarded on the blockchain.

## Payment for goods and services.
Paying for coffee with cryptocurrencies is a taxable event.

## Chain forks and airdrops.
Have to report taxable income using market value on the day of the split or airdrop.

## Donating cryptocurrency.
Cryptocurrency can be donated directly to a charity. The tax deduction will be equal to the fair market value of the donated coins, 
and the donor will not pay tax on the gain. 

---


# Pine Tree Tax


PTT provides the FIFO (first in, first out) and LIFO (last in, first out) methods of accounting. It considers every transaction 
between two different cryptocurrencies as a taxable event. It tracks the cost basis from the price of the original purchase and transfers that 
cost basis from the original token to the new token.


Capital gains or losses events are triggered when a cryptocurrency is sold for USD or when a quantity is transferred to an account not owned by me.


PTT is a command line application that reads the cryptocurrency transactions from an input .csv file. Example transaction file:

```
| id | datetime                 | origin_wallet | origin_asset | origin_quantity | destination_wallet | destination_asset | destination_quantity | usd_value | usd_fee |
|----+--------------------------+---------------+--------------+-----------------+--------------------+-------------------+----------------------+-----------+---------+
|  1 | 2016-05-10T13:01:00.000Z | External      | BTC          |      0.18312594 | Coinbase           | BTC               |           0.18312594 |     83.23 |         |
|  2 | 2016-05-12T20:59:00.000Z | Coinbase      | BTC          |      0.01000000 | External           | BTC               |           0.01000000 |      4.56 |         |
|  3 | 2016-05-12T23:12:00.000Z | Coinbase      | BTC          |      0.08788000 | External           | BTC               |           0.08788000 |     40.00 |         |
|  4 | 2016-06-02T08:09:00.000Z | Bank          | USD          |   2475.25000000 | Coinbase           | BTC               |           4.61456003 |   2475.25 |   24.75 |
|  5 | 2016-06-05T18:57:00.000Z | External      | BTC          |      0.04112000 | Coinbase           | BTC               |           0.04112000 |     23.72 |         |
|  6 | 2016-06-05T19:50:00.000Z | Coinbase      | BTC          |      0.02000000 | External           | BTC               |           0.02000000 |     11.51 |         |
|  7 | 2016-06-07T15:37:00.000Z | External      | BTC          |      0.07062000 | Coinbase           | BTC               |           0.07062000 |     40.77 |         |
|  8 | 2016-06-14T12:23:36.000Z | Gdax          | BTC          |      1.49551345 | Gdax               | ETH               |          55.22575516 |   1033.61 |    3.10 |

```

The value in the field "usd_value" represents the market value of the transaction at the time the exchange took place.

---


The application generates 2 output files in .csv format (long and short term capital gains). Example output file:

|           quantity | asset | buy_date             | sell_date            | cost_basis | proceeds |     gain |
|--------------------|-------|----------------------|----------------------|------------|----------|----------|
|         0.00084522 | BTC   | 2016-06-24T13:10:00Z | 2017-07-31T16:10:00Z |       0.56 |     2.42 |     1.86 |
|         0.39549275 | BTC   | 2016-06-24T13:10:00Z | 2017-10-13T10:57:24Z |     259.83 |  2216.62 |  1956.79 |
| 3.5656486000000003 | BTC   | 2016-06-24T13:29:33Z | 2017-10-13T10:57:24Z |    2388.88 | 19984.39 | 17595.51 |
|               0.05 | BTC   | 2016-06-24T13:29:33Z | 2017-12-04T00:40:00Z |       33.5 |    566.8 |    533.3 |
|                  5 | BTC   | 2016-06-24T13:29:33Z | 2017-12-10T15:02:00Z |    3349.85 |    77000 | 73650.15 |
|         0.16960901 | BTC   | 2016-06-24T13:29:33Z | 2017-12-18T16:55:00Z |     113.63 |  3218.38 |  3104.75 |


# Running
## Process a transaction file and generate long and short term capital gains reports in .csv format:
```
cargo run -- <INPUT_FILE_NAME>
```
By default, PTT uses LIFO accounting. To specify FIFO use the -a parameter:
```
run -- transactions.csv -a FIFO
```

## Convert from Kraken transaction format to Pine Tree Tax format:
```
cargo run --   trades.csv -c kraken 
```

## Convert from Bittrex transaction format to Pine Tree Tax format:
```
cargo run --  BittrexOrderHistory_2017.csv -c bittrex
```



---

# Contributing

1. Fork it
2. Download your fork to your PC (`git clone https://github.com/your_username/pine-tree-tax && cd pine-tree-tax`)
3. Create your feature branch (`git checkout -b my-new-feature`)
4. Make changes and add them (`git add .`)
5. Commit your changes (`git commit -m 'Add some feature'`)
6. Push to the branch (`git push origin my-new-feature`)
7. Create new pull request

---



