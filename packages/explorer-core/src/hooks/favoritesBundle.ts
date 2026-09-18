import * as v from "valibot"

import {Address} from "@ton/core"

import type {RegisteredAddressName} from "../metadata/types"
import {parseFavoriteAccounts, type FavoriteAccount} from "./useFavoriteAccounts"
import {parseFavoriteBlocks, type FavoriteBlock} from "./useFavoriteBlocks"
import {parseFavoriteTransactions, type FavoriteTransaction} from "./useFavoriteTransactions"

export const FAVORITES_BUNDLE_FORMAT = "acton-favorites"
export const FAVORITES_BUNDLE_VERSION = 1

export interface FavoritesBundle {
  readonly format: typeof FAVORITES_BUNDLE_FORMAT
  readonly version: typeof FAVORITES_BUNDLE_VERSION
  readonly exportedAt: string
  readonly network: string
  readonly accounts: readonly FavoriteAccount[]
  readonly blocks: readonly FavoriteBlock[]
  readonly transactions: readonly FavoriteTransaction[]
  readonly addressNames: readonly RegisteredAddressName[]
}

export interface CreateFavoritesBundleOptions {
  readonly network: string
  readonly accounts: readonly FavoriteAccount[]
  readonly blocks: readonly FavoriteBlock[]
  readonly transactions: readonly FavoriteTransaction[]
  readonly addressNames: readonly RegisteredAddressName[]
}

const favoritesFormatError = "This file is not a supported Acton favorites bundle"
const favoritesNetworkError = "The favorites bundle does not specify a network"

const FavoritesBundleSchema = v.pipe(
  v.unknown(),
  v.check(value => !Array.isArray(value), "The JSON root must be an object"),
  v.object(
    {
      format: v.literal(FAVORITES_BUNDLE_FORMAT, favoritesFormatError),
      version: v.literal(FAVORITES_BUNDLE_VERSION, favoritesFormatError),
      network: v.pipe(v.string(favoritesNetworkError), v.trim(), v.nonEmpty(favoritesNetworkError)),
      exportedAt: v.fallback(v.string(), ""),
      accounts: v.optional(v.array(v.unknown(), "The accounts section must be an array"), []),
      blocks: v.optional(v.array(v.unknown(), "The blocks section must be an array"), []),
      transactions: v.optional(
        v.array(v.unknown(), "The transactions section must be an array"),
        [],
      ),
      addressNames: v.optional(
        v.array(v.unknown(), "The address names section must be an array"),
        [],
      ),
    },
    "The JSON root must be an object",
  ),
)

const AddressNameSchema = v.object({
  address: v.pipe(v.string(), v.trim(), v.nonEmpty()),
  name: v.pipe(v.string(), v.trim(), v.nonEmpty()),
})

export function createFavoritesBundle(options: CreateFavoritesBundleOptions): FavoritesBundle {
  return {
    format: FAVORITES_BUNDLE_FORMAT,
    version: FAVORITES_BUNDLE_VERSION,
    exportedAt: new Date().toISOString(),
    network: options.network,
    accounts: [...options.accounts],
    blocks: [...options.blocks],
    transactions: [...options.transactions],
    addressNames: normalizeAddressNames(options.addressNames),
  }
}

export function parseFavoritesBundle(raw: string): FavoritesBundle {
  let value: unknown
  try {
    value = JSON.parse(raw) as unknown
  } catch {
    throw new Error("The selected file is not valid JSON")
  }

  const result = v.safeParse(FavoritesBundleSchema, value, {abortEarly: true})
  if (!result.success) {
    throw new Error(result.issues[0].message)
  }

  const bundle = result.output

  return {
    ...bundle,
    accounts: parseFavoriteAccounts(JSON.stringify(bundle.accounts)),
    blocks: parseFavoriteBlocks(JSON.stringify(bundle.blocks)),
    transactions: parseFavoriteTransactions(JSON.stringify(bundle.transactions)),
    addressNames: parseAddressNames(bundle.addressNames),
  }
}

function parseAddressNames(value: readonly unknown[]): readonly RegisteredAddressName[] {
  const namesByAddress = new Map<string, RegisteredAddressName>()
  for (const candidate of value) {
    const result = v.safeParse(AddressNameSchema, candidate)
    if (!result.success) {
      continue
    }

    const {address, name} = result.output

    const key = normalizeAddressKey(address)
    if (!namesByAddress.has(key)) {
      namesByAddress.set(key, {address, name})
    }
  }

  return [...namesByAddress.values()]
}

function normalizeAddressNames(
  entries: readonly RegisteredAddressName[],
): readonly RegisteredAddressName[] {
  const namesByAddress = new Map<string, RegisteredAddressName>()
  for (const entry of entries) {
    const address = entry.address.trim()
    const name = entry.name.trim()
    if (!address || !name) {
      continue
    }

    const key = normalizeAddressKey(address)
    namesByAddress.set(key, {address, name})
  }
  return [...namesByAddress.values()]
}

function normalizeAddressKey(address: string): string {
  try {
    return Address.parse(address).toRawString()
  } catch {
    return address
  }
}
