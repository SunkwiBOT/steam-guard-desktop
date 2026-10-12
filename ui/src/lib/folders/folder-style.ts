//! No Svelte or i18n import here: these helpers run in plain node tests.

export const CARD_COLORS: readonly string[] = [
  "",
  "#3b82f6",
  "#06b6d4",
  "#10b981",
  "#84cc16",
  "#f59e0b",
  "#f97316",
  "#f43f5e",
  "#ec4899",
  "#8b5cf6",
  "#71717a",
];

export const DEFAULT_FOLDER_ICON = "folder";

/** ! Same list and order as `FOLDER_ICONS` in src-tauri/src/settings/model.rs. */
export const FOLDER_ICON_IDS: readonly string[] = [
  "folder",
  "folder-open",
  "star",
  "heart",
  "flame",
  "zap",
  "trophy",
  "crown",
  "target",
  "crosshair",
  "gamepad-2",
  "swords",
  "shield",
  "rocket",
  "ghost",
  "gem",
  "book",
  "music",
  "palette",
  "globe",
];

export const isCardColor = (value: string | null | undefined): value is string =>
  typeof value === "string" && value !== "" && CARD_COLORS.includes(value);

/** A stored icon id, or the plain folder for an unknown or missing one. */
export const folderIconId = (id: string | null | undefined): string =>
  id && FOLDER_ICON_IDS.includes(id) ? id : DEFAULT_FOLDER_ICON;

/** Inline style of a tinted badge: the colour on a 20 % tint of itself. */
export const tintStyle = (color: string | null | undefined): string =>
  isCardColor(color) ? `color:${color};background:${color}33` : "";

/** New folders get a random colour and a decorative icon, never the plain
 *  folder or no colour. */
export const randomFolderStyle = (
  random: () => number = Math.random,
): {
  color: string;
  icon: string;
} => {
  const colors = CARD_COLORS.filter((color) => color);
  const icons = FOLDER_ICON_IDS.filter((id) => id !== DEFAULT_FOLDER_ICON);
  return {
    color: colors[Math.min(colors.length - 1, Math.floor(random() * colors.length))],
    icon: icons[Math.min(icons.length - 1, Math.floor(random() * icons.length))],
  };
};
