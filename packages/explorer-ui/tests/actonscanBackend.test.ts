import {env} from "node:process"
import type {NetworkTpsSnapshot} from "@acton/explorer-core/api/networkStats"
import {afterEach, beforeEach, expect, mock, spyOn, test} from "bun:test"

import {loadNetworkTps} from "../src/actonscanBackend"

// biome-ignore lint/style/noProcessEnv: Preserve the caller's configuration for transport tests
const originalBackendUrl = env.VITE_ACTONSCAN_BACKEND_URL
const window = {
  window_seconds: 60,
  coverage_seconds: 60,
  transactions: 150,
  tps: 2.5,
  complete: true,
}
const snapshot = {
  status: "ready",
  latest_masterchain_seqno: 42,
  latest_block_time: 1_800_000_000,
  windows: [window],
}

beforeEach(() => {
  env.VITE_ACTONSCAN_BACKEND_URL = " https://actonscan.example/backend/ "
})

afterEach(() => {
  mock.restore()
  if (originalBackendUrl === undefined) {
    // biome-ignore lint/style/noProcessEnv: Restore the original environment after each test
    // biome-ignore lint/performance/noDelete: Assigning undefined would create a string value
    delete env.VITE_ACTONSCAN_BACKEND_URL
  } else {
    env.VITE_ACTONSCAN_BACKEND_URL = originalBackendUrl
  }
})

test("loads a validated TPS snapshot and strips unrelated backend fields", async () => {
  const fetchMock = spyOn(globalThis, "fetch").mockResolvedValue(
    Response.json({
      ...snapshot,
      debug: "internal",
      windows: [{...window, debug: "internal"}],
    }),
  )

  const result = await loadNetworkTps(new AbortController().signal)
  const [url, options] = fetchMock.mock.calls[0]
  expect({url, accept: new Headers(options?.headers).get("Accept"), result}).toMatchInlineSnapshot(`
    {
      "accept": "application/json",
      "result": {
        "latest_block_time": 1800000000,
        "latest_masterchain_seqno": 42,
        "status": "ready",
        "windows": [
          {
            "complete": true,
            "coverage_seconds": 60,
            "tps": 2.5,
            "transactions": 150,
            "window_seconds": 60,
          },
        ],
      },
      "url": "https://actonscan.example/backend/api/v1/stats/tps",
    }
  `)
})

test("normalizes null and missing block metadata while the backend is syncing", async () => {
  const fetchMock = spyOn(globalThis, "fetch")
  const results: NetworkTpsSnapshot[] = []
  for (const metadata of [
    {},
    {latest_masterchain_seqno: null, latest_block_time: null},
    {latest_masterchain_seqno: 0, latest_block_time: 0},
  ]) {
    fetchMock.mockResolvedValue(Response.json({status: "syncing", windows: [], ...metadata}))
    // biome-ignore lint/performance/noAwaitInLoops: Each response uses the same fetch mock
    results.push(await loadNetworkTps(new AbortController().signal))
  }
  expect(results).toMatchInlineSnapshot(`
    [
      {
        "latest_block_time": undefined,
        "latest_masterchain_seqno": undefined,
        "status": "syncing",
        "windows": [],
      },
      {
        "latest_block_time": undefined,
        "latest_masterchain_seqno": undefined,
        "status": "syncing",
        "windows": [],
      },
      {
        "latest_block_time": 0,
        "latest_masterchain_seqno": 0,
        "status": "syncing",
        "windows": [],
      },
    ]
  `)
})

test("rejects malformed TPS responses with the offending field path", async () => {
  const fetchMock = spyOn(globalThis, "fetch")
  const invalidResponses = [
    {name: "null snapshot", body: null},
    {name: "array snapshot", body: []},
    {name: "unknown status", body: {...snapshot, status: "offline"}},
    {name: "non-array windows", body: {...snapshot, windows: {}}},
    {name: "null window", body: {...snapshot, windows: [null]}},
    {name: "string seqno", body: {...snapshot, latest_masterchain_seqno: "42"}},
    {name: "negative timestamp", body: {...snapshot, latest_block_time: -1}},
    {name: "zero duration", body: {...snapshot, windows: [{...window, window_seconds: 0}]}},
    {
      name: "fractional coverage",
      body: {...snapshot, windows: [{...window, coverage_seconds: 0.5}]},
    },
    {name: "negative transactions", body: {...snapshot, windows: [{...window, transactions: -1}]}},
    {
      name: "unsafe integer",
      body: {...snapshot, windows: [{...window, transactions: Number.MAX_SAFE_INTEGER + 1}]},
    },
    {name: "negative TPS", body: {...snapshot, windows: [{...window, tps: -0.5}]}},
    {name: "missing TPS", body: {...snapshot, windows: [{...window, tps: undefined}]}},
    {name: "string complete", body: {...snapshot, windows: [{...window, complete: "true"}]}},
  ]

  const results: {name: string; outcome: string}[] = []
  for (const {name, body} of invalidResponses) {
    fetchMock.mockResolvedValue(Response.json(body))
    // biome-ignore lint/performance/noAwaitInLoops: Each response uses the same fetch mock
    const outcome = await loadNetworkTps(new AbortController().signal).then(
      () => "accepted",
      (error: Error) => error.message,
    )
    results.push({name, outcome})
  }
  expect(results).toMatchInlineSnapshot(`
    [
      {
        "name": "null snapshot",
        "outcome": "Actonscan backend returned an invalid TPS snapshot at root",
      },
      {
        "name": "array snapshot",
        "outcome": "Actonscan backend returned an invalid TPS snapshot at status",
      },
      {
        "name": "unknown status",
        "outcome": "Actonscan backend returned an invalid TPS snapshot at status",
      },
      {
        "name": "non-array windows",
        "outcome": "Actonscan backend returned an invalid TPS snapshot at windows",
      },
      {
        "name": "null window",
        "outcome": "Actonscan backend returned an invalid TPS snapshot at windows.0",
      },
      {
        "name": "string seqno",
        "outcome": "Actonscan backend returned an invalid TPS snapshot at latest_masterchain_seqno",
      },
      {
        "name": "negative timestamp",
        "outcome": "Actonscan backend returned an invalid TPS snapshot at latest_block_time",
      },
      {
        "name": "zero duration",
        "outcome": "Actonscan backend returned an invalid TPS snapshot at windows.0.window_seconds",
      },
      {
        "name": "fractional coverage",
        "outcome": "Actonscan backend returned an invalid TPS snapshot at windows.0.coverage_seconds",
      },
      {
        "name": "negative transactions",
        "outcome": "Actonscan backend returned an invalid TPS snapshot at windows.0.transactions",
      },
      {
        "name": "unsafe integer",
        "outcome": "Actonscan backend returned an invalid TPS snapshot at windows.0.transactions",
      },
      {
        "name": "negative TPS",
        "outcome": "Actonscan backend returned an invalid TPS snapshot at windows.0.tps",
      },
      {
        "name": "missing TPS",
        "outcome": "Actonscan backend returned an invalid TPS snapshot at windows.0.tps",
      },
      {
        "name": "string complete",
        "outcome": "Actonscan backend returned an invalid TPS snapshot at windows.0.complete",
      },
    ]
  `)
})

test("rejects non-finite TPS decoded from valid JSON", async () => {
  const body = JSON.stringify({...snapshot, windows: [{...window, tps: "overflow"}]}).replace(
    '"overflow"',
    "1e400",
  )
  spyOn(globalThis, "fetch").mockResolvedValue(new Response(body))

  await expect(
    loadNetworkTps(new AbortController().signal),
  ).rejects.toThrowErrorMatchingInlineSnapshot(
    `"Actonscan backend returned an invalid TPS snapshot at windows.0.tps"`,
  )
})

test("retains the HTTP status when the backend rejects the request", async () => {
  spyOn(globalThis, "fetch").mockResolvedValue(new Response("Unavailable", {status: 503}))

  await expect(
    loadNetworkTps(new AbortController().signal),
  ).rejects.toThrowErrorMatchingInlineSnapshot(`"Actonscan backend returned HTTP 503"`)
})

test("forwards caller cancellation to the backend request", async () => {
  const controller = new AbortController()
  const fetchMock = spyOn(globalThis, "fetch").mockResolvedValue(Response.json(snapshot))
  await loadNetworkTps(controller.signal)
  const requestSignal = fetchMock.mock.calls[0][1]?.signal
  const abortedBefore = requestSignal?.aborted

  controller.abort()

  expect({abortedBefore, abortedAfter: requestSignal?.aborted}).toMatchInlineSnapshot(`
    {
      "abortedAfter": true,
      "abortedBefore": false,
    }
  `)
})
