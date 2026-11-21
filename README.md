# SlimJxmmi Capital

A Windows desktop app that trades NQ and MNQ futures on your Topstep accounts, from your own PC, with your own keys,
by the strategy file you give it. The app carries no strategy of its own, and nothing you type into it leaves your PC
except to Topstep and to your data provider.

**[Download SlimJxmmiCapital-setup.exe](https://github.com/slimjxmmi-web/slimjxmmi-capital/raw/main/SlimJxmmiCapital-setup.exe)**

## What you need

- Windows 10 or 11.
- A license key. It is tied to your email and comes from the owner.
- Topstep accounts, and the ProjectX API for them (TopstepX, Settings, API; the code `topstep` takes 50% off). The
  app signs in with your ProjectX username and API key, never your password.
- Prices, from one of two places (Settings, API keys, **Prices from**):
  - **Databento** (recommended): an API key with live CME futures data (dataset GLBX.MDP3). The cleanest feed, and
    the one backtesting uses.
  - **TopstepX**: the cheaper option. The same ProjectX API login the app trades with, so no second subscription.
- A strategy file (`.wasm`). Yours: see [strategy-kit](strategy-kit/), which also holds what to tell Claude so it
  writes one from your rules.

**Want to try it first?** Open **DEMO** in the app and upload your strategy file there: paper accounts trade it against
the live price, with no money at risk. It needs only a price feed (a Databento key, or your TopstepX login). The demo
keeps its own calendar, marked DEMO.

## Install

1. Download the installer above and run it. Windows may say it protected your PC, because the installer is not
   signed with a paid certificate: choose **More info**, then **Run anyway**.
2. Open the app. Type your email and license key, then read and accept the agreement. That is all the first start asks.
3. Add your keys in Settings, API keys: your Databento key for the prices (recommended), and your TopstepX username
   and API key for trading. With no Databento key, the prices can come from TopstepX instead (Prices from).
4. Forward-test first if you like: **DEMO** in the sidebar. For your firm: the app starts **paused, with no accounts**. Add the accounts it found (Accounts), upload your strategy file (Settings,
   Firms and strategy), read what the dashboard says each account would do, and only then take the pause off.

New versions are offered by the app itself, in its own window. Update while flat.

## Know what this is

Futures trading can lose money, and an automated strategy can lose it fast. The app places the orders your strategy
file tells it to place: what it trades, how big, and when is your decision and your responsibility. Nothing here is
financial advice. Test a strategy on history and on an eval account before it touches a funded one, and follow your
firm's rules.

## On a Mac

The same app for a Mac with an Apple chip (M1 or later). Intel Macs are not supported. Your license key works on both.

**[Download SlimJxmmiCapital-mac.zip](https://github.com/slimjxmmi-web/slimjxmmi-capital/raw/main/SlimJxmmiCapital-mac.zip)**

1. Open the zip if your browser did not, and drag **SlimJxmmi Capital** into your **Applications** folder.
2. Open it. The first time, macOS refuses, because the app is not signed with a paid Apple certificate: click **Done**.
3. Open **System Settings**, **Privacy & Security**, scroll down to **Security**, and click **Open Anyway** next to
   SlimJxmmi Capital. Confirm with your password.
4. From then on it opens like any app, and it offers its own updates.

If macOS says the app "is damaged", open Terminal, run this, and open the app again:

```
xattr -dr com.apple.quarantine "/Applications/SlimJxmmi Capital.app"
```

A sleeping Mac trades nothing: keep it awake, and plugged in, while you trade.
