// ! Valid folder icon ids come from Rust (`AppInfoDto.folder_icons`); nothing here mirrors that list.

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

export const isCardColor = (value: string | null | undefined): value is string =>
  typeof value === "string" && value !== "" && CARD_COLORS.includes(value);

export const folderIconId = (id: string | null | undefined, icons: readonly string[]): string =>
  id && icons.includes(id) ? id : DEFAULT_FOLDER_ICON;

export const tintStyle = (color: string | null | undefined): string =>
  isCardColor(color) ? `color:${color};background:${color}33` : "";

export const randomFolderStyle = (
  icons: readonly string[],
  random: () => number = Math.random,
): {
  color: string;
  icon: string;
} => {
  const colors = CARD_COLORS.filter((color) => color);
  const choices = icons.filter((id) => id !== DEFAULT_FOLDER_ICON);
  return {
    color: colors[Math.min(colors.length - 1, Math.floor(random() * colors.length))],
    icon: choices[Math.min(choices.length - 1, Math.floor(random() * choices.length))],
  };
};
