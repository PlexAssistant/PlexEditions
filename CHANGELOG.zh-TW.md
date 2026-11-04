# 變更紀錄

## [Unreleased]

### 新增

- `editions all` 為還沒有 Edition 的電影寫入。
- `editions reset` 清除電影庫裡的 Edition，並解除鎖定。
- `editions new` 監聽新電影，連線中斷後重連。
- `--dry-run` 印出將寫入的 Edition，不改伺服器。
- `EDITION_LANG` 接受 `en`、`zh`、`zh-TW`、`ja`、`ko`、`ar`、`fr`、`ru`、`es`。譯名在 `locales/edition.json`。
