<script lang="ts">
  import { onMount } from "svelte";

  import { wireMenuDismissal } from "../../lib/state/menus.svelte";
  import { app } from "../../lib/state/state.svelte";
  import { startCodeTicker } from "../../lib/steam-guard/codes.svelte";
  import AddAccountDialog from "../accounts/AddAccountDialog.svelte";
  import Breadcrumb from "./Breadcrumb.svelte";
  import BusyOverlay from "./BusyOverlay.svelte";
  import BackupDialog from "../vault/BackupDialog.svelte";
  import CardGrid from "../ui/CardGrid.svelte";
  import ConfirmDialog from "../dialogs/ConfirmDialog.svelte";
  import ContextMenu from "../ui/ContextMenu.svelte";
  import Dropdown from "../ui/Dropdown.svelte";
  import FolderDialog from "../folders/FolderDialog.svelte";
  import FolderEditorDialog from "../folders/FolderEditorDialog.svelte";
  import Footer from "./Footer.svelte";
  import Header from "./Header.svelte";
  import SettingsDialog from "../settings/SettingsDialog.svelte";
  import SteamGuardDialog from "../steam-guard/SteamGuardDialog.svelte";
  import VaultDialog from "../vault/VaultDialog.svelte";
  import Toast from "./Toast.svelte";
  import UpdateDialog from "../updates/UpdateDialog.svelte";

  onMount(() => {
    wireMenuDismissal();
    return startCodeTicker();
  });
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
    <BackupDialog />
    <ConfirmDialog />
    <FolderDialog />
    <FolderEditorDialog />
    <ContextMenu />
    <Dropdown />
  </div>
  <BusyOverlay />
  <Toast />
</div>
