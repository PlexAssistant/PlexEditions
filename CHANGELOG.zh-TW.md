# 變更紀錄

## [0.1.3] — 2026-11-04

### 新增

- crate 長出 library target，PlexAssistant 以依賴引用同一份邏輯。`editions` 二進位的行為不變。
- `Config::from_env_with_url` 讓未設定的 `PLEX_URL` 回退到傳入的預設位址，給與 Plex 同機的嵌入方用；`Config::from_env` 仍然必須明確設定。
- `startup::await_ready` 是 `editions new` 等待未綁定或未啟動伺服器的重試迴圈。

## [0.1.2] — 2026-11-04

### 新增

- `editions new` 在開始監聽前，先為還沒有 Edition 的電影寫入，連線中斷 30 分鐘後再做一次。
- `PLEX_TOKEN` 留空時，從 `Preferences.xml` 讀取 token。把 Plex 的資料目錄掛到 `/config`，或用 `PLEX_CONFIG_DIR` 指向存放該檔的目錄。
- `compose.yaml` 用來跑這個映像。它把 Plex 的資料目錄以唯讀掛上，token 留空。

### 變更

- 還讀不到 token，或伺服器沒有回應時，`editions new` 每 30 分鐘重試一次，不再退出。

## [0.1.1] — 2026-11-04

### 修復

- 發佈的映像帶著依賴快取層的樁二進位，任何命令都靜默退出。現在強制重編本 crate。
- 檔名裡的剪輯、發行關鍵字不再區分大小寫（`EXTENDED` 視同 Extended）。
- `editions new` 改聽媒體庫 timeline 幀（`itemID`、type 1），這才是伺服器實際發送的事件形狀；寫入會重試到媒體分析完成。

## [0.1.0] — 2026-11-04

### 新增

- `editions all` 為還沒有 Edition 的電影寫入。
- `editions reset` 清除電影庫裡的 Edition，並解除鎖定。
- `editions new` 監聽新電影，連線中斷後重連。
- `--dry-run` 印出將寫入的 Edition，不改伺服器。
- `EDITION_LANG` 接受 `en`、`zh`、`zh-TW`、`ja`、`ko`、`ar`、`fr`、`ru`、`es`。譯名在 `locales/edition.json`。

[0.1.3]: https://github.com/PlexAssistant/PlexEditions/releases/tag/v0.1.3
[0.1.2]: https://github.com/PlexAssistant/PlexEditions/releases/tag/v0.1.2
[0.1.1]: https://github.com/PlexAssistant/PlexEditions/releases/tag/v0.1.1
[0.1.0]: https://github.com/PlexAssistant/PlexEditions/releases/tag/v0.1.0
