# 變更紀錄

## [Unreleased]

### 修復

- 發佈的映像帶著依賴快取層的樁二進位，任何命令都靜默退出。現在強制重編本 crate。
- 檔名裡的剪輯、發行關鍵字不再區分大小寫（`EXTENDED` 視同 Extended）。

## [0.1.0] — 2026-11-04

### 新增

- `editions all` 為還沒有 Edition 的電影寫入。
- `editions reset` 清除電影庫裡的 Edition，並解除鎖定。
- `editions new` 監聽新電影，連線中斷後重連。
- `--dry-run` 印出將寫入的 Edition，不改伺服器。
- `EDITION_LANG` 接受 `en`、`zh`、`zh-TW`、`ja`、`ko`、`ar`、`fr`、`ru`、`es`。譯名在 `locales/edition.json`。

[0.1.0]: https://github.com/PlexAssistant/PlexEditions/releases/tag/v0.1.0
