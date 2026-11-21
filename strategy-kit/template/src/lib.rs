//! A strategy to start from. It holds the bars, fires nothing and places nothing: write your rule in `push`, your
//! hours in `hours` and what each account does in `decide`, build it, and upload the .wasm file in the app (Settings,
//! Firms and strategy).
//!
//! The app has no plan of its own. When to trade, how many contracts, what target and what stop, and when you are
//! finished with an account are yours to decide, here: until `decide` is written, a signal places nothing and the
//! dashboard says so. Times are on the clock of the timezone picked in the app: the platform and this code use the
//! same one.
//!
//! ```text
//! cargo test -p template
//! cargo build --release --target wasm32-unknown-unknown -p template
//! ```
use sjxm::{Bar, Leg, Side, Strategy};

pub struct Mine {
    bars: Vec<Bar>,
    #[allow(dead_code)] // yours to use in `hours`, or not
    window_start: i64,
}

impl Strategy for Mine {
    const HISTORY: usize = 20;

    fn new(window_start: i32) -> Self {
        Mine { bars: Vec::new(), window_start: window_start as i64 }
    }

    fn push(&mut self, bar: Bar) -> (Vec<Leg>, Option<usize>) {
        self.bars.push(bar);
        if self.bars.len() < Self::HISTORY {
            return (Vec::new(), None);
        }
        // Your rule. `self.bars` is every 5-minute bar so far, the last one just closed; `bar.minute` and `bar.weekday`
        // are its close on your clock (9:30 am is minute 570; Monday is 0). To fire, return one leg and its index:
        //
        //     let leg = Leg { side: Side::Long, tf: 5, entry: bar.close, extreme: bar.low, atr: 0.0, brk: 1.0, margin: 1.0 };
        //     return (vec![leg], Some(0));
        let _ = Side::Long;
        (Vec::new(), None)
    }

    fn bars(&self) -> usize {
        self.bars.len()
    }

    // Your hours. Asked once for every signal before any account is looked at: return false and the signal is passed
    // over. As it comes, every hour the market is open is yours. `self.window_start` is WINDOW_START from the settings
    // file (18 unless set), for you to use or not.
    //
    //     fn hours(&self, minute: u16, weekday: u8) -> bool {
    //         (570..960).contains(&minute) && weekday < 5        // 9:30 am to 4:00 pm, Monday to Friday
    //     }

    // Your plan. The desk asks this for every account a signal could go to, hands over what it knows of the account
    // (`Account`: eval or funded, balance, floor, goal, fees, its trades so far, its winning days, whether it traded
    // today) and does exactly what comes back, never more contracts than the firm allows. It is empty as it comes:
    // remove the comment marks and put your own numbers in. `use sjxm::{points, Account, Order, Plan, What, MNQ};` at
    // the top for the names used here.
    //
    //     fn decide(a: &Account) -> Plan {
    //         let (qty, risk, reward) = (2, 200.0, 400.0);        // 2 MNQ, $200 at risk for $400, before fees
    //         let room = a.balance - a.floor - qty as f64 * a.fee_mnq;
    //         let what = if a.traded_today > 0 {
    //             What::Wait                                      // one trade a session on each account
    //         } else if room < risk {
    //             What::NoRoom { room, needs: risk }              // too close to the floor: it sits out, and the desk says why
    //         } else {
    //             What::Trade(Order {
    //                 micro: true,                                // MNQ; false is NQ
    //                 qty,
    //                 target: points(reward, qty, MNQ, true),     // dollars to points, on the tick
    //                 stop: points(risk, qty, MNQ, false),
    //                 net: reward - qty as f64 * a.fee_mnq,       // what the dashboard shows the trade is for
    //                 room,
    //             })
    //         };
    //         Plan { label: "day", priority: 0, what }            // the label is what the board calls where the account stands
    //     }
    //
    // `What::Done` tells the desk you are finished with an account for good. When several accounts could take a signal
    // the lowest `priority` goes first. One more answer is yours when you want it, zero as it comes:
    // `keep_after_payout` (room a payout request leaves in a funded account).
}

sjxm::export!(Mine);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_template_holds_its_bars_and_fires_nothing() {
        let mut s = Mine::new(18);
        for k in 0..40 {
            let (legs, fired) = s.push(Bar { open: 1.0, high: 2.0, low: 0.5, close: 1.5, tc: 1_780_527_600 + 300 * k, minute: 0, weekday: 0 });
            assert!(legs.is_empty() && fired.is_none());
        }
        assert_eq!(s.bars(), 40);
    }

    #[test]
    fn the_template_places_nothing_and_closes_no_hour_until_its_author_does() {
        let account = sjxm::Account::from_facts(&[0.0, 50_000.0, 48_000.0, 3_000.0, 50_000.0, 3.78, 1.22, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]);
        assert_eq!(Mine::decide(&account), sjxm::Plan::UNSIZED);
        assert!(Mine::new(18).hours(0, 6) && Mine::new(18).hours(570, 0));
        assert_eq!(Mine::keep_after_payout(3.78, 1.22), 0.0);
    }
}
