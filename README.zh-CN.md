# PlexEditions

[English](README.md) | 简体中文 | [正體中文](README.zh-TW.md)

通过 Plex 的 HTTP API，把选定的模块写进电影的 Edition。只处理电影库，剧集不涉及。

Edition 功能的思路受 [edition-manager-for-plex](https://github.com/x1ao4/edition-manager-for-plex) 启发（MIT）。

## 写入内容

顺序由 `EDITION_MODULES` 决定，模块写英文 id 或中文设置名均可。没有值的模块跳过，其余以 `EDITION_SEPARATOR` 连接，未设置时是 ` · `。

| 模块 | 设置名 | 写入 |
|---|---|---|
| source | 片源版本 | REMUX、BD、WEB-DL |
| cut | 剪辑版本 | Director's Cut / 导演剪辑版 |
| release | 发行版本 | IMAX Edition / IMAX 版 |
| resolution | 分辨率 | 4K、1080P |
| dynamicrange | 动态范围 | HDR、DV P7、HDR · DV P7、SDR |
| videocodec | 视频编码 | HEVC |
| framerate | 帧率 | 24P |
| audiocodec | 音频编码 | TRUEHD 7.1 |
| bitrate | 比特率 | 8.5 Mbps；低于 1 Mbps 时为 800 Kbps |
| size | 大小 | 20.00 GB |
| country | 国家 | United States / 美国 |
| contentrating | 内容分级 | PG-13；Not Rated 写成 NR |
| rating | 评分 | 8.2 |
| duration | 时长 | 120 MIN / 120 分钟 |

`source`、`cut`、`release` 先读最大文件的文件名，再读内嵌视频标题。`release` 保留每一个命中。`.strm` 的文件名通常三者都匹配不上。其余模块读该文件的 Plex 元数据。

已有 Edition 的电影不改。`reset` 清除电影库里每一条非空的 Edition，包括手动输入的，并解除锁定。

## 环境变量

| 变量 | 含义 |
|---|---|
| `PLEX_URL` | Plex 服务地址，须从容器内连得到。`127.0.0.1` 指向容器自己，除非与宿主共用网络。 |
| `PLEX_TOKEN` | Plex token。环境变量里给了就直接用；留空时从 `PLEX_CONFIG_DIR` 下的 `Preferences.xml` 读 `PlexOnlineToken`。 |
| `PLEX_CONFIG_DIR` | 存放 `Preferences.xml` 的目录。未设时读 `/config/Library/Application Support/Plex Media Server`，[compose.yaml](compose.yaml) 就把 Plex 的数据目录挂在这里。 |
| `EDITION_MODULES` | 要写入的模块，按顺序，`;` 或 `；` 分隔。未设时全部模块按默认顺序写入。 |
| `EDITION_LANG` | `en`、`zh`、`zh-TW`、`ja`、`ko`、`ar`、`fr`、`ru`、`es` 之一，默认 `en`。译名在 `locales/edition.json`。 |
| `EDITION_SKIP_LIBRARIES` | 跳过的媒体库，`;` 或 `；` 分隔。 |
| `EDITION_SEPARATOR` | 模块之间的连接符，默认 ` · `。 |

## 运行

```
editions all --dry-run
editions all
editions reset
editions new
```

`all` 与 `reset` 跑完即退出。`new` 一直运行：连接之前，以及 socket 断线 30 分钟后，先为还没有 Edition 的电影补上，再开始监听；监听期间扫进的电影，在 Plex 上报时写入。读不到 token 或服务器没有响应时，同样每 30 分钟重试一次，不会退出，所以容器可以先启动，等 Plex 完成认领或上线后再接上。

## 许可

[GPL-3.0](LICENSE)
