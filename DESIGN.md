# Design

## Look

Ledger: a Swiss, type-led look. Minimal, quiet, dense. Dark theme first with an equally finished light theme, chosen by the system setting unless the user picks one. The interface is chrome-free: the app draws its own title bar of plain text in place of the Windows one, then the two team tables set straight on the window. Hierarchy comes from type size, weight, position and hairline rules, never from boxes or colour. No cards, chips, pills, badges, coloured stripes, uppercase labels, decorative icons or gradients; the player sheet is the only surface with a shadow.

## Colour

- The interface is monochrome: neutral greys with no blue cast. The one accent is ink. Primary buttons, selected segments and switches that are on invert text and background.
- Colour carries meaning only: the tier scale, the semantic tones (`--info`, `--warning`, `--danger`, `--success`), the focus ring (`--focus`), the division tag (`--division`) and the close button's hover red (`--window-close`), which follows the Windows convention.
- The division tag is a vivid sky blue that nothing else uses. Its hue sits in the gap the tier scale leaves, and it is more saturated and greener than the periwinkle of `--info` and `--focus`, so it never reads as a statistic tier or a notice. It meets 4.5:1 on every surface, highlight fill included, in both themes.
- The tier scale keeps the eight community hues from WoWS Numbers (red, orange, yellow, light green, green, teal, pink, purple), softened to near-equal lightness so no single tier shouts. Each tier meets 4.5:1 as text on `--bg`, `--surface-raised` and `--surface-hover` in its theme.
- Teams carry no colour. Allies sit on the left and enemies on the right, each headed by its name.
- Nothing on screen may suggest the user's location. The map name is not shown, and the region always reads as a game server ("NA server").

## Tokens

All colour, spacing, type and motion values live as CSS custom properties in `src/app.css`. Components use the tokens and never inline a new value; a new value goes into `app.css` first.

- Surfaces: `--bg`, `--surface`, `--surface-raised` (the player sheet), `--surface-hover`, `--me-fill` (your own row), `--border` (hairlines), `--border-strong` (header rules, control outlines).
- Text: `--text`, `--text-muted`, `--text-faint`. All three meet 4.5:1 on every surface in both themes.
- Accent: `--accent`, `--accent-strong`, `--accent-contrast` are ink, not a hue. `--ally`, `--enemy` and `--me` stay neutral.
- Division: `--division`, one colour shared by every division tag in both teams.
- Statistic tiers: `--tier-bad`, `--tier-below-average`, `--tier-average`, `--tier-good`, `--tier-very-good`, `--tier-great`, `--tier-unicum`, `--tier-super-unicum`.
- Type: `--font-ui` and `--font-display` (Segoe UI Variable), `--font-mono` (Cascadia Mono). Sizes `--fs-xs` 12, `--fs-sm` 13, `--fs-md` 14, `--fs-lg` 16, `--fs-xl` 22 px, all scaled by `--text-scale`, which the text-size setting changes through `data-text-size`. Weights `--weight-regular` (reading), `--weight-medium` (buttons), `--weight-strong` (titles and your own name only). `--tracking-display` tightens titles. Numbers use the `.num` class for tabular figures.
- Layout: `--row-height` (36 px comfortable, 30 px compact, never under 28 px), changed by `data-density`; `--cell-pad` for table cells; `--measure` for reading columns; `--pip-size` for the row marker; `--titlebar-height` (52 px) and `--window-control-width` (46 px) for the title bar.
- Window: `--window-close` and `--window-close-text` colour the close button on hover, the same in both themes.
- Space: `--space-1` to `--space-6`. Radius: `--radius-sm`, `--radius-md`, `--radius-lg`. Motion: `--duration-fast`, `--duration-base`, `--ease-out`.

## Patterns

- Title bar: one row at `--titlebar-height` holding the title (the match type in battle, otherwise Crowsnest) and the server on the left; the status, refresh and settings on the right; then minimize, maximize and close at the window's edge in the Windows order. The window controls are `--window-control-width` wide, fill the bar's height and show glyphs only, each with an accessible name and a tooltip. Dragging any empty part of the bar moves the window and a double-click maximizes or restores it. The setup screen keeps the status and window controls without a title, and the start-up screen shows only the window controls. The player sheet opens below the bar.
- Team header: the team name at `--fs-xl`, its coverage ("11 of 12 with stats · mean") on the same line at the right, and the team figures on the line below in normal case.
- Table: column groups use booktabs-style spanner rules over "This ship" and "Account"; column heads are 12 px, muted and normal case; rows are separated by hairlines only. The table hangs into the window gutter, so names align with the team title while hover and fill tints keep breathing room.
- Statistic values: right-aligned tabular numbers. Coloured values tint win rate and personal rating only. Damage and frags stay plain because personal rating already scores them against expected values. The tier name is always the cell's title and accessible label, so colour is never the only cue.
- Row highlight: Edge is a small tier-coloured square hanging in the gutter left of the name; Fill tints the row at `--highlight-fill-alpha`; None shows neither.
- Division tag: players who queued together show their division's letter in brackets after the name, `[-REEE] CaptainMidway (A)`, in `--division`. Each team letters its own divisions from A in the order the game numbered them. The tag never truncates; a long name gives way first. Assistive technology hears "division A" instead of the brackets. When Crowsnest cannot read divisions for a battle it shows no tags rather than guessing.
- Ship cell: the tier numeral right-aligned in a fixed slot, the ship name, then the class abbreviation as faint text.
- Player sheet: an Account table and a This ship table, each with All, Solo, Div 2 and Div 3 columns over the same rows, so division play reads against solo play. The ship table shows skeleton cells while ship statistics load and is left out when the player has none for this ship.
- Your own row: `--me-fill` behind the row and your name at `--weight-strong`; the word "You" is there for assistive technology.
- States: waiting, loading, setup, replays-off and error screens are plain left-aligned text in a `--measure` column, with no icon and no card. The connection is stated once, in the title bar; the waiting screen says what it waits for instead: "Waiting for a battle" when connected, "Waiting for the replays folder" when not. Loading uses skeleton cells of the final row height, not spinners.
- Notices: one line above the tables, the icon in the tone colour, the text in body colour and a hairline below.
- Status: a 14 px radar glyph, then "Connected" or "Not connected", then what Crowsnest is doing ("Waiting for battle", "Loading stats", "In battle") or why it is not connected ("Setup needed", "Game not found", "Replays off", "Replays folder unavailable"). Colour states the connection: the glyph is `--success` when connected and `--warning`, with amber text, when not. Motion states the activity: the sweep turns slowly while waiting, quickly while statistics load and stops in battle. Connected means Crowsnest is watching the replays folder for battles, and the tooltip names that folder; Crowsnest does not watch the game process. The text always states both.
- Refresh: the icon spins while statistics load (at least 700 ms, so every press is acknowledged), then the button cools down for 10 seconds with the remaining seconds shown beside it. It uses `aria-disabled`, not `disabled`, so keyboard focus is never lost, and its label says when it will be available again.
- Controls are at least 28 px tall with a visible 2 px focus ring (`--focus`); a switch's hit area extends past its track. Icon-only buttons have an accessible name and a tooltip.
- Errors appear inline next to what failed, never in a modal.
- Motion is short and purposeful: a screen fades up 8 px when you switch views, a new battle's tables fade in, arriving values fade in over their skeletons, the player sheet slides in and out with its content kept while it closes, buttons press down 1 px and icon buttons scale slightly. All motion stops when the system asks for reduced motion.
