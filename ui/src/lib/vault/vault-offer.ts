import type { SettingsDto } from "../bindings/SettingsDto";
import type { VaultStatusDto } from "../bindings/VaultStatusDto";

export type VaultOfferGateType = {
  check: (
    vault: Pick<VaultStatusDto, "password_suggested">,
    settings: Pick<SettingsDto, "vault_offer_dismissed">,
  ) => boolean;
};

export const createVaultOfferGate = (): VaultOfferGateType => {
  let offered = false;
  return {
    check: (vault, settings) => {
      if (offered || settings.vault_offer_dismissed || !vault.password_suggested) return false;
      offered = true;
      return true;
    },
  };
};
