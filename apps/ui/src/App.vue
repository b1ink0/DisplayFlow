<script setup lang="ts">
import { provide, ref, computed, watch, defineAsyncComponent } from 'vue'
import type { AsyncComponentLoader } from 'vue'
import UApp from '@nuxt/ui/components/App.vue'
import UAlert from '@nuxt/ui/components/Alert.vue'
import USlideover from '@nuxt/ui/components/Slideover.vue'
import UNavigationMenu from '@nuxt/ui/components/NavigationMenu.vue'
import USkeleton from '@nuxt/ui/components/Skeleton.vue'
import {
  ArrowLeftRight,
  Pause,
  Play,
  Menu,
  Sun,
  Moon,
  RefreshCw,
} from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import Badge from '@/components/ui/Badge.vue'
import PageHeader from '@/components/product/PageHeader.vue'
import ViewLoading from '@/components/product/ViewLoading.vue'
import { useWorkspace } from '@/composables/useWorkspace'
import { useAppearance } from '@/composables/useAppearance'
import { workspaceKey } from '@/composables/context'
import { desktop } from '@/services/bridge'
const workspace = useWorkspace()
provide(workspaceKey, workspace)
const { snapshot, settings, busy, error, notice, run, refresh, mutate } =
  workspace
const { mode, toggle } = useAppearance()
const tab = ref('routes')
const drawer = ref(false)
const navigationLimit = ref(20)
const baseNavigation = [
  {
    id: 'routes',
    name: 'Monitor routes',
    icon: 'i-lucide-monitor',
    description: 'Connect each shared display to the right computer.',
  },
  {
    id: 'computers',
    name: 'Computers',
    icon: 'i-lucide-network',
    description: 'Pair your computers and inspect their available displays.',
  },
  {
    id: 'modules',
    name: 'Modules',
    icon: 'i-lucide-puzzle',
    description: 'Manage input providers and monitor tools.',
  },
  {
    id: 'activity',
    name: 'Activity',
    icon: 'i-lucide-activity',
    description: 'Recent input requests and agent events.',
  },
  {
    id: 'settings',
    name: 'Settings',
    icon: 'i-lucide-settings-2',
    description: 'Configure Deskflow, local access and background operation.',
  },
]
const contributions = computed(() =>
  (settings.value?.modules ?? []).flatMap((module) =>
    (module.manifest.ui?.panels ?? []).map((panel) => ({
      id: `module:${module.manifest.id}:${panel.id}`,
      name: panel.title,
      description: panel.description,
      icon: 'i-lucide-puzzle',
      moduleId: module.manifest.id,
      panelId: panel.id,
    })),
  ),
)
const navigation = computed(() => [...baseNavigation, ...contributions.value])
const current = computed(() => navigation.value.find((n) => n.id === tab.value))
const items = computed(() =>
  navigation.value.slice(0, navigationLimit.value).map((n) => ({
    label: n.name,
    icon: n.icon,
    active: n.id === tab.value,
    badge:
      n.id === 'routes'
        ? settings.value?.routes.length || undefined
        : undefined,
    onSelect: () => {
      tab.value = n.id
      drawer.value = false
    },
  })),
)
const loadView = (loader: AsyncComponentLoader) =>
  defineAsyncComponent({ loader, loadingComponent: ViewLoading, delay: 120 })
const views = {
  routes: loadView(() => import('@/views/RoutesView.vue')),
  computers: loadView(() => import('@/views/ComputersView.vue')),
  modules: loadView(() => import('@/views/ModulesView.vue')),
  activity: loadView(() => import('@/views/ActivityView.vue')),
  settings: loadView(() => import('@/views/SettingsView.vue')),
}
const ModulePanelView = loadView(() => import('@/views/ModulePanelView.vue'))
const currentView = computed(() => views[tab.value as keyof typeof views])
const panel = computed(() =>
  contributions.value.find((p) => p.id === tab.value),
)
watch(navigation, (items) => {
  if (!items.some((item) => item.id === tab.value)) tab.value = 'modules'
})
</script>
<template>
  <UApp :tooltip="{ delayDuration: 350 }">
    <div class="flex h-dvh min-h-0 overflow-hidden bg-app">
      <aside
        class="hidden w-56 shrink-0 flex-col border-e border-default bg-navigation lg:flex"
        aria-label="Workspace"
      >
        <div class="flex h-14 shrink-0 items-center gap-3 px-4">
          <ArrowLeftRight :size="18" /><span class="text-base font-semibold"
            >DisplayFlow</span
          >
        </div>
        <nav class="min-h-0 flex-1 overflow-y-auto px-3 py-4">
          <UNavigationMenu
            orientation="vertical"
            :items="items"
            :ui="{
              link: 'min-h-9 text-[13px]',
              linkLeadingIcon: 'size-4',
              linkLabel: 'truncate',
            }"
          /><Button
            v-if="navigation.length > navigationLimit"
            variant="ghost"
            class="w-full"
            @click="navigationLimit += 20"
            >More tools</Button
          >
        </nav>
        <footer class="flex flex-col gap-2 border-t border-default p-4">
          <div class="flex items-center gap-2 text-xs">
            <span
              :class="[
                'size-1.5 rounded-full',
                settings?.paused ? 'bg-warning' : 'bg-success',
              ]"
            />{{ settings?.paused ? 'Routing paused' : 'Routing enabled' }}
          </div>
          <p
            class="truncate text-xs text-muted"
            :title="settings?.computerName"
          >
            {{ settings?.computerName ?? 'Starting…' }}
          </p>
          <div class="flex items-center justify-between text-xs text-muted">
            <span>v0.1.0</span><span>{{ snapshot?.platform }}</span>
          </div>
        </footer>
      </aside>
      <div class="flex min-w-0 flex-1 flex-col">
        <header
          class="flex h-14 shrink-0 items-center justify-between gap-3 border-b border-default px-4 sm:px-6"
        >
          <div class="flex min-w-0 items-center gap-2">
            <Button
              variant="ghost"
              class="lg:hidden"
              aria-label="Open navigation"
              @click="drawer = true"
              ><Menu :size="18" /></Button
            ><span class="truncate text-sm text-toned">{{
              current?.name
            }}</span>
          </div>
          <div class="flex shrink-0 items-center gap-2">
            <Badge v-if="!desktop" tone="warning">Preview</Badge
            ><Button
              variant="ghost"
              :aria-label="
                mode === 'dark' ? 'Use light appearance' : 'Use dark appearance'
              "
              @click="toggle"
              ><Sun v-if="mode === 'dark'" :size="16" /><Moon
                v-else
                :size="16" /></Button
            ><Button
              :disabled="busy || !settings"
              @click="
                run(() =>
                  mutate((s) => {
                    s.paused = !s.paused
                  }),
                )
              "
              ><Play v-if="settings?.paused" :size="14" /><Pause
                v-else
                :size="14"
              />{{ settings?.paused ? 'Resume' : 'Pause' }}</Button
            >
          </div>
        </header>
        <main
          id="main-content"
          class="min-h-0 flex-1 overflow-y-auto"
          :aria-busy="busy"
        >
          <div
            class="mx-auto flex w-full max-w-6xl flex-col gap-6 p-4 sm:p-6 lg:p-8"
          >
            <PageHeader
              :title="current?.name ?? 'DisplayFlow'"
              :description="current?.description"
              ><template v-if="tab === 'routes'" #actions
                ><Button
                  :disabled="busy"
                  @click="
                    run(() => refresh(true), 'Monitor inventory refreshed')
                  "
                  ><RefreshCw
                    :size="14"
                    :class="{ 'animate-spin': busy }"
                  />Refresh monitors</Button
                ></template
              ></PageHeader
            >
            <UAlert
              v-if="!desktop"
              color="warning"
              variant="subtle"
              title="Browser preview"
              description="Configuration is saved locally. Hardware control, pairing and modules require the desktop app."
              icon="i-lucide-info"
            />
            <UAlert
              v-if="error"
              role="alert"
              color="error"
              variant="subtle"
              :title="error"
              :close="{ 'aria-label': 'Dismiss error' }"
              @update:open="error = ''"
            />
            <UAlert
              v-if="notice"
              role="status"
              color="success"
              variant="subtle"
              :title="notice"
              :close="{ 'aria-label': 'Dismiss notice' }"
              @update:open="notice = ''"
            />
            <div v-if="!snapshot" class="flex flex-col gap-4">
              <USkeleton class="h-12" /><USkeleton class="h-48" />
            </div>
            <div v-else class="min-w-0">
              <KeepAlive :max="5"
                ><component :is="currentView" v-if="currentView" :key="tab"
              /></KeepAlive>
              <KeepAlive :max="8"
                ><ModulePanelView
                  v-if="panel"
                  :key="panel.id"
                  :module-id="panel.moduleId"
                  :panel-id="panel.panelId"
              /></KeepAlive>
            </div>
          </div>
        </main>
      </div>
    </div>
    <USlideover
      v-model:open="drawer"
      title="DisplayFlow"
      description="Workspace navigation"
      side="left"
      ><template #body
        ><UNavigationMenu
          orientation="vertical"
          :items="items"
          :ui="{ link: 'min-h-11 text-[13px]', linkLeadingIcon: 'size-4' }"
        /><Button
          v-if="navigation.length > navigationLimit"
          variant="ghost"
          @click="navigationLimit += 20"
          >More tools</Button
        ></template
      ></USlideover
    >
  </UApp>
</template>
