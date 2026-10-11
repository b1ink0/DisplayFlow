<script setup lang="ts">
import { ref } from 'vue'
import {
  Computer,
  Copy,
  Link2,
  ShieldCheck,
  Trash2,
  ArrowDownToLine,
  Network,
} from 'lucide-vue-next'
import { useApp } from '@/composables/context'
import { call, desktop } from '@/services/bridge'
import type { Monitor } from '@/types'
import Button from '@/components/ui/Button.vue'
import Badge from '@/components/ui/Badge.vue'
const { settings, snapshot, busy, run } = useApp()
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
  <section class="panel mb-6">
    <div class="flex items-center gap-4 p-5">
      <div
        class="flex size-12 items-center justify-center rounded-xl bg-violet-50 text-violet-600"
      >
        <Computer :size="25" />
      </div>
      <div>
        <h2 class="font-semibold">{{ settings?.computerName }}</h2>
        <p class="hint mt-1">
          This computer · {{ snapshot?.platform }} · port
          {{ settings?.listenPort }}
        </p>
      </div>
      <Badge
        class="ml-auto"
        :tone="snapshot?.agentStatus === 'Listening' ? 'success' : 'neutral'"
        >{{ snapshot?.agentStatus }}</Badge
      >
    </div>
  </section>
  <div class="mb-4 flex items-center justify-between">
    <h2 class="section-title">Paired computers</h2>
    <span class="text-xs text-stone-400"
      >{{ settings?.peers.length ?? 0 }} connected identities</span
    >
  </div>
  <div
    v-if="!settings?.peers.length"
    class="panel mb-6 flex items-center gap-4 p-7"
  >
    <Network :size="30" class="text-stone-300" />
    <div>
      <h3 class="text-sm font-semibold">Bring your other computers along</h3>
      <p class="hint mt-1">
        Install DisplayFlow on each computer, enable its peer agent, and
        exchange an invitation below.
      </p>
    </div>
  </div>
  <div class="mb-6 space-y-3">
    <article v-for="peer in settings?.peers" :key="peer.id" class="panel p-5">
      <div class="flex items-center gap-3">
        <Computer :size="21" class="text-stone-400" />
        <div class="flex-1">
          <h3 class="text-sm font-semibold">{{ peer.name }}</h3>
          <p class="hint mt-1">
            {{ peer.address }}:{{ peer.port }} · identity pinned
          </p>
        </div>
        <Button
          :disabled="busy"
          @click="
            run(async () => {
              inventories[peer.id] = await call<Monitor[]>('peer_inventory', {
                id: peer.id,
              })
            }, 'Peer reached')
          "
          ><ArrowDownToLine :size="14" />Inventory</Button
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
          ><Trash2 :size="14"
        /></Button>
      </div>
      <div
        v-if="inventories[peer.id]"
        class="mt-4 space-y-2 border-t border-stone-100 pt-4"
      >
        <p v-if="!inventories[peer.id]?.length" class="hint">
          No monitors reported.
        </p>
        <div
          v-for="m in inventories[peer.id]"
          :key="m.id"
          class="rounded-lg bg-stone-50 px-3 py-2"
        >
          <p class="text-xs font-medium">{{ m.name }} · {{ m.provider }}</p>
          <p class="mt-1 font-mono text-[10px] break-all text-stone-400">
            {{ m.id }}
          </p>
        </div>
      </div>
    </article>
  </div>
  <div class="panel p-6">
    <div class="mb-5 flex items-center gap-2">
      <Link2 :size="18" class="text-violet-600" />
      <h2 class="section-title">Pair a computer</h2>
    </div>
    <label for="local-address" class="label"
      >This computer’s local IP address</label
    ><input
      id="local-address"
      v-model="address"
      class="field max-w-sm"
      placeholder="192.168.x.x"
    />
    <p class="hint mt-2">
      Use the address on your shared local network. Enable the agent in Settings
      first. Allow DisplayFlow’s TCP port through the firewall on your private
      network only.
    </p>
    <div class="mt-6 grid grid-cols-2 gap-6">
      <section>
        <h3 class="mb-2 text-sm font-medium">1. Create an invitation here</h3>
        <p class="hint mb-3">
          Copy it directly to your other computer. It expires after ten minutes
          and can be used once.
        </p>
        <Button
          :disabled="busy || !desktop || !settings?.networkEnabled || !address"
          @click="run(create)"
          >Create invitation</Button
        ><template v-if="invitation"
          ><label class="label mt-4" for="outgoing-invite"
            >Private invitation</label
          ><textarea
            id="outgoing-invite"
            :value="invitation"
            readonly
            class="field h-28 resize-none font-mono text-[10px]"
          /><Button
            class="mt-2"
            @click="run(copyInvitation, 'Invitation copied')"
            ><Copy :size="13" />Copy</Button
          ></template
        >
      </section>
      <section>
        <h3 class="mb-2 text-sm font-medium">2. Or accept an invitation</h3>
        <p class="hint mb-3">
          Paste an invitation received directly from a computer you trust.
        </p>
        <label for="incoming-invite" class="sr-only">Invitation to accept</label
        ><textarea
          id="incoming-invite"
          v-model="incoming"
          class="field h-28 resize-none font-mono text-xs"
          placeholder="Paste invitation JSON"
        /><Button
          variant="primary"
          class="mt-3"
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
  </div>
  <div class="mt-5 flex items-start gap-3 rounded-lg bg-stone-100/70 p-4">
    <ShieldCheck :size="18" class="shrink-0 text-stone-400" />
    <p class="hint">
      Pairing grants inventory access and input requests allowed by your local
      assignments. It grants no remote shell, file access, or recovery command.
      Keep invitations private; revoke unwanted peers on both computers. Changes
      to an IP address require pairing again.
    </p>
  </div>
</template>
