# Design system

## Direction

Quiet, literary and dark. A sage-green sidebar supports a spacious dated writing surface, with Daymark's own copy, palette and typography. No calendar, ornamental dashboard or fake analytics.

## Tokens

The authoritative values are in `src/ui/theme.rs`, not this document. Reuse `COLORS` and the initialized Freya theme:

- Near-black green background; slightly raised writing surface; deeper-green selected rows.
- Sage primary accent for the main action, date, focus rings and saved indicator.
- Warm off-white primary text, muted gray-green secondary text, separate placeholder tone.
- Error messages use a warm contrasting surface and explicit text, not color alone.
- Hairline borders; 8–14 px component radii; 18–32 px internal spacing.

Use `Size::flex(1.)` with a `Content::flex()` parent for remaining main-axis space. `expanded()` sets both axes to `fill`; it is not CSS `flex: 1` and can push siblings offscreen.

## Type

- **DM Sans**: navigation, body, labels and controls; body 17 px with generous line height.
- **DM Serif Display**: brand, page heading, title and date numerals.
- Both fonts are embedded and licensed under SIL OFL 1.1; no font CDN or runtime download. See `assets/fonts/`.

Body text is the central element. Avoid tiny low-contrast writing, full-width walls of text, all-caps body copy or decorative icons without accessible context.

## Components

`Action` composes one Freya `Button`, icon and text. `icon`, `caption`, `prose` and `divider` keep shared details consistent. `WritingArea` owns entry-keyed editing state. The sidebar and workspace are stateful components; small stateless visual helpers stay functions.

Use Freya's built-in button, input, focus and scrolling behavior before creating new primitives. Never replace a button with a clickable unlabeled rectangle merely to save styling code.

## Interaction rules

- New entry and search are always available outside focus mode.
- Search narrows the sidebar only. Empty results never erase the open text.
- Save status distinguishes a blank page, saved entry and failed draft.
- Delete is explicit, confirmed and disabled after a failed save.
- Tab navigates; focus rings remain visible. Escape leaves the body field. Platform Ctrl/Cmd+N/F/S work inside title, body and search.
- Input's pre-key policy must allow app shortcuts to reach the root while preserving ordinary text shortcuts and Tab.
- Body has an accessible multiline-input role/name. Do not claim screen-reader certification from this alone.

## Responsive desktop layout

Initial window 1360×920, minimum 1060×760 logical pixels; sidebar 280 px; content width capped at 960 px. Text body and timeline scroll independently. Focus mode removes the sidebar, not the writing surface. Check the minimum size and error/confirmation panels after changing spacing.
