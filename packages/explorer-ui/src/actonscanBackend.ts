import type {LoadNetworkTps} from "@acton/explorer-core/api/networkStats"
// biome-ignore lint/performance/noNamespaceImport: Valibot supports tree shaking through its namespace API
import * as v from "valibot"

const TPS_PATH = "api/v1/stats/tps"
const DEFAULT_ACTONSCAN_BACKEND_URL = "https://api.actonscan.com/"
const REQUEST_TIMEOUT_MS = 4000

const NonnegativeIntegerSchema = v.pipe(v.number(), v.safeInteger(), v.minValue(0))
const NetworkTpsSnapshotSchema = v.object({
  status: v.picklist(["syncing", "ready"]),
  // The backend uses null until the first block arrives; the core model uses undefined.
  latest_masterchain_seqno: v.nullish(NonnegativeIntegerSchema, () => undefined),
  latest_block_time: v.nullish(NonnegativeIntegerSchema, () => undefined),
  windows: v.array(
    v.object({
      window_seconds: v.pipe(NonnegativeIntegerSchema, v.minValue(1)),
      coverage_seconds: NonnegativeIntegerSchema,
      transactions: NonnegativeIntegerSchema,
      tps: v.pipe(v.number(), v.finite(), v.minValue(0)),
      complete: v.boolean(),
    }),
  ),
})

/** Validates Actonscan's response at the transport boundary before exposing it to core views. */
export const loadNetworkTps: LoadNetworkTps = async signal => {
  const requestSignal = AbortSignal.any([signal, AbortSignal.timeout(REQUEST_TIMEOUT_MS)])
  const response = await fetch(tpsUrl(), {
    headers: {Accept: "application/json"},
    signal: requestSignal,
  })
  if (!response.ok) {
    throw new Error(`Actonscan backend returned HTTP ${response.status}`)
  }

  const result = v.safeParse(NetworkTpsSnapshotSchema, await response.json(), {abortEarly: true})
  if (!result.success) {
    const path = v.getDotPath(result.issues[0]) ?? "root"
    throw new Error(`Actonscan backend returned an invalid TPS snapshot at ${path}`)
  }

  return result.output
}

function tpsUrl(): string {
  const configured =
    import.meta.env.VITE_ACTONSCAN_BACKEND_URL?.trim() || DEFAULT_ACTONSCAN_BACKEND_URL
  return `${configured.replace(/\/$/, "")}/${TPS_PATH}`
}
