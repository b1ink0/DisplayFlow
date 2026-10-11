<script setup lang="ts">
import { provide, ref, computed, watch } from 'vue'
import { TabsRoot, TabsContent, TabsList, TabsTrigger } from 'reka-ui'
import {
  ArrowLeftRight,
  Monitor,
  Network,
  Puzzle,
  Activity,
  Settings2,
  Pause,
  Play,
  Radio,
  ArrowUpRight,
  RefreshCw,
} from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import Badge from '@/components/ui/Badge.vue'
import RoutesView from '@/views/RoutesView.vue'
import ComputersView from '@/views/ComputersView.vue'
import ModulesView from '@/views/ModulesView.vue'
import SettingsView from '@/views/SettingsView.vue'
import ActivityView from '@/views/ActivityView.vue'
import ModulePanelView from '@/views/ModulePanelView.vue'
import { useWorkspace } from '@/composables/useWorkspace'
import { workspaceKey } from '@/composables/context'
import { desktop } from '@/services/bridge'
const workspace = useWorkspace()
provide(workspaceKey, workspace)
const { snapshot, settings, busy, error, notice, run, refresh, mutate } =
  workspace
const tab = ref('routes')
const baseNavigation = [
  {
    id: 'routes',
    name: 'Monitor routes',
    icon: Monitor,
    description: 'Assign computers to the inputs of your shared monitors.',
  },
  {
    id: 'computers',
    name: 'Computers',
    icon: Network,
    description: 'Pair computers and check their connection status.',
  },
  {
    id: 'modules',
    name: 'Modules',
    icon: Puzzle,
    description: 'Add monitor providers and tools.',
  },
  {
    id: 'activity',
    name: 'Activity',
    icon: Activity,
    description: 'View input requests, connections, and errors.',
  },
  {
    id: 'settings',
    name: 'Settings',
    icon: Settings2,
    description:
      'Configure Deskflow, input providers, and background operation.',
  },
]
const contributions = computed(() =>
  (settings.value?.modules ?? []).flatMap((module) =>
    (module.manifest.ui?.panels ?? []).map((panel) => ({
      id: `module:${module.manifest.id}:${panel.id}`,
      name: panel.title,
      description: panel.description,
      icon: Puzzle,
      moduleId: module.manifest.id,
      panelId: panel.id,
    })),
  ),
)
const navigation = computed(() => [...baseNavigation, ...contributions.value])
watch(navigation, (items) => {
  if (!items.some((item) => item.id === tab.value)) tab.value = 'modules'
})
</script>
<template>
  <TabsRoot v-model="tab" orientation="vertical" class="flex min-h-screen">
    <aside
      class="fixed inset-y-0 z-10 flex w-60 flex-col border-r border-stone-200 bg-[#fafaf9] px-4 py-7"
    >
      <div class="mb-10 flex items-center gap-3 px-3">
        <div
          class="flex size-9 items-center justify-center rounded-xl bg-violet-600 text-white shadow-sm"
        >
          <ArrowLeftRight :size="20" />
        </div>
        <span class="text-xl font-semibold tracking-tight"
          >DisplayFlow<span class="text-violet-500">.</span></span
        >
      </div>
      <p
        class="mb-3 px-3 text-[10px] font-semibold tracking-[0.15em] text-stone-400 uppercase"
      >
        Workspace
      </p>
      <TabsList class="space-y-1" aria-label="Workspace navigation">
        <TabsTrigger
          v-for="item in navigation"
          :key="item.id"
          :value="item.id"
          :class="[
            'flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition',
            tab === item.id
              ? 'bg-violet-100/65 text-violet-700'
              : 'text-stone-500 hover:bg-stone-100 hover:text-stone-800',
          ]"
        >
          <component :is="item.icon" :size="17" />{{ item.name
          }}<span
            v-if="item.id === 'routes' && settings?.routes.length"
            class="ml-auto text-xs"
            >{{ settings.routes.length }}</span
          >
        </TabsTrigger>
      </TabsList>
      <div class="mt-auto rounded-xl border border-stone-200 bg-white p-4">
        <div class="flex items-center gap-2 text-xs font-semibold">
          <span
            :class="[
              'size-1.5 rounded-full',
              settings?.paused ? 'bg-amber-400' : 'bg-emerald-500',
            ]"
          />{{ settings?.paused ? 'Routing paused' : 'Routing enabled' }}
        </div>
        <p class="mt-2 text-xs leading-relaxed text-stone-500">
          {{ settings?.computerName ?? 'Loading workspace…' }}
        </p>
        <p class="mt-1 text-[10px] text-stone-400">
          Local agent · {{ snapshot?.platform ?? 'starting' }}
        </p>
      </div>
      <div
        class="mt-5 flex items-center justify-between px-3 text-[10px] font-medium text-stone-400"
      >
        <span>DISPLAYFLOW / 0.1.0</span><ArrowUpRight :size="13" />
      </div>
    </aside>
    <main class="ml-60 min-w-0 flex-1">
      <header
        class="flex h-16 items-center justify-between border-b border-stone-200/80 bg-white/60 px-9"
      >
        <div class="flex items-center gap-2 text-xs text-stone-400">
          <span>Workspace</span><span>/</span
          ><span class="text-stone-700">{{
            navigation.find((n) => n.id === tab)?.name
          }}</span>
        </div>
        <div class="flex items-center gap-3">
          <Badge v-if="!desktop" tone="warning">Browser preview</Badge
          ><Badge v-else tone="neutral"><Radio :size="12" /> Desktop app</Badge
          ><Button
            :disabled="busy || !settings"
            @click="
              run(() =>
                mutate((s) => {
                  s.paused = !s.paused
                }),
              )
            "
            ><Pause v-if="!settings?.paused" :size="14" /><Play
              v-else
              :size="14"
            />{{ settings?.paused ? 'Resume' : 'Pause' }}</Button
          >
        </div>
      </header>
      <div class="mx-auto max-w-6xl px-9 py-9">
        <div class="mb-7 flex items-end justify-between">
          <div>
            <p
              class="mb-2 text-[10px] font-semibold tracking-[0.16em] text-violet-600 uppercase"
            >
              {{
                tab === 'routes' ? 'Monitor configuration' : 'Workspace control'
              }}
            </p>
            <h1 class="text-3xl font-semibold tracking-tight text-stone-900">
              {{ navigation.find((n) => n.id === tab)?.name }}
            </h1>
            <p class="mt-2 text-sm text-stone-500">
              {{ navigation.find((n) => n.id === tab)?.description }}
            </p>
          </div>
          <Button
            v-if="tab === 'routes'"
            :disabled="busy"
            @click="run(() => refresh(true), 'Monitor inventory refreshed')"
            ><RefreshCw :size="14" :class="{ 'animate-spin': busy }" />Refresh
            monitors</Button
          >
        </div>
        <div
          v-if="!desktop"
          class="mb-5 rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-xs text-amber-800"
        >
          This browser preview saves UI configuration locally. Monitor
          discovery, commands, pairing, and modules require the desktop app.
        </div>
        <div
          v-if="error"
          role="alert"
          class="mb-5 flex items-start justify-between gap-3 rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700"
        >
          <span>{{ error }}</span
          ><button aria-label="Dismiss error" @click="error = ''">×</button>
        </div>
        <div
          v-if="notice"
          role="status"
          class="mb-5 rounded-lg border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700"
        >
          {{ notice }}
        </div>
        <TabsContent value="routes"><RoutesView /></TabsContent
        ><TabsContent value="computers"><ComputersView /></TabsContent
        ><TabsContent value="modules"><ModulesView /></TabsContent
        ><TabsContent value="activity"><ActivityView /></TabsContent
        ><TabsContent value="settings"><SettingsView /></TabsContent>
        <TabsContent
          v-for="panel in contributions"
          :key="panel.id"
          :value="panel.id"
          ><ModulePanelView
            :module-id="panel.moduleId"
            :panel-id="panel.panelId"
        /></TabsContent>
      </div>
    </main>
  </TabsRoot>
</template>
