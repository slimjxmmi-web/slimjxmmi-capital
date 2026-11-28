# What to tell Claude

Open this folder (`strategy-kit`) in Claude Code, or attach `sdk/src/lib.rs`, `template/src/lib.rs` and `FIRMS.md` to a
chat, and start with the message below. One file per firm: say which firm this one is for. Fill in the part between the lines with your own rules, in plain words. The more exact
you are (numbers, times, what happens after a win or a loss), the less Claude has to guess.

```
You are writing a strategy file for SlimJxmmi Capital, a desktop app that trades NQ / MNQ futures on prop-firm
accounts. The app has no strategy of its own: each firm runs the file I upload for it.

This file is for: <Topstep, Vest or FTMO>.

Read sdk/src/lib.rs, template/src/lib.rs and FIRMS.md (that firm's part above all) before you write anything. Then write my strategy as a new crate in this
workspace, next to template (copy template, give it my strategy's name, add it to the workspace members).

The rules of the kit:
- Rust only, built with: cargo build --release --target wasm32-unknown-unknown -p <name>
- No dependencies but the sdk (sjxm). No files, no network, no clock: the file runs sealed and the app refuses one
  that imports anything.
- push() gets every closed 5-minute NQ bar in order and returns the setups in place and which one fired, if any.
- hours() says whether a signal at that minute and weekday is one I trade. Times are on my clock: the timezone I
  picked in the app. Mine is: <your timezone, as picked in Settings>.
- decide() gets one account (eval or funded, balance, floor, goal, fees, trades so far, winning days, whether it
  traded this session) and returns what that account does with the signal: an order (NQ or MNQ, contracts, target and
  stop in points), Wait, NoRoom, or Done. Use sjxm::points to turn dollars into points on the tick.
- Never size an order the account has no room for: check the room to the floor first and return NoRoom.
- Keep every order inside what FIRMS.md says that firm enforces (contract limits, daily loss limit, Vest's units).
- Write tests for every rule: bars in, the signal out; an account in, the order out. Run cargo test and show me the
  result. Then build the .wasm file and tell me where it is.

---------------------------------------------------------------
MY STRATEGY

When it enters (the exact rule, on 5-minute bars):

Long, short or both:

The hours and days it trades:

How many contracts, NQ or MNQ:

Target and stop (points or dollars):

How many trades a day on each account:

What changes between an eval and a funded account, if anything:

When it is finished with an account:
---------------------------------------------------------------

Before you write code, tell me in plain words what you understood, and ask me about anything my rules leave open.
Do not fill a gap with your own idea of a good strategy.
```

When Claude is done you have a `.wasm` file. Upload it in the app while flat (Settings, Firms and strategy), keep the
app paused, and read what the dashboard says each account would do before you take the pause off.

To change the strategy later, tell Claude what to change, build again and upload the new file.
