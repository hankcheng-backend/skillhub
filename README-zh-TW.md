# SkillHub

[English README](README.md)

一款桌面應用程式，用於跨多個 AI 代理（Claude、Cursor、Windsurf、Codex、Gemini 等）管理技能。

透過本地 GUI 瀏覽、同步、安裝和上傳技能——或讓 AI 代理透過內建的 MCP 伺服器直接管理技能。

## 功能

- **多代理同步** — 安裝一次技能，透過符號連結同步到所有代理
- **本地 GUI** — 使用原生桌面應用程式瀏覽、搜尋和管理技能
- **MCP 伺服器** — AI 代理可透過 JSON-RPC 搜尋、安裝和管理技能
- **GitLab 整合** — 從私有 GitLab 儲存庫同步技能
- **自動更新** — 內建更新器會檢查 GitHub Releases 的新版本

## 使用說明

- 點擊技能卡片上的**代理圖示**，可透過符號連結（symlink）將技能共享給該代理，再次點擊則取消共享。
- 帶有**底線**的代理圖示代表該技能的源文件所在的原始代理。
- **刪除**操作會刪除該技能的所有源文件，請謹慎使用。
- **遠端倉庫**目前僅支援 GitLab 託管模式。

## 下載

從 [GitHub Releases](https://github.com/hankcheng-backend/skillhub/releases) 下載最新版本。

| 平台 | 檔案 |
|------|------|
| macOS (Apple Silicon) | `SkillHub_x.x.x_aarch64.dmg` |
| macOS (Intel) | `SkillHub_x.x.x_x64.dmg` |
| Windows | `SkillHub_x.x.x_x64-setup.exe` / `.msi` |

### macOS 安裝注意事項

此應用程式未經程式碼簽署。macOS Gatekeeper 會顯示「已損毀」或「未識別的開發者」警告而阻擋開啟。

將 SkillHub 拖入「應用程式」資料夾後，在終端機執行：

```bash
xattr -cr /Applications/SkillHub.app
```

之後即可正常開啟 SkillHub。

## MCP 伺服器（stdio）

SkillHub 提供一個獨立的 MCP 伺服器執行檔，透過 stdio 與支援外部 MCP 伺服器的 AI 代理通訊。

### 透過 npx 安裝

```bash
npx @llamohank/skillhub-mcp-stdio
```

### 在 AI 代理中設定

將以下內容加入代理的 MCP 設定檔（例如 `claude_desktop_config.json`）：

```json
{
  "mcpServers": {
    "skillhub": {
      "command": "npx",
      "args": ["-y", "@llamohank/skillhub-mcp-stdio"]
    }
  }
}
```

### 支援平台

| 套件 | 平台 |
|------|------|
| `@llamohank/skillhub-mcp-stdio-darwin-arm64` | macOS Apple Silicon |
| `@llamohank/skillhub-mcp-stdio-darwin-x64` | macOS Intel |
| `@llamohank/skillhub-mcp-stdio-win32-x64` | Windows x64 |

## 開發

### 前置需求

- Node.js 20+
- Rust 工具鏈（stable）
- Tauri CLI v2：`npm install -g @tauri-apps/cli`

### 設定

```bash
npm install
npm run tauri dev
```

### 建置

```bash
npm run tauri build
```

## 技術棧

- **前端**：React 19 + TypeScript + Vite
- **後端**：Rust + Tauri 2 + SQLite（rusqlite）
- **MCP 伺服器**：Axum（HTTP）/ stdio（獨立執行檔）