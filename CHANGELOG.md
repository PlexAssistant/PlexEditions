# Changelog

## [0.1.0] — 2026-11-04

### Added

- `editions all` writes an edition for each movie that does not have one.
- `editions reset` clears editions in movie libraries and unlocks the field.
- `editions new` listens for new movies and reconnects after a dropped socket.
- `--dry-run` prints the edition that would be written.
- `EDITION_LANG` accepts `en`, `zh`, `zh-TW`, `ja`, `ko`, `ar`, `fr`, `ru`, and `es`. Labels are in `locales/edition.json`.

[0.1.0]: https://github.com/PlexAssistant/PlexEditions/releases/tag/v0.1.0
