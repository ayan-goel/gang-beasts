# In-match settings

Escape on a focused arena or the on-screen button opens a native modal dialog. The return button receives initial focus. Graphics and mute use the same preferences as the page header. Local players can also review both sets of key bindings. Escape or the return button closes the dialog.

| Mode | While the menu is open | On return |
| --- | --- | --- |
| Local | The existing pause operation clears both fighters’ inputs and freezes simulation time. | The same round resumes and the canvas takes focus. |
| Private | Moving focus off the canvas releases held input through the existing network sampler. The server keeps simulating the room. | An available canvas takes focus and accepts fresh input. |

The dialog is mounted outside the arena so ordinary menu keystrokes cannot reach its spectator camera listener. The browser manages modal focus and background interaction. The private menu has no server command. [Menu component](../../web/src/app/MatchMenu.tsx) · [Remote controls](../../web/src/online/RemoteControls.ts)

## Verification

Both [menu tests](../../web/tests/match-menu.spec.ts) failed against the preceding production build because Escape left the arena focused. Both passed on the updated production build in Chromium, Firefox and Playwright WebKit on macOS. The private-room test inspected actual outgoing movement and hand bits, watched both clocks advance during the menu and sent fresh movement after returning to play. The local test changed the graphics preset during a frozen clock interval, resumed with Escape and reopened the menu with the Pause button.

All 103 web unit tests, type checking and the production build passed. Chromium also passed four graphics recovery checks, a local ring-out into a second round and the narrow-screen start check on the packaged build. Two additional tests import the generated WASM source directly. They failed to load that development URL in production preview and passed on the development server, covering walkway reset and a match-winning rematch. [Local match tests](../../web/tests/skyline.spec.ts)
