import type {ContractABI} from "@ton/tolk-abi-to-typescript"

import * as v from "valibot"

export type EmulateAbiEndpoint = "destination" | "source"

interface EmulateNavigationCommonPayload {
  readonly targetAddress: string
  readonly sourceAddress: string
  readonly messageValue: string
  readonly messageTransport: "internal" | "external"
  readonly bounce: boolean
  readonly mcSeqnoInput: string
  readonly rawMessage: string
}

export type EmulateNavigationPayload = EmulateNavigationCommonPayload &
  (
    | {
        readonly inputMode: "builder"
        readonly builder: {
          readonly abi?: ContractABI
          readonly abiSourceMode: "auto" | "manual"
          readonly abiEndpoint: EmulateAbiEndpoint
          readonly messageName: string
          readonly argsJson: string
        }
      }
    | {
        readonly inputMode: "raw"
      }
  )

export interface EmulateNavigationState {
  readonly emulatePayload: EmulateNavigationPayload
}

// Validate the ABI envelope here; the compiler ABI library owns type-index and message decoding.
const ContractAbiEnvelopeSchema = v.object({
  contract_name: v.string(),
  compiler_name: v.string(),
  compiler_version: v.string(),
  storage: v.pipe(
    v.unknown(),
    v.check(value => !Array.isArray(value)),
    v.record(v.string(), v.unknown()),
  ),
  unique_types: v.array(v.unknown()),
  struct_instantiations: v.array(v.unknown()),
  alias_instantiations: v.array(v.unknown()),
  declarations: v.array(v.unknown()),
  incoming_messages: v.array(v.unknown()),
  incoming_external: v.array(v.unknown()),
  outgoing_messages: v.array(v.unknown()),
  emitted_events: v.array(v.unknown()),
  get_methods: v.array(v.unknown()),
  thrown_errors: v.array(v.unknown()),
})

const CommonPayloadEntries = {
  targetAddress: v.string(),
  sourceAddress: v.string(),
  messageValue: v.string(),
  messageTransport: v.picklist(["internal", "external"]),
  bounce: v.boolean(),
  mcSeqnoInput: v.string(),
  rawMessage: v.string(),
}

/** Shared by router handoffs and persisted shares so both accept the same emulator inputs. */
export const EmulateNavigationPayloadSchema = v.variant("inputMode", [
  v.object({inputMode: v.literal("raw"), ...CommonPayloadEntries}),
  v.object({
    inputMode: v.literal("builder"),
    ...CommonPayloadEntries,
    builder: v.pipe(
      v.object({
        abi: v.optional(
          v.custom<ContractABI>(value => v.is(ContractAbiEnvelopeSchema, value)),
          () => undefined,
        ),
        abiSourceMode: v.picklist(["auto", "manual"]),
        abiEndpoint: v.picklist(["destination", "source"]),
        messageName: v.string(),
        argsJson: v.pipe(v.string(), v.check(isJson)),
      }),
      v.check(builder => builder.abiSourceMode !== "manual" || builder.abi !== undefined),
    ),
  }),
])

const NavigationStateSchema = v.object({emulatePayload: EmulateNavigationPayloadSchema})

/** Ignores invalid router state without partially applying it to the emulator form. */
export function readEmulateNavigationPayload(state: unknown): EmulateNavigationPayload | undefined {
  const result = v.safeParse(NavigationStateSchema, state)

  return result.success ? result.output.emulatePayload : undefined
}

function isJson(value: string): boolean {
  try {
    JSON.parse(value)
    return true
  } catch {
    return false
  }
}
