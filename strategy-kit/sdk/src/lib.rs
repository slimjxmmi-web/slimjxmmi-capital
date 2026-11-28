//! What a strategy file is written against. A strategy is a Rust library built to WebAssembly, and it holds every
//! trading decision. The app has none of its own: it knows the firm's rules and how to place an order, nothing else.
//!
//! - **when to fire**: the engine feeds it every closed 5-minute NQ bar, in order, and posts what it fires (`push`);
//! - **which hours it trades**: before any signal is taken the desk asks whether this is a time it trades (`hours`);
//! - **what each account does**: for every account a signal could go to, the desk hands over what it knows of the
//!   account and gets back the order (contracts, target, stop), or "not this one", or "this account is done"
//!   (`decide`). Which account goes first when several could take it is the strategy's too (the plan's priority).
//!
//! A strategy that leaves `decide` as it comes fires signals and trades nothing.
//! It runs sealed: it can compute, and nothing else (no files, no network, no clock but the bar's own close time).
//! Times are in the timezone picked in the app (Settings): the platform and the code use the same one.
//!
//! ```text
//! use sjxm::{Account, Bar, Leg, Plan, Side, Strategy, What};
//! struct Mine { bars: Vec<Bar> }
//! impl Strategy for Mine { ... }
//! sjxm::export!(Mine);
//! ```
//!
//! export! writes the functions the engine calls (strategy ABI 3, engine/src/strategy.rs), so a strategy never
//! touches them. No dependencies: a strategy file stays a few kilobytes.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    /// When the bar closed, unix seconds.
    pub tc: i64,
    /// The same moment on the clock of the timezone picked in the app: minutes since midnight (9:30 am is 570) ...
    pub minute: u16,
    /// ... and the day of the week there: 0 Monday to 6 Sunday.
    pub weekday: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Side {
    Long,
    Short,
}

/// A setup in place on a bar. The one that fired is the signal; the others are only logged ("close not strong enough").
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Leg {
    pub side: Side,
    /// The timeframe the setup formed on, in minutes: 5 or 15.
    pub tf: u8,
    /// The price the signal is taken at: the close of the bar that fired.
    pub entry: f64,
    /// The far end of the setup (the low under a long, the high over a short).
    pub extreme: f64,
    pub atr: f64,
    /// How far the bar closed past what it had to break, and that less what it needed: above zero fires. A strategy
    /// with no such measure sets brk and margin to 1.0 on a leg that fires and returns no other legs.
    pub brk: f64,
    pub margin: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    /// An evaluation: it passes when its profit reaches `goal`.
    Eval,
    /// A funded account: it pays out after `days_needed` winning days.
    Funded,
}

/// A trade the account took before.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Past {
    pub won: bool,
    /// It was in micros (MNQ); false for minis (NQ).
    pub micro: bool,
}

/// How many facts the desk hands over about an account (the order of `Account::from_facts`).
pub const FACTS: usize = 17;

/// What the desk knows of the account it asks about. Money is in dollars. The desk states facts; what to make of
/// them (an opening trade, a small one, none at all) is the strategy's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Account {
    pub kind: Kind,
    pub balance: f64,
    /// The balance at which the firm closes the account.
    pub floor: f64,
    /// The profit an eval must show to pass, raised by the firm's consistency rule if a day ran big.
    pub goal: f64,
    /// The balance the account started at.
    pub start: f64,
    /// Round-trip fees for one contract, NQ and MNQ.
    pub fee_nq: f64,
    pub fee_mnq: f64,
    /// Trades it has taken on this desk.
    pub trades: u32,
    /// The first of them.
    pub first: Option<Past>,
    /// Winning days toward its next payout, as the firm counts them, and how many the firm asks for.
    pub winning_days: u32,
    pub days_needed: u32,
    /// Payouts it has given.
    pub payouts: u32,
    /// It has no history on this desk at all: no trade, no winning day carried in, no payout.
    pub untouched: bool,
    /// Trades it has taken in this session (a session starts at 6 pm New York time).
    pub traded_today: u32,
    /// Accounts on the desk that have traded in this session.
    pub session_trades: u32,
    /// Two switches from the settings file (STRATEGY_OPTION_1, STRATEGY_OPTION_2), for the strategy to give a meaning.
    pub option1: bool,
    pub option2: bool,
}

impl Account {
    /// The account from the numbers the desk sends, in their order.
    pub fn from_facts(f: &[f64; FACTS]) -> Account {
        let count = |v: f64| if v.is_finite() && v > 0.0 { v as u32 } else { 0 };
        Account {
            kind: if f[0] == 1.0 { Kind::Funded } else { Kind::Eval },
            balance: f[1],
            floor: f[2],
            goal: f[3],
            start: f[4],
            fee_nq: f[5],
            fee_mnq: f[6],
            trades: count(f[7]),
            first: (f[8] != 0.0).then_some(Past { won: f[8] > 0.0, micro: f[9] != 0.0 }),
            winning_days: count(f[10]),
            days_needed: count(f[11]),
            payouts: count(f[12]),
            untouched: f[13] != 0.0,
            traded_today: count(f[14]),
            session_trades: count(f[15]),
            option1: f[16] == 1.0 || f[16] == 3.0,
            option2: f[16] == 2.0 || f[16] == 3.0,
        }
    }
}

/// The order for one account: what to trade and the bracket around it, in points from the entry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Order {
    /// MNQ ($2 a point) when true, NQ ($20 a point) when false.
    pub micro: bool,
    pub qty: u32,
    pub target: f64,
    pub stop: f64,
    /// Dollars the trade should net after fees: what the desk shows.
    pub net: f64,
    /// The room to the floor this order was sized against.
    pub room: f64,
}

/// What an account does with a signal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum What {
    /// It takes the trade, with this order.
    Trade(Order),
    /// Not this signal (it traded already today, say). Nothing is said about it.
    Wait,
    /// It would, but it is too close to its floor: it sits out, and the desk says why.
    NoRoom { room: f64, needs: f64 },
    /// The strategy is finished with this account for good. The desk stops routing to it and tells the owner.
    Done,
    /// The strategy has no answer. What it says until its plan is written: nothing is placed.
    Unsized,
}

/// A strategy's answer about one account.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plan {
    /// What the board calls where this account stands ("day", "scalp", anything short). Up to 16 plain characters.
    pub label: &'static str,
    /// When several accounts could take a signal, the lowest goes first; equal ones go by the higher balance.
    pub priority: i32,
    pub what: What,
}

impl Plan {
    pub const UNSIZED: Plan = Plan { label: "", priority: 0, what: What::Unsized };
}

/// Dollars a point: NQ and its micro.
pub const NQ: f64 = 20.0;
pub const MNQ: f64 = 2.0;
pub const TICK: f64 = 0.25;

/// Dollars as points for a number of contracts, on the tick: rounded up for a target, down for a stop.
pub fn points(dollars: f64, qty: u32, point_value: f64, up: bool) -> f64 {
    let pts = dollars / (qty as f64 * point_value) / TICK;
    (if up { pts.ceil() } else { pts.floor() }) * TICK
}

pub trait Strategy {
    /// Bars needed before the first signal is possible. The engine loads this much history before it goes live.
    const HISTORY: usize;
    /// A fresh strategy with no bars. window_start: the hour set as WINDOW_START in the settings file (18 unless set).
    fn new(window_start: i32) -> Self;
    /// One closed 5-minute bar. Returns the legs in place on it and the index of the one that fired, if one did.
    fn push(&mut self, bar: Bar) -> (Vec<Leg>, Option<usize>);
    /// How many bars the strategy holds.
    fn bars(&self) -> usize;

    /// Whether this is a time it trades: asked once for every signal, whoever sent it, before any account is looked at.
    /// minute and weekday are the signal's, on the clock of the timezone picked in the app. Always, as it comes.
    fn hours(&self, _minute: u16, _weekday: u8) -> bool {
        true
    }
    /// What this account does with a signal. The app has no plan of its own: until this is written, nothing is placed.
    fn decide(_account: &Account) -> Plan {
        Plan::UNSIZED
    }
    /// The room above the floor a payout must leave on a funded account, so it can keep trading after it.
    fn keep_after_payout(_fee_nq: f64, _fee_mnq: f64) -> f64 {
        0.0
    }
}

/// Days since 1970-01-01 of a calendar date.
fn days(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let (era, yoe) = (y.div_euclid(400), y.rem_euclid(400));
    let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719_468
}

/// The calendar year a day (since 1970-01-01) falls in.
fn year_of(day: i64) -> i64 {
    let z = day + 719_468;
    let (era, doe) = (z.div_euclid(146_097), z.rem_euclid(146_097));
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    yoe + era * 400 + ((5 * doy + 2) / 153 >= 10) as i64
}

/// Unix seconds of the nth Sunday of a month, at an hour UTC.
fn sunday(year: i64, month: i64, nth: i64, utc_hour: i64) -> i64 {
    let first = days(year, month, 1);
    let weekday = (first + 4).rem_euclid(7); // 0 is Sunday: 1970-01-01 was a Thursday
    (first + (7 - weekday) % 7 + 7 * (nth - 1)) * 86_400 + utc_hour * 3600
}

/// Minutes since midnight in New York at this unix time: 9:30 am is 570, 7:00 pm is 1140. Daylight saving by the rule
/// in force since 2007 (second Sunday of March to first Sunday of November, changing at 2:00 am local).
pub fn new_york_minute(tc: i64) -> i64 {
    let year = year_of(tc.div_euclid(86_400));
    let daylight = tc >= sunday(year, 3, 2, 7) && tc < sunday(year, 11, 1, 6);
    (tc - if daylight { 4 } else { 5 } * 3600).rem_euclid(86_400) / 60
}

/// Makes a strategy the file's one strategy: writes the functions the engine calls. Once per file.
#[macro_export]
macro_rules! export {
    ($strategy:ty) => {
        #[cfg(target_arch = "wasm32")]
        mod __sjxm_exports {
            use super::*;

            struct __SjxmState {
                strategy: $strategy,
                legs: Vec<$crate::Leg>,
                fired: i32,
            }
            thread_local! {
                static __SJXM: std::cell::RefCell<Option<__SjxmState>> = const { std::cell::RefCell::new(None) };
                static __SJXM_FACTS: std::cell::RefCell<[f64; $crate::FACTS]> = const { std::cell::RefCell::new([0.0; $crate::FACTS]) };
                static __SJXM_PLAN: std::cell::Cell<$crate::Plan> = const { std::cell::Cell::new($crate::Plan::UNSIZED) };
            }
            fn __sjxm_order<T>(read: impl Fn(&$crate::Order) -> T, none: T) -> T {
                match __SJXM_PLAN.with(|p| p.get()).what {
                    $crate::What::Trade(order) => read(&order),
                    _ => none,
                }
            }
            fn __sjxm_new(window_start: i32) -> __SjxmState {
                __SjxmState { strategy: <$strategy as $crate::Strategy>::new(window_start), legs: Vec::new(), fired: -1 }
            }
            fn __sjxm_leg<T>(i: i32, read: impl Fn(&$crate::Leg) -> T, none: T) -> T {
                __SJXM.with(|s| s.borrow().as_ref().and_then(|st| st.legs.get(i as usize)).map(&read).unwrap_or(none))
            }

            #[no_mangle]
            pub extern "C" fn sjxm_abi() -> i32 {
                3
            }
            /// One fact about the account the desk is about to ask about (the order of Account::from_facts).
            #[no_mangle]
            pub extern "C" fn fact(i: i32, value: f64) {
                __SJXM_FACTS.with(|f| {
                    let mut facts = f.borrow_mut();
                    if let Some(slot) = usize::try_from(i).ok().and_then(|i| facts.get_mut(i)) {
                        *slot = value;
                    }
                });
            }
            /// What that account does with a signal: 0 a trade (read it with size_*), 1 wait, 2 no room, 3 done, 4 no answer.
            #[no_mangle]
            pub extern "C" fn decide() -> i32 {
                let account = __SJXM_FACTS.with(|f| $crate::Account::from_facts(&f.borrow()));
                let plan = <$strategy as $crate::Strategy>::decide(&account);
                __SJXM_PLAN.with(|p| p.set(plan));
                match plan.what {
                    $crate::What::Trade(_) => 0,
                    $crate::What::Wait => 1,
                    $crate::What::NoRoom { .. } => 2,
                    $crate::What::Done => 3,
                    $crate::What::Unsized => 4,
                }
            }
            #[no_mangle]
            pub extern "C" fn plan_priority() -> i32 {
                __SJXM_PLAN.with(|p| p.get().priority)
            }
            /// The label of the last answer: where its text sits in this module's memory, and how long it is.
            #[no_mangle]
            pub extern "C" fn label_ptr() -> i32 {
                __SJXM_PLAN.with(|p| p.get().label.as_ptr() as i32)
            }
            #[no_mangle]
            pub extern "C" fn label_len() -> i32 {
                __SJXM_PLAN.with(|p| p.get().label.len() as i32)
            }
            #[no_mangle]
            pub extern "C" fn size_micro() -> i32 {
                __sjxm_order(|o| o.micro as i32, 0)
            }
            #[no_mangle]
            pub extern "C" fn size_qty() -> i32 {
                __sjxm_order(|o| o.qty as i32, 0)
            }
            #[no_mangle]
            pub extern "C" fn size_target() -> f64 {
                __sjxm_order(|o| o.target, 0.0)
            }
            #[no_mangle]
            pub extern "C" fn size_stop() -> f64 {
                __sjxm_order(|o| o.stop, 0.0)
            }
            #[no_mangle]
            pub extern "C" fn size_net() -> f64 {
                __sjxm_order(|o| o.net, 0.0)
            }
            /// The room the last answer saw: the order's, or what a "no room" had.
            #[no_mangle]
            pub extern "C" fn size_room() -> f64 {
                match __SJXM_PLAN.with(|p| p.get()).what {
                    $crate::What::Trade(order) => order.room,
                    $crate::What::NoRoom { room, .. } => room,
                    _ => 0.0,
                }
            }
            /// What a "no room" needed.
            #[no_mangle]
            pub extern "C" fn size_needs() -> f64 {
                match __SJXM_PLAN.with(|p| p.get()).what {
                    $crate::What::NoRoom { needs, .. } => needs,
                    _ => 0.0,
                }
            }
            /// Whether a signal at this time of this weekday is one the strategy trades: 1 yes, 0 no.
            #[no_mangle]
            pub extern "C" fn hours(minute: i32, weekday: i32) -> i32 {
                __SJXM.with(|s| {
                    let mut state = s.borrow_mut();
                    let st = state.get_or_insert_with(|| __sjxm_new(18));
                    <$strategy as $crate::Strategy>::hours(&st.strategy, minute.clamp(0, 1439) as u16, weekday.clamp(0, 6) as u8) as i32
                })
            }
            #[no_mangle]
            pub extern "C" fn keep_after_payout(fee_nq: f64, fee_mnq: f64) -> f64 {
                <$strategy as $crate::Strategy>::keep_after_payout(fee_nq, fee_mnq)
            }
            #[no_mangle]
            pub extern "C" fn init(window_start: i32) {
                __SJXM.with(|s| *s.borrow_mut() = Some(__sjxm_new(window_start)));
            }
            #[no_mangle]
            pub extern "C" fn push(open: f64, high: f64, low: f64, close: f64, tc: i64, minute: i32, weekday: i32) -> i32 {
                __SJXM.with(|s| {
                    let mut state = s.borrow_mut();
                    let st = state.get_or_insert_with(|| __sjxm_new(18));
                    let (legs, fired) = <$strategy as $crate::Strategy>::push(&mut st.strategy, $crate::Bar { open, high, low, close, tc, minute: minute.clamp(0, 1439) as u16, weekday: weekday.clamp(0, 6) as u8 });
                    st.fired = fired.map(|k| k as i32).unwrap_or(-1);
                    st.legs = legs;
                    st.legs.len() as i32
                })
            }
            #[no_mangle]
            pub extern "C" fn fired() -> i32 {
                __SJXM.with(|s| s.borrow().as_ref().map(|st| st.fired).unwrap_or(-1))
            }
            #[no_mangle]
            pub extern "C" fn bars() -> i32 {
                __SJXM.with(|s| s.borrow().as_ref().map(|st| <$strategy as $crate::Strategy>::bars(&st.strategy) as i32).unwrap_or(0))
            }
            #[no_mangle]
            pub extern "C" fn history() -> i32 {
                <$strategy as $crate::Strategy>::HISTORY as i32
            }
            #[no_mangle]
            pub extern "C" fn leg_side(i: i32) -> i32 {
                __sjxm_leg(i, |l| if l.side == $crate::Side::Long { 1 } else { -1 }, 0)
            }
            #[no_mangle]
            pub extern "C" fn leg_tf(i: i32) -> i32 {
                __sjxm_leg(i, |l| l.tf as i32, 0)
            }
            #[no_mangle]
            pub extern "C" fn leg_entry(i: i32) -> f64 {
                __sjxm_leg(i, |l| l.entry, 0.0)
            }
            #[no_mangle]
            pub extern "C" fn leg_extreme(i: i32) -> f64 {
                __sjxm_leg(i, |l| l.extreme, 0.0)
            }
            #[no_mangle]
            pub extern "C" fn leg_atr(i: i32) -> f64 {
                __sjxm_leg(i, |l| l.atr, 0.0)
            }
            #[no_mangle]
            pub extern "C" fn leg_break(i: i32) -> f64 {
                __sjxm_leg(i, |l| l.brk, 0.0)
            }
            #[no_mangle]
            pub extern "C" fn leg_margin(i: i32) -> f64 {
                __sjxm_leg(i, |l| l.margin, 0.0)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Timelike};

    #[test]
    fn new_york_time_follows_daylight_saving() {
        assert_eq!(super::new_york_minute(1780524000), 18 * 60); // 2026-06-01 22:00 UTC = 6:00 pm EDT
        assert_eq!(super::new_york_minute(1768258800), 18 * 60); // 2026-01-12 23:00 UTC = 6:00 pm EST
    }

    #[test]
    fn new_york_time_matches_the_time_zone_database_every_half_hour_from_2007_to_2045() {
        let (mut t, end) = (1_167_609_600, 2_366_841_600); // 2007-01-01 to 2045-01-01 UTC
        while t < end {
            for at in [t, t + 1799] {
                let ny = chrono_tz::America::New_York.timestamp_opt(at, 0).single().unwrap();
                assert_eq!(super::new_york_minute(at), ny.hour() as i64 * 60 + ny.minute() as i64, "at {at}");
            }
            t += 1800;
        }
    }
}
