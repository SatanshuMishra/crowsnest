# Design

## Look

Minimal, quiet, dense. Dark theme first with an equally finished light theme, chosen by the system setting unless the user picks one. The interface is chrome-light: a slim top bar, then the two team tables filling the window. No cards inside cards, no decorative gradients, no drop shadows except on popovers.

## Tokens

All colour, spacing, type and motion values live as CSS custom properties in `src/app.css`. Components use the tokens and never inline a new value; a new value goes into `app.css` first.

- Surfaces: `--bg`, `--surface`, `--surface-raised`, `--surface-hover`, `--border`, `--border-strong`.
- Text: `--text`, `--text-muted`, `--text-faint`. Body text meets 4.5:1 on every surface in both themes.
- Accent: `--accent` (teal, from the logo). Team colours: `--ally`, `--enemy`, `--me`.
- Statistic tiers: `--tier-bad`, `--tier-below-average`, `--tier-average`, `--tier-good`, `--tier-very-good`, `--tier-great`, `--tier-unicum`, `--tier-super-unicum`. Each meets 4.5:1 as text on `--surface` and `--surface-raised` in its theme.
- Type: `--font-ui` (Segoe UI Variable), sizes `--fs-xs` to `--fs-xl`, all scaled by `--text-scale`, which the text-size setting changes through `data-text-size` on the root element. Numbers use the `.num` class for tabular figures.
- Density: `--row-height`, changed by `data-density`.
- Space: `--space-1` to `--space-6`. Radius: `--radius-sm`, `--radius-md`, `--radius-lg`. Motion: `--duration-fast`, `--duration-base`, `--ease-out`; all motion respects reduced-motion.

## Patterns

- Statistic values: right-aligned tabular numbers. When coloured values are on, the value takes its tier colour; the tier name is always available as the cell's title and accessible label, so colour is never the only cue.
- Row highlight: either a 3 px left edge in the tier colour, a tinted fill at `--highlight-fill-alpha`, or none, driven by the chosen statistic.
- The player's own row carries a visible outline and the word "You" for assistive technology.
- Empty, loading, error and unavailable states are designed for every view. Loading uses skeleton rows of the final row height, not spinners.
- Controls are at least 28 px tall with a visible focus ring (`--focus`). Icon-only buttons have an accessible name and a tooltip.
- Errors appear inline next to what failed, never in a modal.
