本仓库是 **Yuli Ledger**：给自己用的 Windows 桌面记账软件。说明与代码标识用英语；软件界面以英语为源文案，简体中文（`zh-Hans`）为翻译。

设计文档：[docs/README.md](docs/README.md)。

# Yuli Ledger

Personal, offline bookkeeping for a single Windows PC. Not a phone app, not a cloud product, not a commercial money suite.

**Status:** Tauri 2 desktop app (pre-v1). Data lives in `%LOCALAPPDATA%\YuliLedger\ledger.sqlite`.

## Develop

Requires Node.js, Rust (MSVC), and Windows WebView2 (bundled with current Windows 10/11).

```
npm install
npm run tauri dev
```

`tauri dev` is only for coding. The product is a desktop window, not a site in the browser.

## Build

Requires the same toolchain as Develop. The first NSIS build downloads NSIS 3.11 into `%LOCALAPPDATA%\tauri\NSIS` (GitHub). After that, bundling works offline.

NSIS installer:

```
npm run tauri -- build
```

Artifact: `src-tauri/target/release/bundle/nsis/Yuli Ledger_0.1.0_x64-setup.exe` (current-user install, Start Menu entry). WebView2 is expected from Windows; the installer does not download it.

Portable folder (database still under LocalAppData; unzip and double-click the exe):

```
npm run tauri -- build
npm run package:portable
```

Copies `src-tauri/target/release/Yuli Ledger.exe` to `release/portable/`.

The running app does not use the network. `tauri build` may, only to fetch bundler tools.

## Tests

```
cd src-tauri
cargo test
```

## Docs

Start at [docs/README.md](docs/README.md).
