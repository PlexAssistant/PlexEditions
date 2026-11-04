# Changelog

## [0.1.2] — 2026-11-04

### Added

- `editions new` writes editions for movies that do not have one before it listens, and again 30 minutes after the connection drops.
- The Plex token is read from `Preferences.xml` when `PLEX_TOKEN` is empty. Mount the Plex data directory at `/config`, or point `PLEX_CONFIG_DIR` at the directory that holds the file.
- `compose.yaml` runs the image. It mounts the Plex data directory read-only and leaves the token unset.

### Changed

- `editions new` waits 30 minutes instead of exiting when the token cannot be read yet or the server does not answer.

## [0.1.1] — 2026-11-04

### Fixed

- The published image carried the stub binary from the dependency-caching layer, so every command exited silently. The crate is now forced to rebuild.
- Cut and release tokens in file names now match regardless of case (`EXTENDED` reads as Extended).
- `editions new` now follows library timeline frames (`itemID`, type 1), the shape this server actually sends; an edition is retried until the media analysis finishes.

## [0.1.0] — 2026-11-04

### Added

- `editions all` writes an edition for each movie that does not have one.
- `editions reset` clears editions in movie libraries and unlocks the field.
- `editions new` listens for new movies and reconnects after a dropped socket.
- `--dry-run` prints the edition that would be written.
- `EDITION_LANG` accepts `en`, `zh`, `zh-TW`, `ja`, `ko`, `ar`, `fr`, `ru`, and `es`. Labels are in `locales/edition.json`.

[0.1.2]: https://github.com/PlexAssistant/PlexEditions/releases/tag/v0.1.2
[0.1.1]: https://github.com/PlexAssistant/PlexEditions/releases/tag/v0.1.1
[0.1.0]: https://github.com/PlexAssistant/PlexEditions/releases/tag/v0.1.0
