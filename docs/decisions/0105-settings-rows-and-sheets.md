# 0105. Settings as grouped rows that open small sheets

Status: **accepted** (2026-11-13, owner, from the Remaining Screens mockups; 2026-11-14: "settings
nothing like we designed", so built as drawn).

## Context
Settings was web's form: boxed sections with every field and choice open at once.

## Decision
The same settings and sections, laid out as phone settings usually are.

- The page opens with who you are: the photo, the name, the home area and "Change photo". A
  picked photo is resized and saved at once.
- **Profile** and **Appearance** are groups of rows. A row shows the setting's name, its
  current value and a chevron, and opens a small sheet: a field with "Save profile"
  (Username, Home area, About you), or the choices with a tick on the current one (Theme,
  Type size, Accent with its colour dots, Font). A choice applies at once.
- The other sections (Regional, Notifications, Devices, Network, Privacy, Account) keep
  their controls and take the grouped look: a small caps label over a soft group, one item
  per row, switches at the right.
- On a phone a small sheet (a setting, the feed's and the map's filters) rises from the
  bottom over the page instead of filling the screen, and every sheet's footer is a round
  close button and a pill action.

## Consequences
- UI only. `components/settings/SettingRow.svelte`, `SettingChoice.svelte`;
  `styles/ux/settings.css`, `settingrows.css`, `sheets.css`.
- `accounts.mjs` opens the Home area row, saves in its sheet, and waits for the photo to
  save by itself.
