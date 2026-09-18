import * as v from "valibot"

interface FavoriteRecord {
  readonly savedAt: number
}

interface FavoritesStoreOptions<T extends FavoriteRecord> {
  readonly storagePrefix: string
  readonly storageVersion: string
  readonly changeEvent: string
  readonly recordSchema: v.GenericSchema<unknown, T>
  readonly normalize: (favorites: readonly T[]) => readonly T[]
}

interface FavoritesCacheEntry<T> {
  readonly raw: string | null
  readonly value: readonly T[]
}

export interface FavoritesStore<T extends FavoriteRecord> {
  readonly empty: readonly T[]
  readonly merge: (namespace: string, favorites: readonly T[]) => void
  readonly parse: (raw: string | null) => readonly T[]
  readonly read: (namespace: string) => readonly T[]
  readonly subscribe: (namespace: string, onStoreChange: () => void) => () => void
  readonly write: (namespace: string, favorites: readonly T[]) => void
}

export function createFavoritesStore<T extends FavoriteRecord>(
  options: FavoritesStoreOptions<T>,
): FavoritesStore<T> {
  const cache = new Map<string, FavoritesCacheEntry<T>>()
  const empty: readonly T[] = []

  // One damaged entry must not hide the other favorites stored in the same list.
  const StoredFavoritesSchema = v.array(v.fallback(v.optional(options.recordSchema), undefined))

  const storageKey = (namespace: string): string =>
    `${options.storagePrefix}:${namespace}:${options.storageVersion}`

  const readRaw = (namespace: string): string | null => {
    try {
      return globalThis.localStorage?.getItem(storageKey(namespace)) ?? null
    } catch {
      return null
    }
  }

  const parse = (raw: string | null): readonly T[] => {
    if (!raw) {
      return empty
    }

    try {
      const parsed = v.parse(StoredFavoritesSchema, JSON.parse(raw))

      return options.normalize(parsed.filter((entry): entry is T => entry !== undefined))
    } catch {
      return empty
    }
  }

  const read = (namespace: string): readonly T[] => {
    const raw = readRaw(namespace)
    const cached = cache.get(namespace)
    if (cached?.raw === raw) {
      return cached.value
    }

    const value = parse(raw)
    cache.set(namespace, {raw, value})
    return value
  }

  const write = (namespace: string, favorites: readonly T[]): void => {
    const key = storageKey(namespace)
    const value = options.normalize(favorites)

    try {
      if (value.length > 0) {
        const raw = JSON.stringify(value)
        globalThis.localStorage?.setItem(key, raw)
        cache.set(namespace, {raw, value})
      } else {
        globalThis.localStorage?.removeItem(key)
        cache.set(namespace, {raw: null, value})
      }
    } catch {
      cache.set(namespace, {raw: readRaw(namespace), value})
    }

    globalThis.dispatchEvent?.(new CustomEvent(options.changeEvent, {detail: {namespace}}))
  }

  const merge = (namespace: string, favorites: readonly T[]): void => {
    if (favorites.length === 0) {
      return
    }
    write(namespace, [...favorites, ...read(namespace)])
  }

  const subscribe = (namespace: string, onStoreChange: () => void): (() => void) => {
    const handleLocalChange = (event: Event) => {
      const detail = (event as CustomEvent<{readonly namespace?: string}>).detail
      if (!detail?.namespace || detail.namespace === namespace) {
        onStoreChange()
      }
    }
    const handleStorageChange = (event: StorageEvent) => {
      if (event.key === storageKey(namespace)) {
        cache.delete(namespace)
        onStoreChange()
      }
    }

    globalThis.addEventListener?.(options.changeEvent, handleLocalChange)
    globalThis.addEventListener?.("storage", handleStorageChange)

    return () => {
      globalThis.removeEventListener?.(options.changeEvent, handleLocalChange)
      globalThis.removeEventListener?.("storage", handleStorageChange)
    }
  }

  return {empty, merge, parse, read, subscribe, write}
}
