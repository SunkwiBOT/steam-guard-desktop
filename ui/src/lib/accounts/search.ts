import { SearchScope } from "../state/state.constants";
import type { AccountDto } from "../bindings/AccountDto";

export const matchesSearch = (entry: AccountDto, needle: string, scope: SearchScope): boolean => {
  const names =
    entry.display_name.toLowerCase().includes(needle) ||
    entry.persona_name.toLowerCase().includes(needle);
  if (scope === SearchScope.DISPLAY_NAME) return names;
  const identifiers =
    entry.account_name.toLowerCase().includes(needle) || entry.steam_id.includes(needle);
  return scope === SearchScope.IDENTIFIER ? identifiers : names || identifiers;
};

export type AccountFilterType = {
  folderId: string | null;
  search: string;
  scope: SearchScope;
};

export const filterAccounts = (
  accounts: readonly AccountDto[],
  { folderId, search, scope }: AccountFilterType,
): AccountDto[] => {
  const needle = search.trim().toLowerCase();
  return accounts
    .filter((entry) => {
      if (folderId !== null && entry.folder_id !== folderId) return false;
      if (!needle) return folderId === null ? !entry.folder_id : true;
      return matchesSearch(entry, needle, scope);
    })
    .sort((left, right) => left.display_name.localeCompare(right.display_name));
};
