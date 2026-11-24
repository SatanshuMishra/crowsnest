# Product

Crowsnest is a desktop match monitor for World of Warships. When a battle loads, it shows every player on both teams with their ship and their public random-battle statistics, so the player can read the match before the first shot.

## Who uses it

PC players of World of Warships on the EU, NA and Asia servers, usually with the game running full screen on one monitor and Crowsnest on a second monitor, or alt-tabbed during the loading screen. They glance at it for a few seconds at a time.

## Principles

1. Private by construction. No account, no login, no telemetry, no analytics, no backend of our own. Settings and caches stay on the user's machine. The only network calls are to Wargaming's official API for the players in the current battle, to wows-numbers.com for the published expected values that personal rating needs, and to GitHub for the update check, which can be switched off.
2. Fast and light. A native shell with the system webview, no Electron. Idle memory and CPU close to zero between battles. The roster appears within a second of the game writing it.
3. Glanceable. The two teams side by side, one row per player, numbers aligned, colour that reinforces the number rather than replacing it.
4. Works out of the box. It finds the game by itself and says clearly what to do when it cannot.

## Not in scope

Match history kept on a server, second-screen pairing through a cloud relay, clan battle ladders, and anything that needs a Wargaming login.
