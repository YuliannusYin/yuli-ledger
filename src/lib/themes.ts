export const UI_THEMES = ["metal", "claude", "vscode", "github", "tiktok"] as const;

export type UiThemeId = (typeof UI_THEMES)[number];

export function resolvedSkin(theme: string | null | undefined): UiThemeId {
  return UI_THEMES.includes(theme as UiThemeId) ? (theme as UiThemeId) : "metal";
}

export function resolvedTheme(scheme: string | null | undefined): "light" | "dark" {
  if (scheme === "light" || scheme === "dark") return scheme;
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

export function applyAppearance(scheme: string | null | undefined, skin: string | null | undefined) {
  const root = document.documentElement;
  root.dataset.theme = resolvedTheme(scheme);
  root.dataset.skin = resolvedSkin(skin);
}

export const THEME_PREVIEWS: Record<UiThemeId, { light: [string, string, string]; dark: [string, string, string] }> = {
  metal: { light: ["#f4f4f5", "#d4d4d8", "#c2410c"], dark: ["#18181b", "#3f3f46", "#fb923c"] },
  claude: { light: ["#faf9f5", "#e3e0d6", "#d97757"], dark: ["#1f1e1d", "#3d3b36", "#de7356"] },
  vscode: { light: ["#ffffff", "#e5e5e5", "#007acc"], dark: ["#1e1e1e", "#3e3e42", "#007acc"] },
  github: { light: ["#ffffff", "#d0d7de", "#0969da"], dark: ["#0d1117", "#30363d", "#4493f8"] },
  tiktok: { light: ["#f8f8f8", "#e3e3e3", "#fe2c55"], dark: ["#000000", "#2f2f2f", "#fe2c55"] },
};
