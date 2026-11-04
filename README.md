# PlexEditions

English | [正體中文](README.zh-TW.md)

Writes a movie's edition title through the Plex HTTP API.

Edition 功能的思路受 [edition-manager-for-plex](https://github.com/x1ao4/edition-manager-for-plex) 启发（MIT）。

## What gets written

`EDITION_MODULES` sets the order. A module with no value is omitted. The rest are joined with `EDITION_SEPARATOR` (` · ` when unset).

| Module | Config name | Text |
|---|---|---|
| source | 片源版本 | REMUX, BD, WEB-DL |
| cut | 剪辑版本 | Director's Cut / 导演剪辑版 |
| release | 发行版本 | IMAX Edition / IMAX 版 |
| resolution | 分辨率 | 4K, 1080P |
| dynamicrange | 动态范围 | HDR, DV P7, HDR · DV P7, SDR |
| videocodec | 视频编码 | HEVC |
| framerate | 帧率 | 24P |
| audiocodec | 音频编码 | TRUEHD 7.1 |
| bitrate | 比特率 | 8.5 Mbps; under 1 Mbps, 800 Kbps |
| size | 大小 | 20.00 GB |
| country | 国家 | United States / 美国 |
| contentrating | 内容分级 | PG-13; Not Rated becomes NR |
| rating | 评分 | 8.2 |
| duration | 时长 | 120 MIN / 120 分钟 |

`source`, `cut`, and `release` read the largest file's name, then the embedded video title. `release` keeps every match. A `.strm` name usually matches none of the three. The other modules read that file's Plex metadata.

An existing edition is left unchanged. `reset` clears every non-empty edition in movie libraries, including one entered by hand, and unlocks the field.

## Run

```
PLEX_URL=http://127.0.0.1:32400
PLEX_TOKEN=
EDITION_MODULES=source;resolution;dynamicrange
EDITION_LANG=zh
EDITION_SKIP_LIBRARIES=
EDITION_SEPARATOR= · 
```

`EDITION_MODULES` and `EDITION_SKIP_LIBRARIES` split on `;` or `；`. `EDITION_LANG` is `en`, `zh`, `zh-TW`, `ja`, `ko`, `ar`, `fr`, `ru`, or `es`. Shows are not included. Labels live in `locales/edition.json`.

```
editions all --dry-run
editions all
editions reset
editions new
```

`all` and `reset` exit when the libraries are done. `new` listens on the Plex notification socket and reconnects if it drops. Without that socket, run `all` from cron.

The same settings are in [compose.yaml](compose.yaml), which runs `new` and restarts it. `PLEX_URL` has to be reachable from the container; `127.0.0.1` only works when the container shares the host network. Fill the libraries already scanned with `docker compose run --rm editions all`.

## License

[GPL-3.0](LICENSE)
