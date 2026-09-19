<script setup lang="ts">
// On a phone VitePress shows two menus: the hamburger for the site nav and
// a "Menu" bar for the page sidebar. This puts the sidebar inside the
// hamburger's screen so there is one menu, and the "Menu" bar is hidden.
import { useSidebar } from "vitepress/theme";
import VPSidebarItem from "vitepress/dist/client/theme-default/components/VPSidebarItem.vue";

const { sidebarGroups, hasSidebar } = useSidebar();
</script>

<template>
  <div v-if="hasSidebar" class="mobile-sidebar">
    <VPSidebarItem v-for="item in sidebarGroups" :key="item.text" :item="item" :depth="0" />
  </div>
</template>

<style scoped>
.mobile-sidebar {
  margin-top: 8px;
  padding-top: 16px;
  border-top: 1px solid var(--vp-c-divider);
}
</style>
