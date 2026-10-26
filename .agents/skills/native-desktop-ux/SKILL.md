---
name: native-desktop-ux
description: Use when designing or reviewing Daymark layout, typography, dark colors, navigation, keyboard behavior, accessibility or window sizing.
---

# A quiet native writing surface

1. The writing is the primary content. Keep sidebar calendar/timeline navigation secondary and the page spacious; no fake charts, onboarding funnel or dashboard.
2. Reuse the active theme and embedded DM fonts. Never fetch fonts, icons or imagery while running.
3. Use labeled Freya controls with visible keyboard focus, logical Tab order and contrast. Accessible roles/names are necessary, not proof of screen-reader compatibility.
4. Respect platform Ctrl/Cmd shortcuts. New, search and save/retry must work inside text fields, without consuming ordinary text editing.
5. Search filters the timeline, never the draft. Make empty/no-results states understandable. Focus mode hides navigation, not text or saving feedback.
6. Say saved only after commit. Errors persist with actionable copy; preserve the buffer and block accidental discard. Deletion is explicit and confirmed.
7. The local-only lock does not mean encryption. Privacy copy must say that the SQLite file is plaintext.
8. Check 1360×920 and the 1060×760 minimum, long text/titles, empty/error/confirmation states and focus mode. Scroll the body and timeline independently.
9. Do not remove native window decorations, add broad OS indexing, share journal content, or introduce external services for visual polish.
10. Verify native IME, assistive technology, high DPI and platform behavior manually before promising support. Record gaps rather than implying certification.

References: [design tokens and interactions](../../../docs/DESIGN.md), [GNOME HIG](https://developer.gnome.org/hig/), [Apple HIG](https://developer.apple.com/design/human-interface-guidelines), [Windows design guidance](https://learn.microsoft.com/en-us/windows/apps/design/). Extract applicable principles; do not transplant SwiftUI, web or enterprise-dashboard implementation details into Freya.
