# PlexEditions

[English](README.md) | 正體中文

透過 Plex 的 HTTP API，把選定模組寫進電影的 Edition。

Edition 功能的思路受 [edition-manager-for-plex](https://github.com/x1ao4/edition-manager-for-plex) 啟發（MIT）。

## 寫入內容

順序由 `EDITION_MODULES` 決定。沒有值的模組略過，其餘以 `EDITION_SEPARATOR` 相接，未設定時是 ` · `。

| 模組 | 設定名 | 寫入 |
|---|---|---|
| source | 片源版本 | REMUX、BD、WEB-DL |
| cut | 剪辑版本 | Director's Cut / 導演剪輯版 |
| release | 发行版本 | IMAX Edition / IMAX 版 |
| resolution | 分辨率 | 4K、1080P |
| dynamicrange | 动态范围 | HDR、DV P7、HDR · DV P7、SDR |
| videocodec | 视频编码 | HEVC |
| framerate | 帧率 | 24P |
| audiocodec | 音频编码 | TRUEHD 7.1 |
| bitrate | 比特率 | 8.5 Mbps；低於 1 Mbps 時為 800 Kbps |
| size | 大小 | 20.00 GB |
| country | 国家 | United States / 美國 |
| contentrating | 内容分级 | PG-13；Not Rated 寫成 NR |
| rating | 评分 | 8.2 |
| duration | 时长 | 120 MIN / 120 分鐘 |

`source`、`cut`、`release` 先讀最大檔的檔名，再讀內嵌影片標題。`release` 保留每一個命中。`.strm` 的檔名通常三者都對不上。其餘模組讀該檔的 Plex 中繼資料。

已有 Edition 的電影不改。`reset` 清除電影庫裡每一筆非空的 Edition，包含手動輸入的，並解除鎖定。

## 執行

```
PLEX_URL=http://127.0.0.1:32400
PLEX_TOKEN=
PLEX_CONFIG_DIR=
EDITION_MODULES=source;resolution;dynamicrange
EDITION_LANG=zh
EDITION_SKIP_LIBRARIES=
EDITION_SEPARATOR= · 
```

`EDITION_MODULES` 與 `EDITION_SKIP_LIBRARIES` 以 `;` 或 `；` 分隔。`EDITION_LANG` 為 `en`、`zh`、`zh-TW`、`ja`、`ko`、`ar`、`fr`、`ru`、`es`。劇集不處理。設定名是簡體，與程式接受的字串一致。譯名在 `locales/edition.json`。

`PLEX_TOKEN` 留空時，從 `PLEX_CONFIG_DIR` 下的 `Preferences.xml` 讀取 `PlexOnlineToken`。未設定該目錄時，讀 `/config/Library/Application Support/Plex Media Server`，[compose.yaml](compose.yaml) 就是把 Plex 的資料目錄掛在這裡。環境變數裡給了 token 就直接用。

```
editions all --dry-run
editions all
editions reset
editions new
```

`all` 與 `reset` 跑完即結束。`new` 會一直跑：連上之前，以及 socket 斷線 30 分鐘後，先為還沒有 Edition 的電影補上，再開始監聽。監聽期間掃進的電影，在 Plex 回報時寫入。還讀不到 token 或伺服器沒有回應時，也是每 30 分鐘重試一次，不會退出，所以容器可以比 Plex 完成登入或啟動更早起來。

`PLEX_URL` 必須從容器內連得到。`127.0.0.1` 指向容器自己，除非容器與宿主共用網路。

## 授權

[GPL-3.0](LICENSE)
