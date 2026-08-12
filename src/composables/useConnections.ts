import { computed, onUnmounted, ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { ConnectionConfig, ConnectionRuntimeSnapshot } from '@/types/settings'
import type { ConnectionStatus } from '@/types/types'

interface ConnectionView extends ConnectionConfig {
  runtime: ConnectionRuntimeSnapshot
}

export function normalizeConnectionError(error: unknown): string {
  if (error instanceof Error) return error.message
  if (typeof error === 'string') return error
  return 'Неизвестная ошибка подключения'
}

const MUTATION_TIMEOUT_MS = 10_000

async function withTimeout<T>(promise: Promise<T>, label: string): Promise<T> {
  let timeoutId: number | undefined
  const timeout = new Promise<never>((_, reject) => {
    timeoutId = window.setTimeout(() => reject(new Error(`${label}: превышено время ожидания`)), MUTATION_TIMEOUT_MS)
  })
  try {
    return await Promise.race([promise, timeout])
  } finally {
    if (timeoutId !== undefined) window.clearTimeout(timeoutId)
  }
}

export function useConnections(): {
  configs: Ref<ConnectionConfig[]>
  runtimeStates: Ref<Map<string, ConnectionRuntimeSnapshot>>
  connections: Ref<ConnectionView[]>
  loading: Ref<boolean>
  error: Ref<string | null>
  reload: () => Promise<void>
  add: (config: ConnectionConfig) => Promise<void>
  update: (id: string, config: ConnectionConfig) => Promise<void>
  remove: (id: string) => Promise<void>
  connect: (id: string) => Promise<void>
  disconnect: (id: string) => Promise<void>
} {
  const configs = ref<ConnectionConfig[]>([])
  const runtimeStates = ref(new Map<string, ConnectionRuntimeSnapshot>())
  const loading = ref(false)
  const error = ref<string | null>(null)
  let unlistenFns: UnlistenFn[] = []
  let subscribed = false
  let disposed = false

  const connections = computed(() => configs.value.map((config) => ({
    ...config,
    runtime: runtimeStates.value.get(config.id) ?? {
      id: config.id,
      status: 'Disconnected' as ConnectionStatus,
      isTyping: false,
    },
  })))

  function applySnapshot(snapshot: ConnectionRuntimeSnapshot[]) {
    runtimeStates.value = new Map(snapshot.map((state) => [state.id, state]))
  }

  async function reload() {
    loading.value = true
    error.value = null
    try {
      const [nextConfigs, snapshot] = await Promise.all([
        invoke<ConnectionConfig[]>('get_connections'),
        invoke<ConnectionRuntimeSnapshot[]>('get_connection_runtime_snapshot'),
      ])
      configs.value = nextConfigs
      applySnapshot(snapshot)
    } catch (reason) {
      error.value = normalizeConnectionError(reason)
      throw reason
    } finally {
      loading.value = false
    }
  }

  async function subscribe() {
    if (subscribed) return
    subscribed = true
    const listeners = await Promise.all([
      listen<[string, string]>('connection-status-changed', ({ payload }) => {
        const [id, rawStatus] = payload
        const status = rawStatus.startsWith('Error:') ? 'Error' : rawStatus as ConnectionStatus
        const previous = runtimeStates.value.get(id)
        runtimeStates.value.set(id, { ...previous, id, status } as ConnectionRuntimeSnapshot)
      }),
      listen<[string, string]>('message-received', ({ payload }) => {
        const [id, lastMessage] = payload
        const previous = runtimeStates.value.get(id)
        runtimeStates.value.set(id, { ...previous, id, lastMessage, isTyping: false, previewText: undefined, status: previous?.status ?? 'Disconnected' })
      }),
      listen<string>('message-cleared', ({ payload: id }) => {
        const previous = runtimeStates.value.get(id)
        if (previous) runtimeStates.value.set(id, { ...previous, lastMessage: undefined })
      }),
      listen<{ id: string; isTyping: boolean; previewText?: string }>('typing-changed', ({ payload }) => {
        const { id, isTyping, previewText } = payload
        const previous = runtimeStates.value.get(id)
        runtimeStates.value.set(id, { ...previous, id, isTyping, previewText, status: previous?.status ?? 'Disconnected' })
      }),
      listen('connections-changed', () => { void reload() }),
      listen<string>('connection-removed', ({ payload }) => {
        runtimeStates.value.delete(payload)
      }),
    ])
    if (disposed) {
      listeners.forEach((unlisten) => unlisten())
      return
    }
    unlistenFns = listeners
  }

  async function mutation(command: string, args: Record<string, unknown>) {
    error.value = null
    try {
      await withTimeout(invoke(command, args), `Операция ${command}`)
    } finally {
      // The backend event/snapshot is authoritative even when the IPC call
      // times out after the mutation was accepted.
      await reload()
    }
  }

  void subscribe().then(reload).catch((reason) => {
    error.value = normalizeConnectionError(reason)
  })

  onUnmounted(() => {
    disposed = true
    unlistenFns.splice(0).forEach((unlisten) => unlisten())
    subscribed = false
  })

  return {
    configs,
    runtimeStates,
    connections,
    loading,
    error,
    reload,
    add: (config) => mutation('add_connection', { config }),
    update: (id, config) => mutation('update_connection', { id, config }),
    remove: (id) => mutation('remove_connection', { id }),
    connect: (id) => mutation('connect_connection', { id }),
    disconnect: (id) => mutation('disconnect_connection', { id }),
  }
}
