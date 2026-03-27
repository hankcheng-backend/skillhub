const translations = {
  "zh-TW": {
    // Layout
    appName: "SkillHub",
    settings: "設定",

    // Sidebar
    addSource: "新增來源",
    sources: "來源",
    localSkills: "本機技能",

    // Search
    searchSkills: "搜尋技能...",

    // Settings tabs
    general: "一般",
    versions: "版本",
    mcp: "MCP 伺服器",

    // Settings general
    language: "語言",
    autostart: "開機自動啟動",
    autostartDesc: "系統啟動時自動開啟 SkillHub",
    agentManagement: "代理管理",
    enabled: "啟用",
    back: "返回",

    // Versions
    agentVersions: "代理版本",
    checkVersions: "檢查版本",
    checking: "檢查中...",
    notInstalled: "未安裝",
    latest: "最新版",
    upToDate: "已是最新",
    updateAvailable: "有新版本",
    refresh: "重新整理",
    installCommandTitle: "安裝指令",
    updateCommandTitle: "更新指令",
    runInTerminal: "請在終端機執行",
    copyCommand: "複製指令",
    commandCopied: "已複製指令",
    copyFailed: "複製失敗，請手動複製",

    // MCP
    mcpPort: "連接埠",
    mcpPortDesc: "MCP 伺服器監聽的連接埠（預設 9800）",
    mcpStatus: "狀態",
    mcpRunning: "執行中",
    mcpStopped: "已停止",
    mcpRestartRequired: "變更將在重啟後生效",
    mcpPortInvalid: "連接埠必須為 1024–65535 之間的整數",

    // Skill card
    noDescription: "無說明",

    // Skill grid empty/loading
    noResultsFound: "找不到結果",
    noSkillsAvailable: "尚無技能",
    tryDifferentSearch: "試試其他搜尋字詞",
    noSkillsInSource: "此來源沒有任何技能。",
    noSkillsMatchSearch: "沒有符合搜尋的技能",
    noSkillsYet: "尚無技能",
    enableAgentsHint: "請先至設定啟用代理，開始掃描技能目錄。",

    // Agent badges
    origin: "來源",
    syncedClickToUnsync: "已同步 — 點擊取消同步",
    clickToSync: "點擊同步",
    syncFailed: "同步失敗",

    // Skill modal
    agents: "代理：",
    viewFullText: "查看全文",
    collapseFullText: "收起全文",
    loading: "載入中...",
    loadFailed: "無法載入技能內容。",

    // Delete
    delete: "刪除",
    deleteSkill: "刪除技能",
    deleteConfirmMsg: "確定要刪除",
    deleteConfirmSuffix: "嗎？此操作無法復原。",
    deleting: "刪除中...",
    cancel: "取消",

    // Upload
    upload: "上傳",
    uploading: "上傳中...",
    uploadSuccess: "上傳成功",
    uploadFailed: "上傳失敗",
    uploadSkill: "上傳技能",
    selectTargetSource: "選擇目標來源",
    noGitlabSources: "尚未設定 GitLab 來源，請先新增。",
    overwriteRemoteConfirm: "此技能已存在於遠端，是否覆寫？",

    // Remote card
    lastUpdated: "更新於",
    install: "安裝",
    installing: "安裝中...",
    installed: "已安裝",
    reinstall: "重新安裝",
    installFailed: "安裝失敗",
    installSuccess: "安裝成功",
    overwriteConfirm: "覆寫現有的技能？",
    source: "來源：",

    // Add source dialog
    addSourceTitle: "新增來源",
    type: "類型",
    name: "名稱",
    repositoryUrl: "儲存庫網址",
    folderId: "資料夾 ID",
    personalAccessToken: "個人存取權杖",
    fillAllFields: "請填寫所有必填欄位。",
    add: "新增",
    comingSoon: "即將推出",
    addSourceErrNameRequired: "來源名稱不可為空。",
    addSourceErrRepoUrlRequired: "請填寫 GitLab 儲存庫網址。",
    addSourceErrTokenRequired: "請填寫 GitLab 個人存取權杖。",
    addSourceErrInvalidRepoUrl: "GitLab 儲存庫網址格式不正確。",
    addSourceErrUnauthorized: "GitLab 驗證失敗：Token 無效或權限不足。",
    addSourceErrNetwork: "無法連線到 GitLab，請檢查網路或伺服器狀態。",

    // Agent directory
    skillDir: "技能目錄",
    locateDir: "定位資料夾",
    agentNotInstalled: "未安裝 {agent}，請執行以下指令安裝：",
    agentDirMissing: "找不到 {agent} 設定資料夾，請定位該代理的設定資料夾。",
    agentDirNotFound: "找不到設定資料夾",
    defaultPath: "預設路徑",
    customPath: "自訂路徑",

    // Placeholders
    placeholderFolderId: "Google Drive 資料夾 ID",
    placeholderSourceName: "company-skills",
    placeholderRepoUrl: "https://gitlab.com/...",
    placeholderToken: "glpat-...",

    // PAT expiry
    patExpiredBadge: "Token 已過期",
    patExpiredTitle: "更新 GitLab Token",
    patExpiredDesc: "此來源的 Personal Access Token 已過期或無效，請輸入新的 Token。",
    patExpiredSave: "儲存",
    patUpdateSuccess: "Token 已更新",
    patUpdateFailed: "Token 更新失敗",
  },
  en: {
    // Layout
    appName: "SkillHub",
    settings: "Settings",

    // Sidebar
    addSource: "Add Source",
    sources: "Sources",
    localSkills: "Local Skills",

    // Search
    searchSkills: "Search skills...",

    // Settings tabs
    general: "General",
    versions: "Versions",
    mcp: "MCP Server",

    // Settings general
    language: "Language",
    autostart: "Launch at Login",
    autostartDesc: "Automatically open SkillHub when the system starts",
    agentManagement: "Agent Management",
    enabled: "Enabled",
    back: "Back",

    // Versions
    agentVersions: "Agent Versions",
    checkVersions: "Check Versions",
    checking: "Checking...",
    notInstalled: "Not installed",
    latest: "Latest",
    upToDate: "Up to date",
    updateAvailable: "Update available",
    refresh: "Refresh",
    installCommandTitle: "Install Command",
    updateCommandTitle: "Update Command",
    runInTerminal: "Run in terminal",
    copyCommand: "Copy command",
    commandCopied: "Command copied",
    copyFailed: "Copy failed. Please copy manually.",

    // MCP
    mcpPort: "Port",
    mcpPortDesc: "Port the MCP Server listens on (default: 9800)",
    mcpStatus: "Status",
    mcpRunning: "Running",
    mcpStopped: "Stopped",
    mcpRestartRequired: "Changes will take effect after restart",
    mcpPortInvalid: "Port must be an integer between 1024 and 65535",

    // Skill card
    noDescription: "No description",

    // Skill grid empty/loading
    noResultsFound: "No results found",
    noSkillsAvailable: "No skills available",
    tryDifferentSearch: "Try a different search term",
    noSkillsInSource: "This source doesn't have any skills yet.",
    noSkillsMatchSearch: "No skills match your search",
    noSkillsYet: "No skills yet",
    enableAgentsHint: "Enable agents in Settings to start scanning skill directories.",

    // Agent badges
    origin: "Origin",
    syncedClickToUnsync: "Synced — click to unsync",
    clickToSync: "Click to sync",
    syncFailed: "Sync failed",

    // Skill modal
    agents: "Agents:",
    viewFullText: "View Full Text",
    collapseFullText: "Collapse",
    loading: "Loading...",
    loadFailed: "Failed to load skill content.",

    // Delete
    delete: "Delete",
    deleteSkill: "Delete Skill",
    deleteConfirmMsg: "Are you sure you want to delete",
    deleteConfirmSuffix: "? This action cannot be undone.",
    deleting: "Deleting...",
    cancel: "Cancel",

    // Upload
    upload: "Upload",
    uploading: "Uploading...",
    uploadSuccess: "Upload successful",
    uploadFailed: "Upload failed",
    uploadSkill: "Upload Skill",
    selectTargetSource: "Select target source",
    noGitlabSources: "No GitLab sources configured. Please add one first.",
    overwriteRemoteConfirm: "This skill already exists on the remote. Overwrite?",

    // Remote card
    lastUpdated: "Updated",
    install: "Install",
    installing: "Installing...",
    installed: "Installed",
    reinstall: "Reinstall",
    installFailed: "Install failed",
    installSuccess: "Install successful",
    overwriteConfirm: "Overwrite existing skill?",
    source: "Source:",

    // Add source dialog
    addSourceTitle: "Add Source",
    type: "Type",
    name: "Name",
    repositoryUrl: "Repository URL",
    folderId: "Folder ID",
    personalAccessToken: "Personal Access Token",
    fillAllFields: "Please fill in all required fields.",
    add: "Add",
    comingSoon: "Coming soon",
    addSourceErrNameRequired: "Source name cannot be empty.",
    addSourceErrRepoUrlRequired: "GitLab repository URL is required.",
    addSourceErrTokenRequired: "GitLab Personal Access Token is required.",
    addSourceErrInvalidRepoUrl: "Invalid GitLab repository URL.",
    addSourceErrUnauthorized: "GitLab authentication failed: invalid token or insufficient permissions.",
    addSourceErrNetwork: "Cannot connect to GitLab. Check your network or server status.",

    // Agent directory
    skillDir: "Skill Directory",
    locateDir: "Locate Folder",
    agentNotInstalled: "{agent} is not installed. Run the following command:",
    agentDirMissing: "Cannot find {agent} config directory. Please locate the directory.",
    agentDirNotFound: "Config directory not found",
    defaultPath: "Default path",
    customPath: "Custom path",

    // Placeholders
    placeholderFolderId: "Google Drive folder ID",
    placeholderSourceName: "company-skills",
    placeholderRepoUrl: "https://gitlab.com/...",
    placeholderToken: "glpat-...",

    // PAT expiry
    patExpiredBadge: "Token expired",
    patExpiredTitle: "Update GitLab Token",
    patExpiredDesc: "The Personal Access Token for this source has expired or is invalid. Enter a new token.",
    patExpiredSave: "Save",
    patUpdateSuccess: "Token updated",
    patUpdateFailed: "Failed to update token",
  },
} as const;

type Lang = keyof typeof translations;
export type Key = keyof typeof translations["zh-TW"];

function getLang(): Lang {
  const stored = localStorage.getItem("lang");
  return stored === "en" ? "en" : "zh-TW";
}

export function t(key: Key): string {
  return translations[getLang()][key];
}

export function getCurrentLang(): Lang {
  return getLang();
}

export function setLang(lang: Lang): void {
  localStorage.setItem("lang", lang);
  window.location.reload();
}
