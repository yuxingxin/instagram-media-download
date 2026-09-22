# Ins Downloader

桌面客户端：从 Instagram 链接（单条或批量）下载图片和视频。下载引擎是 [instaloader](https://instaloader.github.io/)。支持 Windows、macOS（Apple Silicon）和 Linux。

## 需要

- **使用安装包的人**：Windows x64、macOS Apple Silicon 或 Linux x64。instaloader 已打进应用，不必再装 Python。
- **开发 / 重新打包**：对应系统、Rust（`rustup`）、Node.js。Windows 上运行 vendor 脚本需要 Git Bash。打包时 `scripts/vendor-instaloader.sh` 会下载该平台的独立 Python 并安装 instaloader 到 `src-tauri/resources/`。

## 使用

1. 粘贴一条或多条链接，每行一条。对应 [instaloader Targets](https://instaloader.github.io/cli-options.html#targets)：
   - 帖子 / Reels / IGTV：`/p/`、`/reel/`、`/reels/`、`/tv/`，以及 `username/p|reel|reels|tv/<shortcode>`
   - 主页：`instagram.com/<username>/`
   - 话题：`instagram.com/explore/tags/<hashtag>/` 或 `#hashtag`
   - 地点：`instagram.com/explore/locations/<id>/` 或 `%location_id`
   - 裸短码，或 instaloader 语法 `-shortcode`
2. 选择保存目录（默认 `Pictures/Instagram`）
3. 点「开始下载」

轮播帖会保存全部图片/视频。不下载 caption、json 元数据。

## 开发

```bash
npm install
npm run tauri dev
```

```bash
cd src-tauri && cargo test
npm run tauri build
```

## 发布

1. 把 `package.json`、`src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml` 的 version 改成同一版本号。
2. 在 `CHANGELOG.md` 顶部加上对应版本说明（标题形如 `## 0.1.0 - YYYY-MM-DD`）。
3. 提交后打 tag 并推送到 GitHub：

```bash
git tag v0.1.0
git push origin v0.1.0
```

推送 `v*` tag 到 `github.com:yuxingxin/instagram-media-download` 会触发 GitHub Actions，并行打包三端：

- macOS Apple Silicon：`.dmg`
- Windows x64：NSIS 安装包
- Linux x64：AppImage / deb

Release 会附带 `CHANGELOG.md` 中该版本的说明；找不到对应段落时，用上一个 tag 到当前 tag 的 commit 列表。
