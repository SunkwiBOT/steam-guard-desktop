import { SearchScope } from "../state/state.constants";
import type { AccountType } from "../types";

/**
 * Matches an account against an already lowercased needle. `display_name` only
 * looks at the names shown on the card, `identifier` only at the Steam login and
 * the Steam ID, and `both` at all of them.
 */
export const matchesSearch = (entry: AccountType, needle: string, scope: SearchScope): boolean => {
  const names =
    entry.display_name.toLowerCase().includes(needle) ||
    entry.persona_name.toLowerCase().includes(needle);
  if (scope === SearchScope.DISPLAY_NAME) return names;
  const identifiers =
    entry.account_name.toLowerCase().includes(needle) || entry.steam_id.includes(needle);
  return scope === SearchScope.IDENTIFIER ? identifiers : names || identifiers;
};

export type AccountFilterType = {
  /** Open folder, `null` at the root. */
  folderId: string | null;
  /** Raw search text; trimmed and lowercased here. */
  search: string;
  scope: SearchScope;
};

/**
 * Accounts shown in the grid. A folder always scopes the list, with or
 * without a search term; at the root a search looks through every folder
 * while an empty search only lists the accounts outside folders.
 */
export const filterAccounts = (
  accounts: readonly AccountType[],
  { folderId, search, scope }: AccountFilterType,
): AccountType[] => {
  const needle = search.trim().toLowerCase();
  return accounts
    .filter((entry) => {
      if (folderId !== null && entry.folder_id !== folderId) return false;
      if (!needle) return folderId === null ? !entry.folder_id : true;
      return matchesSearch(entry, needle, scope);
    })
    .sort((left, right) => left.display_name.localeCompare(right.display_name));
};
