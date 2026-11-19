<script lang="ts">
  import { onMount } from "svelte";

  import { app } from "../../lib/state/state.svelte";
  import { startCodeTicker } from "../../lib/steam-guard/codes.svelte";
  import AddAccountDialog from "../accounts/AddAccountDialog.svelte";
  import Breadcrumb from "./Breadcrumb.svelte";
  import BusyOverlay from "./BusyOverlay.svelte";
  import BackupDialog from "../vault/BackupDialog.svelte";
  import CardGrid from "../ui/CardGrid.svelte";
  import ConfirmDialog from "../dialogs/ConfirmDialog.svelte";
  import FolderDialog from "../folders/FolderDialog.svelte";
  import FolderEditorDialog from "../folders/FolderEditorDialog.svelte";
  import Footer from "./Footer.svelte";
  import Header from "./Header.svelte";
  import SettingsDialog from "../settings/SettingsDialog.svelte";
  import SteamGuardDialog from "../steam-guard/SteamGuardDialog.svelte";
  import VaultDialog from "../vault/VaultDialog.svelte";
  import VaultPasswordOfferDialog from "../vault/VaultPasswordOfferDialog.svelte";
  import Toasts from "./Toasts.svelte";
  import UpdateDialog from "../updates/UpdateDialog.svelte";

  onMount(() => startCodeTicker());
</script>

<div class="app" class:idle={app.idle}>
  <div class="app-content" inert={app.busy.active}>
    <Header />
    <Breadcrumb />
    <main class="content">
      <CardGrid />
    </main>
    <Footer />

    <SteamGuardDialog />
    <AddAccountDialog />
    <SettingsDialog />
    <UpdateDialog />
    <VaultDialog />
    <VaultPasswordOfferDialog />
    <BackupDialog />
    <ConfirmDialog />
    <FolderDialog />
    <FolderEditorDialog />
  </div>
  <BusyOverlay />
  <Toasts />
</div>
