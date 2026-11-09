<script setup lang="ts">
import UTextarea from '@nuxt/ui/components/Textarea.vue'
import UInput from '@nuxt/ui/components/Input.vue'
import { ref, computed, watch, onMounted } from 'vue'
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
const addressHint = ref('Detecting the local address...')
const detectedAddress = ref('Automatic')
const addresses = ref<string[]>([])
const nearby = ref<
  { id: string; name: string; address: string; port: number }[]
>([])
const scanned = ref(false)
onMounted(async () => {
  if (!desktop) return
  try {
    const detected = await call<string>('local_address')
    detectedAddress.value = detected
    addressHint.value =
      'Detected automatically. Change it if the other computer uses a different network.'
  } catch {
    addressHint.value =
      'Automatic selection will use the route to the invitation sender. You can also enter an address.'
  }
  try {
    addresses.value = await call<string[]>('local_addresses')
  } catch {
    /* Manual entry remains available. */
  }
})
async function scan() {
  nearby.value = await call('discover_computers')
  scanned.value = true
}
async function copyInvitation() {
  await navigator.clipboard.writeText(invitation.value)
}
async function create() {
  invitation.value = await call<string>('create_invitation', {
    address: address.value.trim(),
  })
}
async function pair() {
  await call('pair_peer', {
    invitation: incoming.value.trim(),
    address: address.value.trim(),
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
      description="Create an invitation on one computer and paste it on the other. Both agents must be Listening."
    >
      <p
        v-if="snapshot?.agentStatus !== 'Listening'"
        class="text-sm text-warning"
      >
        This agent is not Listening. Enable it in Settings and resolve any
        reported error before pairing.
      </p>
      <div class="flex max-w-sm flex-col gap-2">
        <label for="local-address" class="label"
          >This computer’s local IP address</label
        ><UInput
          id="local-address"
          v-model="address"
          :placeholder="detectedAddress + ' (automatic)'"
        />
        <p class="hint">{{ addressHint }}</p>
        <div v-if="addresses.length" class="flex flex-wrap gap-2">
          <Button variant="ghost" :disabled="busy" @click="address = ''"
            >Automatic</Button
          >
          <Button
            v-for="ip in addresses"
            :key="ip"
            variant="ghost"
            :disabled="busy"
            @click="address = ip"
            >{{ ip }}</Button
          >
        </div>
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
              busy || !desktop || snapshot?.agentStatus !== 'Listening'
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
              snapshot?.agentStatus !== 'Listening' ||
              !incoming.trim()
            "
            @click="run(pair, 'Computers paired in both directions')"
            ><Link2 :size="14" />Pair computers</Button
          >
        </section>
      </div>
      <div class="flex flex-col gap-3 border-t border-default pt-4">
        <div class="flex flex-wrap items-center gap-3">
          <h3 class="flex-1 text-sm font-medium">Computers on your network</h3>
          <Button :disabled="busy || !desktop" @click="run(scan)"
            >Find computers</Button
          >
        </div>
        <p class="hint">
          Discovery finds addresses, not trusted identities. Exchange an
          invitation to pair a new computer. Paired computers reconnect
          automatically after an IP change when LAN discovery is available.
        </p>
        <p v-if="scanned && !nearby.length" class="hint">
          No computers found. Check Local Network permission and private-network
          firewall access, or exchange an invitation directly.
        </p>
        <div
          v-for="device in nearby"
          :key="device.id + device.address"
          class="flex flex-wrap items-center gap-3 text-sm"
        >
          <span class="min-w-0 flex-1 break-words">{{ device.name }}</span>
          <span class="text-xs text-muted"
            >{{ device.address }}:{{ device.port }}</span
          >
          <Badge>{{
            settings?.peers.some((p) => p.id === device.id)
              ? 'Paired'
              : 'Invitation required'
          }}</Badge>
        </div>
      </div>
      <p class="hint border-t border-default pt-4">
        Pairing allows inventory and assigned input requests. Keep invitations
        private and revoke unwanted peers on both computers. Removing a pairing
        keeps route assignments so they can be reused after pairing again.
      </p>
    </PageSection>
  </div>
</template>
