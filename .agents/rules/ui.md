# UI rules

- Native Freya only. Keep calendar navigation compact and the timeline, search and writing surface central; no backdating.
- Reuse theme/components and offline fonts. No scattered palette, unlabeled click-only controls or hidden focus.
- Use concise, functional UI copy: name the action, state or consequence. Do not add slogans, motivational filler, cozy marketing language or redundant branding (for example, “Just you. Just this device.” or “A little space for your days”). Prefer removing decorative text to replacing it. Keep factual privacy/save/error messages; writing prompts should be direct, not sentimental.
- Stateful reusable UI is a keyed `PartialEq` component; hooks have stable order. Inspect Freya 0.5.0-rc.13 source before using main-branch examples.
- Search never replaces the draft. Entry switches reset editor history by identity.
- Test shortcuts inside inputs and check the minimum window, error/confirmation states and focus mode.
- Mark known native accessibility/IME limitations honestly.

Details: [DESIGN.md](../../docs/DESIGN.md), [TESTING.md](../../docs/TESTING.md).
