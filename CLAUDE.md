# Crowsnest

Crowsnest is a Tauri 2 and Svelte 5 desktop app for Windows. DESIGN.md holds the design system and its tokens; read it before changing anything a player sees.

## Announcing what changed in a release

Every release with a change a player can see announces it on the What's new pages of the onboarding card, `src/components/Onboarding.svelte`. They are the only place a release shows its changes: add no other changelog dialog, sheet, banner, popover or toast for that purpose. The owner approved this design as it stands, so reuse it exactly and change its look only when the owner asks.

### Add a release

1. In `src/lib/intro.ts`, append one entry to `RELEASES` whose `version` is one more than the last entry's. The card shows the pages of every release newer than the `introSeen` value in the player's settings, then saves the newest `version` when it closes. `INTRO_VERSION` follows the last entry on its own, and no Rust change is needed.
2. Give the entry one `features` item per change a player would notice:
   - `title`: the feature's name in two or three words, sentence case, no full stop, such as Keep on top.
   - `body`: one plain sentence saying what the player can now do and where, such as The pin in the title bar keeps Crowsnest above the game, without a trip to Settings.
   - `hero`: `{ kind: "glyphs", icons: ["<icon>"] }` for one glyph tile, drawn in `src/components/ui/Icon.svelte` in its 24 by 24 stroke style. When a real piece of the interface explains the feature better, add a hero kind that renders it, as `palettes` does.
   - `wash`: teal, violet, amber or green, varied so neighbouring pages differ.
3. Set `appearanceSteps: false`. Release 1 set it so everyone met the appearance steps once; set it again only when the owner asks for everyone to see them.
4. Cover the new pages in `src/lib/intro.test.ts`, then run `npm test` and `npm run check`.
5. Look at the card in a browser preview at 1280 by 760 and at the minimum window, 960 by 540, in the dark and light themes.

### The design to keep

- One modal card over a `--scrim`, at `--onboarding-width` by `--onboarding-height` with `--radius-xl` corners, rising and fading in and leaving faster than it came.
- Each page: a hero on top that fills the free height, washed with two soft radial tints that fade into the card and holding the feature's visual. Below it, left-aligned: the kicker What's new, the title at `--fs-2xl`, and one muted line. The kicker never names a version, since one card can cover several releases.
- The footer: page dots on the left with the current one elongated; on the right, Skip on the first page or Back after it, then a primary Continue that reads Done on the last page.
- Never show the app icon or logo in the card.
- Installing and updating are treated alike: everyone sees the same pages, with no separate welcome.
- The card opens only on the monitor between battles, closes unseen if a battle starts, and returns at the next quiet moment.
- When a release asks for the appearance steps, they follow the What's new pages in the same card, which widens to `--onboarding-wide-width`. They keep their own two-column layout, options on the left and the full player-table preview on the right; do not fold them into the hero layout.
