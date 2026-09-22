# Ins Downloader

[English](README.md) | [简体中文](README.zh-CN.md)

[![Release](https://github.com/yuxingxin/instagram-media-download/actions/workflows/release.yml/badge.svg)](https://github.com/yuxingxin/instagram-media-download/actions/workflows/release.yml)
[![GitHub release](https://img.shields.io/github/v/release/yuxingxin/instagram-media-download)](https://github.com/yuxingxin/instagram-media-download/releases)

Cross-platform desktop app that saves images and videos from Instagram links (one or many) to local disk. The download engine is [instaloader](https://instaloader.github.io/).

- Installer: download the build for the current OS from [Releases](https://github.com/yuxingxin/instagram-media-download/releases).
- Build from source: see [Build from source](#build-from-source).

## Features

- Paste one or more Instagram links and download them line by line
- Posts, Reels, IGTV, profiles, hashtags, locations, share links, and bare shortcodes
- Carousel posts save every image and video
- Installers bundle instaloader, so Python is not required at runtime
- Windows, macOS, and Linux packages

Downloads include images and videos. Captions and JSON metadata are omitted. The app has no Instagram login and requests public content through instaloader.

## Install

Download the file for the current OS from [Releases](https://github.com/yuxingxin/instagram-media-download/releases):

| OS | Arch | Package |
|----|------|---------|
| macOS | Apple Silicon | `.dmg` |
| Windows | x64 | NSIS `.exe` |
| Linux | x64 | `.deb` |

Packages are currently unsigned and not notarized.

### First launch on macOS

After a browser download, Gatekeeper may show **“Ins Downloader” is damaged and can’t be opened**. That is the unsigned-app block.

1. Open the `.dmg` and copy `Ins Downloader.app` to Applications or the Desktop. Do not launch it from the disk image.
2. In Terminal, remove the download quarantine flag:

```bash
xattr -cr "/Applications/Ins Downloader.app"
```

3. Open the copied app.

On Windows, SmartScreen may warn before the NSIS installer runs. Choose more info, then run anyway.

## Usage

1. Launch Ins Downloader.
2. Paste one or more links into the input box, one per line.
3. Confirm the save folder (default `Pictures/Instagram`). Use Choose folder when another path is needed.
4. Click Start download.
5. Check each link’s result in the Status list.

### Supported links

Matches [instaloader Targets](https://instaloader.github.io/cli-options.html#targets):

| Type | Examples |
|------|----------|
| Post / Reels / IGTV | `https://www.instagram.com/p/<shortcode>/`, `/reel/`, `/reels/`, `/tv/`, and `https://www.instagram.com/<username>/p/<shortcode>/` |
| Share link | `https://www.instagram.com/share/p/<shortcode>/`, `/share/reel/<shortcode>/` |
| Profile | `https://www.instagram.com/<username>/` |
| Hashtag | `https://www.instagram.com/explore/tags/<hashtag>/` or `#hashtag` |
| Location | `https://www.instagram.com/explore/locations/<id>/` or `%location_id` |
| Bare shortcode | `DdivRY4CFX6`, or instaloader syntax `-shortcode` |

## Build from source

### Prerequisites

- A matching dev machine: Windows x64, macOS Apple Silicon, or Linux x64
- [Rust](https://rustup.rs/) (`rustup`)
- [Node.js](https://nodejs.org/)
- Git Bash on Windows to run the vendor script

`scripts/vendor-instaloader.sh` downloads a standalone Python for the current platform and installs instaloader into `src-tauri/resources/`.

### Development

```bash
npm install
npm run tauri dev
```

### Test and package

```bash
cd src-tauri && cargo test
npm run tauri build
```

`npm run tauri build` produces an installer for the current OS.

## Release

1. Set the same `version` in `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`.
2. Add release notes at the top of `CHANGELOG.md` (heading like `## 0.1.0 - YYYY-MM-DD`).
3. Commit, then tag and push to GitHub:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Pushing a `v*` tag to [yuxingxin/instagram-media-download](https://github.com/yuxingxin/instagram-media-download) runs GitHub Actions, builds all three platforms, and creates a GitHub Release:

- macOS Apple Silicon: `.dmg`
- Windows x64: NSIS installer
- Linux x64: `.deb`

Release notes use the matching `CHANGELOG.md` section when present; otherwise they use the commit list from the previous tag to the current tag.

Repository Settings → Actions → General → Workflow permissions must allow Read and write.

## Contributing

Report issues via [GitHub Issues](https://github.com/yuxingxin/instagram-media-download/issues). Send changes via pull request.

## Acknowledgments

- [instaloader](https://instaloader.github.io/): download engine
- [Tauri](https://tauri.app/): desktop shell
- [python-build-standalone](https://github.com/astral-sh/python-build-standalone): standalone Python inside the installer

## Disclaimer

Use of this software must comply with Instagram’s Terms of Use and the copyright of downloaded content. This project is not affiliated with Instagram or Meta.
