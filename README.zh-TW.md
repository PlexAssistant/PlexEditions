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
EDITION_MODULES=source;resolution;dynamicrange
EDITION_LANG=zh
EDITION_SKIP_LIBRARIES=
EDITION_SEPARATOR= · 
```

`EDITION_MODULES` 與 `EDITION_SKIP_LIBRARIES` 以 `;` 或 `；` 分隔。`EDITION_LANG` 為 `zh` 或 `en`。劇集不處理。設定名是簡體，與程式接受的字串一致。

```
editions all --dry-run
editions all
editions reset
editions new
```

`all` 與 `reset` 跑完即結束。`new` 聽 Plex 的通知 socket，斷線後重連。沒有這條 socket 時，用 cron 跑 `all`。

## 授權

[GPL-3.0](LICENSE)
