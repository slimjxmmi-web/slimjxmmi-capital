# Strategy kit

SlimJxmmi Capital carries no strategy: no signal, no sizing, no plan. The app reads NQ, builds 5-minute bars, and runs
one strategy file per firm, each written for its firm (Settings, Firms and strategy; `FIRMS.md` says what differs at
Topstep, Vest and FTMO). A file answers every question the app leaves open:

- when it fires (`push`),
- which hours it trades (`hours`),
- what each account does with a signal (`decide`: the order, or "not this one", or "this account is done").

A strategy file is a Rust library built to WebAssembly (a `.wasm` file). You do not have to know Rust: `PROMPT.md` is
what to give Claude so it writes the file from your rules.

- `sdk/`: what a strategy is written against. A `Bar` in, `Leg`s out; an `Account` in, a `Plan` out. No dependencies.
- `template/`: a strategy to start from. It fires nothing and places nothing until you write your rule in `push` and
  your plan in `decide` (examples sit there in comments).
- `PROMPT.md`: the message to start a conversation with Claude from.
- `FIRMS.md`: what each firm does with an order, and the numbers its accounts hand your file. Read it for the firm the
  file is for.

## Build one

Install Rust once (https://rustup.rs), then in this folder:

```
rustup target add wasm32-unknown-unknown
cargo test -p template
cargo build --release --target wasm32-unknown-unknown -p template
```

The file is `target/wasm32-unknown-unknown/release/template.wasm`. Upload it in the app while flat (Settings, Firms and
strategy); the engine restarts with it and the dashboard shows LOADED IN THE ENGINE.

## What a strategy can and cannot do

- It is fed every closed 5-minute NQ bar, in order: a few days of history first, then live. It answers each bar with
  the setups in place and which one, if any, fired. The signal's side and entry go to the desk.
- It decides what every account does. For each account a signal could go to, the desk asks `decide` with what it
  knows of the account (eval or funded, balance, floor, goal, fees, its trades so far and how the first went, its
  winning days, whether it traded this session) and does exactly what comes back: an order (NQ or MNQ, contracts,
  target and stop in points), wait, no room, or done with the account for good. The label it gives is what the board
  calls where the account stands, and the lowest priority goes first when several accounts could take a signal. The
  app adds nothing of its own; it only refuses an order that is not one (no contracts, no target, no stop) or that is
  more than the firm lets an account hold. A strategy with no `decide` trades nothing, and the dashboard says so.
- It decides its hours. Before any account is looked at, the desk asks `hours` whether a signal at that time is one
  it trades. The market's own hours and the firm's (nothing opened from 4:05 pm New York time until the reopen) are
  the desk's; every other hour is the strategy's.
- Its times are on your clock. The timezone is picked in the app (Settings); a bar's `minute` and `weekday`, and the
  ones `hours` is asked about, are on that clock, and so is every time the app shows. The platform and the code use
  the same one: write your hours as you read them on the app.
- The firm's rules stay the desk's: the floor, the target, the winning days and the payout terms.
- It runs sealed. No files, no network, no keys, no clock but the bar's own close time. A bar that runs too long is
  stopped, and a strategy that fails is dropped until a new file is uploaded; prices keep streaming either way.
- Only Rust is accepted: the app refuses a file that is not WebAssembly built by the Rust compiler, that imports
  anything, or that lacks the functions the engine calls. Rust source is not a strategy file; build it first.
- A file built with an older kit is refused, with a message that says so: build it again against this `sdk/`.

## Check it before it trades

- `cargo test`: write tests next to your rule (the template has two to copy from). Feed it bars, check what fires;
  hand `decide` an account, check the order.
- Against history: the engine that comes inside the app (`engine.exe`, in the app's install folder under `bin`) prints
  every signal a file fires on a Databento 1-second CSV (`ts_event, open, high, low, close`):

  ```
  engine --replay nq1s.csv target/wasm32-unknown-unknown/release/template.wasm
  ```

  Each line is the bar's close time (UTC), the side, the timeframe and the entry.
- In the app: a new install starts paused, and stays paused until you say so. Upload the file, watch what the
  dashboard says each account would do, and take the pause off when it reads right.

A strategy that has not been tested on history and on an eval account has no business on a funded one.
