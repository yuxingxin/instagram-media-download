# Ins Downloader

[English](README.md) | [简体中文](README.zh-CN.md)

[![Release](https://github.com/yuxingxin/instagram-media-download/actions/workflows/release.yml/badge.svg)](https://github.com/yuxingxin/instagram-media-download/actions/workflows/release.yml)
[![GitHub release](https://img.shields.io/github/v/release/yuxingxin/instagram-media-download)](https://github.com/yuxingxin/instagram-media-download/releases)

跨平台桌面客户端，从 Instagram 链接（单条或批量）把图片和视频保存到本地。下载引擎是 [instaloader](https://instaloader.github.io/)。

- 使用安装包：从 [Releases](https://github.com/yuxingxin/instagram-media-download/releases) 下载对应系统的文件。
- 从源码构建：见 [从源码构建](#从源码构建)。

## 功能

- 粘贴一条或多条 Instagram 链接，按行逐条下载
- 支持帖子、Reels、IGTV、主页、话题、地点、分享链接和裸短码
- 轮播帖保存全部图片和视频
- 安装包已内置 instaloader，使用时不必再装 Python
- 提供 Windows、macOS、Linux 安装包

保存图片和视频，不保存 caption 与 JSON 元数据。应用未提供 Instagram 登录入口，通过 instaloader 请求公开内容。

## 安装

从 [Releases](https://github.com/yuxingxin/instagram-media-download/releases) 下载当前系统对应的文件：

| 系统 | 架构 | 安装包 |
|------|------|--------|
| macOS | Apple Silicon | `.dmg` |
| Windows | x64 | NSIS `.exe` |
| Linux | x64 | `.deb` |

安装包目前未做平台签名或公证。

### 首次在 macOS 上打开

浏览器下载后，Gatekeeper 可能提示 **「Ins Downloader」已损坏，无法打开**。这是系统拦截未签名应用时的提示。

1. 打开 `.dmg`，把 `Ins Downloader.app` 拷到「应用程序」或桌面。不要在磁盘映像里直接启动。
2. 在终端清除下载隔离标记：

```bash
xattr -cr "/Applications/Ins Downloader.app"
```

3. 打开拷贝后的应用。

Windows 上运行 NSIS 安装包时，SmartScreen 可能先弹出警告。选择「更多信息」，再选择仍要运行。

## 使用

1. 启动 Ins Downloader。
2. 在输入框粘贴一条或多条链接，每行一条。
3. 确认保存目录（默认 `Pictures/Instagram`），需要时点击「选择文件夹」。
4. 点击「开始下载」。
5. 在「状态」列表查看每条链接的结果。

### 支持的链接

对应 [instaloader Targets](https://instaloader.github.io/cli-options.html#targets)：

| 类型 | 示例 |
|------|------|
| 帖子 / Reels / IGTV | `https://www.instagram.com/p/<shortcode>/`、`/reel/`、`/reels/`、`/tv/`，以及 `https://www.instagram.com/<username>/p/<shortcode>/` |
| 分享链接 | `https://www.instagram.com/share/p/<shortcode>/`、`/share/reel/<shortcode>/` |
| 主页 | `https://www.instagram.com/<username>/` |
| 话题 | `https://www.instagram.com/explore/tags/<hashtag>/` 或 `#hashtag` |
| 地点 | `https://www.instagram.com/explore/locations/<id>/` 或 `%location_id` |
| 裸短码 | `DdivRY4CFX6`，或 instaloader 语法 `-shortcode` |

## 从源码构建

### 前置条件

- 对应系统的开发环境：Windows x64、macOS Apple Silicon 或 Linux x64
- [Rust](https://rustup.rs/)（`rustup`）
- [Node.js](https://nodejs.org/)
- Windows 上运行 vendor 脚本需要 Git Bash

打包时 `scripts/vendor-instaloader.sh` 会下载该平台的独立 Python，并安装 instaloader 到 `src-tauri/resources/`。

### 开发运行

```bash
npm install
npm run tauri dev
```

### 测试与打包

```bash
cd src-tauri && cargo test
npm run tauri build
```

`npm run tauri build` 会按当前系统生成安装包。

## 发布

1. 把 `package.json`、`src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml` 的 `version` 改成同一版本号。
2. 在 `CHANGELOG.md` 顶部加上对应版本说明（标题形如 `## 0.1.0 - YYYY-MM-DD`）。
3. 提交后打 tag 并推送到 GitHub：

```bash
git tag v0.1.0
git push origin v0.1.0
```

推送 `v*` tag 到 [yuxingxin/instagram-media-download](https://github.com/yuxingxin/instagram-media-download) 会触发 GitHub Actions，并行打包三端并创建 GitHub Release：

- macOS Apple Silicon：`.dmg`
- Windows x64：NSIS 安装包
- Linux x64：`.deb`

Release 说明优先使用 `CHANGELOG.md` 中该版本的段落；找不到对应段落时，使用上一个 tag 到当前 tag 的 commit 列表。

仓库 Settings → Actions → General → Workflow permissions 需要允许 Read and write。

## 贡献

通过 [GitHub Issues](https://github.com/yuxingxin/instagram-media-download/issues) 报告问题，通过 Pull Request 提交修改。

## 致谢

- [instaloader](https://instaloader.github.io/)：下载引擎
- [Tauri](https://tauri.app/)：桌面壳
- [python-build-standalone](https://github.com/astral-sh/python-build-standalone)：安装包内的独立 Python

## 声明

使用本软件时须遵守 Instagram 服务条款以及所下载内容的著作权规定。本项目与 Instagram、Meta 无关联。
