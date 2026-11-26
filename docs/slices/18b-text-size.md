# Adjustable text size

The **Text size** selector offers 100%, 125%, 150% and 200% on the main menu, in private rooms, in local play and in the practice inspector. Both match menus also expose it. The browser remembers the choice across routes and reloads. Other tabs on the same origin follow changes. Players in separate browser profiles choose independently.

CSS applies each percentage to the root font, relative to the user’s browser default. Returning to 100% restores the ordinary stylesheet. HUD text, settings, scores and form controls scale with the root. The WebGL camera continues to frame its canvas at the available size. The versioned preference `skyline.display.v1.text-percent` accepts only the four displayed values. Missing or unrecognized values use the default, and denied storage leaves the choice usable in memory. [Text-size preference](../../web/src/settings/textSize.ts)

## Narrow layouts

On screens up to 720 CSS pixels wide, enlarged local play places the score and pause button beneath the canvas. Round information and fighter scores can wrap onto separate rows. The score stays accessible with Tab, and Escape returns keyboard control to the arena. The pause dialog scrolls vertically. At narrow widths each volume slider moves below its label and value. The private canvas follows its container through resizes, including when the player changes text size during a round. [Local scoreboard at 200% on a 320-pixel screen](../verification/large-text-scoreboard.png)

The main menu’s links work with JavaScript disabled. A small optional script adds the selector and reads the saved percentage. A fresh Chromium navigation on the macOS production preview transferred 61,873 encoded bytes for the menu, including the 58,680-byte original still and 1,071 bytes of JavaScript. Selecting a size loads no simulation or room session.

## Verification

The [browser regressions](../../web/tests/text-size.spec.ts) measure computed font sizes during a live round. They verify all four choices across two tabs, a keyboard selection on the main menu, reload persistence, and the private guest’s independent scoreboard. At 320 pixels wide they check that the local canvas and scoreboard occupy separate vertical space, that the scores and controls stay inside the page, and that the pause dialog can show its longest camera option. The tests also return focus to the canvas and confirm that the private clock keeps advancing.

The release build passed 115 web unit tests and TypeScript. Chromium passed all eleven text-size, main-menu, match-menu and scoreboard checks. Firefox and Playwright WebKit each passed all three text-size scenarios on the local HTTPS build.

Reproduce with `npm --prefix web run test:e2e -- text-size.spec.ts main-menu.spec.ts match-menu.spec.ts scoreboard.spec.ts`.
