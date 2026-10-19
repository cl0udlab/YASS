export const THEMES = [
  { value: "light", label: "淺色主題" },
  { value: "dark", label: "深色主題" },
  { value: "blue", label: "藍色主題" },
] as const;

export type Theme = (typeof THEMES)[number]["value"];

export const theme = $state({
  current: (document.documentElement.dataset.theme as Theme) ?? "light",
});

export function setTheme(value: Theme) {
  theme.current = value;
  document.documentElement.dataset.theme = value;
  try {
    localStorage.setItem("yass_theme", value);
  } catch {}
}
