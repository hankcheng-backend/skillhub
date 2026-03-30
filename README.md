# SkillHub

[中文版 README](README-zh-TW.md)

A desktop app for managing AI agent skills across multiple agents (Claude, Cursor, Windsurf, Codex, Gemini, etc.).

Browse, sync, install, and upload skills via a local GUI — or let AI agents manage skills directly through the built-in MCP server.

## Features

- **Multi-agent sync** — Install a skill once, symlink it to all your agents
- **Local GUI** — Browse, search, and manage skills with a native desktop app
- **MCP server** — AI agents can search, install, and manage skills via JSON-RPC
- **GitLab integration** — Sync skills from private GitLab repositories
- **Auto-update** — Built-in updater checks GitHub Releases for new versions

## Usage

- Click an **agent icon** on a skill card to share (symlink) the skill to that agent. Click again to unshare.
- The **underlined** agent icon indicates the origin agent where the skill's source files reside.
- **Delete** removes all source files of the skill — use with caution.
- **Remote sources** currently only support GitLab-hosted repositories.

## Download

Download the latest release from [GitHub Releases](https://github.com/hankcheng-backend/skillhub/releases).

| Platform | File |
|----------|------|
| macOS (Apple Silicon) | `SkillHub_x.x.x_aarch64.dmg` |
| macOS (Intel) | `SkillHub_x.x.x_x64.dmg` |
| Windows | `SkillHub_x.x.x_x64-setup.exe` / `.msi` |

### macOS installation note

The app is not code-signed. macOS Gatekeeper will block it with a "damaged" or "unidentified developer" warning.

After dragging SkillHub to Applications, run this in Terminal:

```bash
xattr -cr /Applications/SkillHub.app
```

Then open SkillHub normally.

## MCP Server (stdio)

SkillHub provides a standalone MCP server binary over stdio for AI agents that support external MCP servers.

### Install via npx

```bash
npx @llamohank/skillhub-mcp-stdio
```

### Configure in your AI agent

Add to your agent's MCP config (e.g. `claude_desktop_config.json`):

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

### Supported platforms

| Package | Platform |
|---------|----------|
| `@llamohank/skillhub-mcp-stdio-darwin-arm64` | macOS Apple Silicon |
| `@llamohank/skillhub-mcp-stdio-darwin-x64` | macOS Intel |
| `@llamohank/skillhub-mcp-stdio-win32-x64` | Windows x64 |

## Development

### Prerequisites

- Node.js 20+
- Rust toolchain (stable)
- Tauri CLI v2: `npm install -g @tauri-apps/cli`

### Setup

```bash
npm install
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

## Tech Stack

- **Frontend**: React 19 + TypeScript + Vite
- **Backend**: Rust + Tauri 2 + SQLite (rusqlite)
- **MCP Server**: Axum (HTTP) / stdio (standalone binary)