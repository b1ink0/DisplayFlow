<script setup lang="ts">
import UTextarea from '@nuxt/ui/components/Textarea.vue'
import UInput from '@nuxt/ui/components/Input.vue'
import { ref, computed, watch } from 'vue'
import { Computer, Copy, Link2, Trash2, ArrowDownToLine } from 'lucide-vue-next'
import { useApp } from '@/composables/context'
import { call, desktop } from '@/services/bridge'
import type { Monitor } from '@/types'
import Button from '@/components/ui/Button.vue'
import Badge from '@/components/ui/Badge.vue'
import PageSection from '@/components/product/PageSection.vue'
import ListToolbar from '@/components/product/ListToolbar.vue'
import EmptyState from '@/components/product/EmptyState.vue'
import Disclosure from '@/components/product/Disclosure.vue'
import UPagination from '@nuxt/ui/components/Pagination.vue'
const { settings, snapshot, busy, run } = useApp()
const page = ref(1)
const peers = computed(() =>
  (settings.value?.peers ?? []).slice((page.value - 1) * 10, page.value * 10),
)
watch(
  () => settings.value?.peers.length ?? 0,
  (count) => {
    page.value = Math.max(1, Math.min(page.value, Math.ceil(count / 10)))
  },
)
const inventoryLimits = ref<Record<string, number>>({})
const address = ref('')
const invitation = ref('')
const incoming = ref('')
const inventories = ref<Record<string, Monitor[]>>({})
async function copyInvitation() {
  await navigator.clipboard.writeText(invitation.value)
}
async function create() {
  invitation.value = await call<string>('create_invitation', {
    address: address.value,
  })
}
async function pair() {
  await call('pair_peer', {
    invitation: incoming.value,
    address: address.value,
  })
  incoming.value = ''
}
</script>
<template>
  <div class="flex min-w-0 flex-col gap-6">
    <div class="flex flex-wrap items-center gap-3 border-y border-default py-3">
      <Computer :size="18" class="text-muted" /><span
        class="min-w-0 flex-1 text-sm font-medium break-words"
        >{{ settings?.computerName }}</span
      ><span class="text-xs text-muted"
        >{{ snapshot?.platform }} · port {{ settings?.listenPort }}</span
      ><Badge
        :tone="snapshot?.agentStatus === 'Listening' ? 'success' : 'neutral'"
        >{{ snapshot?.agentStatus }}</Badge
      >
    </div>
    <PageSection>
      <ListToolbar
        title="Paired computers"
        :count="settings?.peers.length ?? 0"
      />
      <EmptyState
        v-if="!settings?.peers.length"
        title="Connect another computer"
        description="Install DisplayFlow on the other computer and exchange an invitation below."
        icon="i-lucide-network"
      />
      <div class="flex flex-col divide-y divide-default">
        <article v-for="peer in peers" :key="peer.id" class="py-3">
          <Disclosure
            :title="peer.name"
            :description="
              peer.address + ':' + peer.port + ' · trusted identity'
            "
          >
            <template #actions
              ><Button
                :disabled="busy"
                @click="
                  run(async () => {
                    inventories[peer.id] = await call<Monitor[]>(
                      'peer_inventory',
                      { id: peer.id },
                    )
                  }, 'Peer reached')
                "
                ><ArrowDownToLine :size="14" /><span class="hidden sm:inline"
                  >Inventory</span
                ></Button
              ><Button
                variant="ghost"
                :disabled="busy"
                aria-label="Revoke peer"
                @click="
                  run(
                    () => call('remove_peer', { id: peer.id }),
                    'Local peer access revoked',
                  )
                "
                ><Trash2 :size="14" /></Button
            ></template>
            <p v-if="!inventories[peer.id]" class="hint">
              Fetch inventory to inspect this computer’s displays.
            </p>
            <p v-else-if="!inventories[peer.id]?.length" class="hint">
              No monitors reported.
            </p>
            <div
              v-for="m in inventories[peer.id]?.slice(
                0,
                inventoryLimits[peer.id] ?? 50,
              )"
              :key="m.id"
              class="flex min-w-0 flex-col gap-1 border-s border-default ps-4"
            >
              <p class="text-sm font-medium break-words">
                {{ m.name }}
                <span class="font-normal text-muted">· {{ m.provider }}</span>
              </p>
              <p class="font-mono text-xs break-all text-muted">{{ m.id }}</p>
            </div>
            <Button
              v-if="
                (inventories[peer.id]?.length ?? 0) >
                (inventoryLimits[peer.id] ?? 50)
              "
              variant="ghost"
              class="self-start"
              @click="
                inventoryLimits[peer.id] = (inventoryLimits[peer.id] ?? 50) + 50
              "
              >Show more displays</Button
            >
          </Disclosure>
        </article>
      </div>
      <UPagination
        v-if="(settings?.peers.length ?? 0) > 10"
        v-model:page="page"
        :total="settings?.peers.length ?? 0"
        :items-per-page="10"
        class="self-end"
      />
    </PageSection>
    <PageSection
      title="Pair a computer"
      description="Use this computer’s address on your shared local network. Enable its agent in Settings first."
    >
      <div class="flex max-w-sm flex-col gap-2">
        <label for="local-address" class="label"
          >This computer’s local IP address</label
        ><UInput
          id="local-address"
          v-model="address"
          placeholder="192.168.x.x"
        />
      </div>
      <div class="flex flex-col gap-6 md:flex-row">
        <section class="flex min-w-0 flex-1 flex-col gap-3">
          <h3 class="text-sm font-medium">Create an invitation</h3>
          <p class="hint">
            Send it directly to the other computer. Invitations expire after ten
            minutes and work once.
          </p>
          <Button
            class="self-start"
            :disabled="
              busy || !desktop || !settings?.networkEnabled || !address
            "
            @click="run(create)"
            >Create invitation</Button
          ><template v-if="invitation"
            ><label class="label" for="outgoing-invite"
              >Private invitation</label
            ><UTextarea
              id="outgoing-invite"
              :model-value="invitation"
              readonly
              :rows="4"
              :ui="{ base: 'font-mono text-xs' }"
            /><Button
              class="self-start"
              @click="run(copyInvitation, 'Invitation copied')"
              ><Copy :size="14" />Copy invitation</Button
            ></template
          >
        </section>
        <section class="flex min-w-0 flex-1 flex-col gap-3">
          <h3 class="text-sm font-medium">Accept an invitation</h3>
          <p class="hint">
            Paste an invitation received directly from a computer you trust.
          </p>
          <label for="incoming-invite" class="sr-only"
            >Invitation to accept</label
          ><UTextarea
            id="incoming-invite"
            v-model="incoming"
            :rows="4"
            :ui="{ base: 'font-mono text-xs' }"
            placeholder="Paste invitation JSON"
          /><Button
            variant="primary"
            class="self-start"
            :disabled="
              busy ||
              !desktop ||
              !settings?.networkEnabled ||
              !incoming ||
              !address
            "
            @click="run(pair, 'Computers paired in both directions')"
            ><Link2 :size="14" />Pair computers</Button
          >
        </section>
      </div>
      <p class="hint border-t border-default pt-4">
        Pairing allows inventory and assigned input requests. Keep invitations
        private and revoke unwanted peers on both computers. Pair again after an
        IP address changes.
      </p>
    </PageSection>
  </div>
</template>
