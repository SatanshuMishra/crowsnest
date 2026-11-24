# 0001: Stack and data sources

Status: accepted, 2026-11-23

## Context

Crowsnest replaces wows-monitor, an Electron and Angular desktop app whose stats, settings and update feed all ran through the vendor server at api.wows-monitor.com. That server was shut down, which stopped the app from working at all: it would not start watching the game until the server answered. Crowsnest must keep user data on the machine, look modern, stay light, find the game by itself, and ship versioned releases with in-app updates.

## Decision

| Concern | Choice | Why |
|---|---|---|
| Shell | Tauri 2 (Rust, system WebView2) | Installer of a few MB and a small fraction of Electron memory, because the webview is shared with Windows. First-party updater with signature checks, single-instance, tray and window-state plugins. |
| UI | Svelte 5 with TypeScript and Vite, plain CSS tokens | Compiles to small vanilla JS with no virtual DOM; runes keep state simple. No CSS framework, so the bundle stays tens of KB. |
| Backend work | Rust: file watching (notify), XML (roxmltree), HTTP (reqwest with rustls), registry (winreg) | All file and network access happens in Rust, so the webview has no network permission at all (CSP `connect-src` is IPC only). |
| Player stats | Wargaming public API, called directly | Official source; no middleman server. Standalone application type, about 10 requests per second, rate-limited client side. |
| Personal rating | wows-numbers.com expected values, cached daily on disk | Public file with a documented formula; attribution link shown in the app. |
| Ship names | Wargaming encyclopedia, cached on disk per game version | One fetch per game patch. |
| Updates | tauri-plugin-updater reading `latest.json` from GitHub Releases, minisign-signed | Updates are verified against a public key compiled into the app; no update server to run. |
| Versioning | Conventional Commits, release-please, Tauri CLI signer | Version, changelog, tag, installer and update manifest are all produced by CI from the commit history. The installer is built in one job and signed in another, so build-time dependencies never see the signing key. |

## Data that leaves the machine

- To Wargaming: the names (then account ids) of the players in the current battle, and the application id.
- To wows-numbers.com: a plain download of the expected-values file, at most once a day. Nothing is sent.
- To GitHub: an update check at start-up, which can be turned off.

Nothing else. There is no telemetry, analytics, crash reporting, account or device identifier. Player statistics are held in memory only and are never written to disk.

## wows-toolkit

wows-toolkit (MIT) was evaluated as a replacement for parts of the old app. Its install-detection approach (Game Center preferences, uninstall registry entries, Steam library manifests) informs Crowsnest's own detection code, because the old app had none. Its live roster reader, which decodes the in-progress replay stream to get account ids and platoons, was not adopted: Crowsnest gets account ids from the official API by name, and the only gain would be platoon markers, at the cost of parsing game-internal entity definitions that change with game patches. Its own match-stats service is a private server and is not used.

## Consequences

- Release builds need a Wargaming application id. It is injected at build time from the `WG_APPLICATION_ID` repository secret and never committed, as Wargaming asks for the id to be kept secure. Users can paste their own id in Settings instead.
- The Russian server (Lesta, Mir Korabley) uses a different API and is not supported.
- Windows is the only release target, because the game client is Windows-only.
