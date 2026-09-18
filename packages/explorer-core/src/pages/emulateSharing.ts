import * as v from "valibot"

import type {RawMessageEmulationOptions} from "../retrace/txTrace/lib/emulateRawMessage"
import {
  EmulateNavigationPayloadSchema,
  type EmulateNavigationPayload,
} from "./emulateNavigationPayload"

export const EMULATION_SHARE_QUERY_PARAM = "share"
export const SHARED_EMULATION_VERSION = 1

const MAX_SHARED_ACCOUNT_OVERRIDES = 64
const MAX_UINT32 = 0xff_ff_ff_ff

const Uint32Schema = v.pipe(v.number(), v.integer(), v.minValue(0), v.maxValue(MAX_UINT32))

const OptionalStringSchema = v.pipe(
  v.optional(v.string(), () => undefined),
  v.transform(value => value || undefined),
)

const AccountStateSchema = v.variant("type", [
  v.object({type: v.literal("uninit")}),
  v.object({type: v.literal("frozen"), stateHash: OptionalStringSchema}),
  v.object({
    type: v.literal("active"),
    codeBoc: OptionalStringSchema,
    dataBoc: OptionalStringSchema,
  }),
])

const AccountOverridesSchema = v.pipe(
  v.unknown(),
  v.check(value => !Array.isArray(value)),
  v.record(
    v.pipe(v.string(), v.nonEmpty()),
    v.object({
      balance: OptionalStringSchema,
      lastTransactionLt: OptionalStringSchema,
      lastTransactionHash: OptionalStringSchema,
      state: v.optional(AccountStateSchema, () => undefined),
    }),
  ),
  v.maxEntries(MAX_SHARED_ACCOUNT_OVERRIDES),
  v.transform(overrides => Object.assign(Object.create(null) as typeof overrides, overrides)),
)

/** The same schema validates incoming shares in the browser and in the storage worker. */
export const SharedEmulationSchema = v.object({
  version: v.literal(SHARED_EMULATION_VERSION),
  input: v.pipe(
    EmulateNavigationPayloadSchema,
    v.check(input => {
      const mcSeqno = Number(input.mcSeqnoInput)

      return (
        input.rawMessage.trim().length > 0 &&
        v.is(Uint32Schema, mcSeqno) &&
        input.mcSeqnoInput === String(mcSeqno)
      )
    }),
  ),
  options: v.object({
    accountStateOverrides: v.optional(AccountOverridesSchema, () => undefined),
    ignoreChksig: v.boolean(),
    now: v.optional(Uint32Schema, () => undefined),
  }),
})

/** Portable emulator input; the schema validates its JSON representation at external boundaries. */
export interface SharedEmulation {
  readonly version: typeof SHARED_EMULATION_VERSION
  readonly input: EmulateNavigationPayload
  readonly options: {
    readonly accountStateOverrides?: RawMessageEmulationOptions["accountStateOverrides"]
    readonly ignoreChksig: boolean
    readonly now?: number
  }
}

const ShareResponseSchema = v.object({
  id: v.pipe(v.string(), v.nonEmpty()),
  expiresAt: v.pipe(v.number(), v.finite(), v.gtValue(0)),
})

const LoadedShareSchema = v.object({emulation: SharedEmulationSchema})

const ErrorResponseSchema = v.object({error: v.string()})

interface EmulationShareResponse {
  readonly id: string
  readonly expiresAt: number
}

export async function createEmulationShare(
  apiPath: string,
  emulation: SharedEmulation,
): Promise<EmulationShareResponse> {
  const response = await fetch(apiPath, {
    method: "POST",
    headers: {"content-type": "application/json"},
    body: JSON.stringify(emulation),
  })

  const payload = await readJsonResponse(response, "Failed to create emulation share")

  const result = v.safeParse(ShareResponseSchema, payload)
  if (!result.success) {
    throw new Error("Emulation share API returned an invalid response")
  }

  return result.output
}

export async function loadEmulationShare(apiPath: string, id: string): Promise<SharedEmulation> {
  const response = await fetch(`${apiPath}/${encodeURIComponent(id)}`)
  const payload = await readJsonResponse(response, "Failed to load shared emulation")

  const result = v.safeParse(LoadedShareSchema, payload)
  if (!result.success) {
    throw new Error("Emulation share API returned an invalid response")
  }

  return result.output.emulation
}

/** Rejects a malformed share atomically instead of applying only valid options. */
export function parseSharedEmulation(value: unknown): SharedEmulation | undefined {
  const result = v.safeParse(SharedEmulationSchema, value)

  return result.success ? result.output : undefined
}

async function readJsonResponse(response: Response, fallback: string): Promise<unknown> {
  let payload: unknown
  try {
    payload = await response.json()
  } catch {
    if (!response.ok) {
      throw new Error(fallback)
    }
    return undefined
  }

  if (!response.ok) {
    const result = v.safeParse(ErrorResponseSchema, payload)

    throw new Error(result.success ? result.output.error : fallback)
  }

  return payload
}
