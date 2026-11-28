# One strategy file per firm

Every firm runs a strategy file of its own, written for that firm. Upload them in Settings, Firms and strategy:
Topstep's in its row, Vest's and FTMO's on their cards. A firm with no file trades nothing, and no firm ever trades on
another firm's file.

Every file is written the same way: this kit, the same three functions (`push`, `hours`, `decide`). What changes from
firm to firm is two things:
- what the firm does with the order you return
- the numbers `decide` is handed for that firm's accounts

Write each file with its firm's notes below in front of you. With Claude, give it this file together with `PROMPT.md`
and say which firm the file is for.

## The same at every firm

- `push` gets the closed 5-minute bars of the NQ future (CME), from the price feed picked in Settings. Every firm's
  file sees the same bars, Vest's included.
- `decide` answers in NQ or MNQ contracts, with a target and a stop in points from the fill. For each signal it is
  asked about each account of that firm, one at a time.
- `hours` is yours. The market's hours are the app's, and so is the close: nothing is opened from 4:05 pm New York time
  until the reopen, and whatever is open then is closed.
- Each firm's books are its own. `trades`, `first`, `traded_today` and `session_trades` count that firm's accounts
  only.

## Topstep

- **Where orders go:** the account's NQ or MNQ contract on TopstepX.
- **floor:** Topstep's trailing end-of-day drawdown.
- **goal:** Topstep's target, raised by its consistency rule.
- **start:** the account's size.
- **Winning days:** days of $150 or more. `days_needed` is your file's payout count.
- **Contract limit:** 5 minis (50 micros) per 50K. Larger orders are refused.
- **Fees:** Topstep's round trips ($3.78 an NQ, $1.22 an MNQ).

## Vest Markets

**Where orders go.** NQ-PERP, Vest's perpetual, sized in units:
- 1 MNQ = 2 units, 1 NQ = 20 units (a unit is $1 a point).
- Size in whole contracts. The smallest order is 1 MNQ: 2 units, $2 a point.

**Prices.** Fills happen at Vest's NQ-PERP price. It's close to the NQ future's price but not the same:
- The signal comes from the future's bars. The target and stop are measured from Vest's own price as the order goes in.
- The spread can be wide at night (4¾ points was seen), so keep the stop well wider than that.

**The numbers `decide` gets:**
- **balance:** the account value.
- **floor:** Vest's own: the account value less its Max. Drawdown Left.
- **start:** the account value when the app first read the account (Vest's page does not show the account's size).
- **goal:** a placeholder (1,000,000,000). Vest judges its evaluations itself, so the app never calls one passed. If
  your plan depends on the profit target, write your Vest plan's target into the file.
- **Winning days:** days of $150 or more (the app's default). Vest's payout terms are not built into the app.

**What the app refuses.** An order whose stop is past Vest's Fail Price, or whose loss at the stop (plus slippage)
would break today's daily loss limit. Size so the stop fits inside both. Beyond that, the most an account may hold is 5
minis per $50,000 of start, and never under 5.

**Fees.** Vest's taker fee is 0.0025% a side: about $26 an NQ and $2.60 an MNQ round trip near 26,000 (`fee_nq`,
`fee_mnq`).

**Copies.** Every Vest account added takes each signal, one after another. Vest shows one account at a time, so copies
fill a few seconds apart, each at its own price.

## FTMO

**Where orders go.** The front NQ or MNQ contract in NinjaTrader. It rolls 8 days before expiry.

**Plans.** Each account follows the FTMO plan picked when it was added: Growth or Pro, 50K, 100K or 150K. `decide`
gets FTMO's numbers for it:

| | Growth | Pro |
|---|---|---|
| floor: max loss, trailing the end-of-day balance, locked at the start | $2,000 / $3,000 / $4,500 | $3,000 / $4,500 / $6,000 |
| goal: target | $3,000 / $6,000 / $9,000 | $3,000 / $6,000 / $9,000 |
| goal raised so no day holds more than | 40% of the profit | 50% of the profit |
| winning day: one of at least | $150 / $300 / $450 | $200 / $300 / $500 |
| `days_needed` | 4 | 5 |
| daily loss limit | funded only, $1,000 / $2,000 / $3,000 | eval and funded, $1,000 / $1,500 / $2,000 |

`start` is the plan's size.

**Contract limits.** The app enforces them: a larger order is not placed (the board says "no sizing"). Ten micros
count as one mini.
- Evaluation: 5, 10 or 15 minis.
- Funded: grows with profit. For a 50K: 2 minis, 3 from +$1,000, 5 from +$2,000.

**Daily loss limit.** At the limit, the account stops for the day. Size so a stop never reaches it.

**No hedging.** FTMO forbids it, so while FTMO has accounts, no firm takes a side opposite a trade open at another.
With a different file per firm this can happen: Topstep's file is long while FTMO's fires short, and FTMO's signal is
skipped.

**Fees.** The app's round trips ($3.78 an NQ, $1.22 an MNQ). FTMO's own commissions may differ a little.

## Tradeify

**Where orders go.** The front NQ or MNQ contract in NinjaTrader, through the same add-on as FTMO. NinjaTrader is signed
in with the Tradeify login, or stays signed in to FTMO with the Tradeify Rithmic login added as a connection (Growth
and Select only: Miami is Tradovate only). It rolls 8 days before expiry.

**Plans.** Each account follows the Tradeify plan picked when it was added. `decide` gets Tradeify's numbers for it
(tradeify.co's pricing data, 25 Nov 2026):

| | Growth 25K / 50K / 100K / 150K | Select eval 25K / 50K / 100K / 150K / 300K | Miami 10K / 25K / 50K |
|---|---|---|---|
| floor | max loss trailing the end-of-day balance: $1,000 / $2,000 / $3,500 / $5,000 | $1,000 / $2,000 / $3,000 / $4,500 / $8,000 | static: $500 / $1,000 / $1,500 below the start |
| goal | $1,500 / $3,000 / $6,000 / $9,000 | $1,500 / $3,000 / $6,000 / $9,000 / $14,000 | $1,000 / $2,000 / $3,500 |
| consistency | none (35% once funded) | 40% on the eval | none |
| daily loss limit | $600 / $1,250 / $2,500 / $3,750 | none (300K: $4,000) | $300 / $600 / $900, or none |
| contracts | 1 / 4 / 8 / 12 minis | 1 / 4 / 8 / 12 / 16 minis | 10 / 20 / 30 micros |

A Select account that passes is added again on its funded path: **Daily** (daily payouts, a daily loss limit) or
**Flex** (every 5 days, none), each with its own drawdown and payout cap.

**Daily loss limit.** At the limit, the account stops for the day. Size so a stop never reaches it.

**No hedging.** While Tradeify has accounts, no firm takes a side opposite a trade open at another.

## Lucid Trading

**Where orders go.** The front NQ or MNQ contract in NinjaTrader, through the same add-on as FTMO. NinjaTrader is signed
in with the Lucid login, or stays signed in to FTMO with the Lucid Rithmic login added as a connection.

**Plans.** Flex and Pro, 25K / 50K / 100K / 150K, each with or without the daily loss limit (Flex also at 40% or 50%
consistency), as bought (support.lucidtrading.com, 27 Nov 2026):

| | Flex | Pro |
|---|---|---|
| floor | max loss trailing the end-of-day balance, $1,000 / $2,000 / $3,000 / $4,500, locked at the start + $100 | the same |
| goal | $1,250 / $3,000 / $6,000 / $9,000 | the same |
| consistency | 40% (or 50%) on the eval, none funded | none on the eval, 40% funded |
| daily loss limit, when bought | none on a 25K, $1,200 / $1,800 / $2,700 (soft: the day stops) | the same |
| contracts | 2 / 4 / 6 / 10 minis; funded, grows with profit | 2 / 4 / 6 / 10 minis |
| payout | 5 days of $100 / $150 / $200 / $250+, half the profit up to $1,000 / $2,000 / $2,500 / $3,000 | over a buffer of the max loss + $100, up to $1,000 / $2,000 / $2,500 / $3,000 |

Lucid allows automated trading and copiers; it forbids hedging (across firms too), high-frequency trading and
microscalping (half the profit from trades held 5 seconds or less). Commissions: $1.75 a side per NQ, $0.50 per MNQ.
