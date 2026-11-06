## 1. Harness

- [x] 1.1 Add `@happy-dom/global-registrator` and `@testing-library/react`, registered through a `bunfig.toml` preload that also cleans up after each test
- [x] 1.2 Replace `renderToStaticMarkup` and `renderWithQueries` with one mounting helper in every component spec
- [x] 1.3 Remove the `document` guards that only static rendering needed

## 2. Interactions

- [x] 2.1 Reach the modal's delete and unsaved-changes dialogs by clicking, then remove `showDeleteConfirm` and `showUnsavedConfirm`
- [x] 2.2 Cover the unlock: it enables the picker, saving and deleting send the override, and reopening the modal locks it again
- [x] 2.3 Cover the move reconciliation choices and the display toggles by interaction

## 3. Documentation

- [x] 3.1 Record the test layers as an ADR and note the convention in `CLAUDE.md`
