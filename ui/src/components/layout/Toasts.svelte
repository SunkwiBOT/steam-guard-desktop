<script lang="ts">
  import { Toaster } from "$lib/components/ui/sonner";
  import { t } from "../../lib/i18n/i18n.svelte";
  import { layout } from "../../lib/state/layout.svelte";

  // ! Same gap used for the toast stack's own vertical spacing, so the offset above the
  // footer and the gap between stacked toasts read as one rhythm.
  const GAP = 8;
</script>

<Toaster
  position="bottom-right"
  visibleToasts={4}
  gap={GAP}
  offset={{ bottom: layout.footerHeight + GAP }}
  mobileOffset={{ bottom: layout.footerHeight + GAP }}
  containerAriaLabel={t.toasts_region()}
/>
