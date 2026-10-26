# UI rules

- Native Freya only. No calendar yet. Keep the timeline, search and writing surface central.
- Reuse theme/components and offline fonts. No scattered palette, unlabeled click-only controls or hidden focus.
- Stateful reusable UI is a keyed `PartialEq` component; hooks have stable order. Inspect Freya 0.5.0-rc.13 source before using main-branch examples.
- Search never replaces the draft. Entry switches reset editor history by identity.
- Test shortcuts inside inputs and check the minimum window, error/confirmation states and focus mode.
- Mark known native accessibility/IME limitations honestly.

Details: [DESIGN.md](../../docs/DESIGN.md), [TESTING.md](../../docs/TESTING.md).
