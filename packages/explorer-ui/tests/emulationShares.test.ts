import {expect, spyOn, test} from "bun:test"
import {deepEquals, file} from "bun"

import {
  createEmulationShare,
  loadEmulationShare,
  type SharedEmulation,
} from "@acton/explorer-core/pages/emulateSharing"
import type {ContractABI} from "@ton/tolk-abi-to-typescript"

import {onRequest as createShare} from "../functions/api/emulations"
import {onRequest as readShare} from "../functions/api/emulations/[id]"
import {EMULATION_SHARE_TTL_MS, type EmulationShareBucket} from "../worker/emulationShares"

const EMULATION = {
  version: 1,
  input: {
    inputMode: "raw",
    targetAddress: "",
    sourceAddress: "",
    messageValue: "0.5",
    messageTransport: "internal",
    bounce: true,
    mcSeqnoInput: "42",
    rawMessage: "te6ccgEBAQEAAgAAAA==",
  },
  options: {
    accountStateOverrides: {
      EQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAM9c: {
        balance: "500000000",
        state: {type: "active", dataBoc: "b5ee9c72"},
      },
    },
    ignoreChksig: true,
    now: 1_753_444_800,
  },
} as const

test("round-trips catalog ABIs and every account state through the client, worker, and storage", async () => {
  const catalog = (await file(
    new URL("../../../crates/acton-abi-catalog/data/data-abis.json", import.meta.url),
  ).json()) as {contracts: {id: string; compilerAbi: ContractABI}[]}

  const cases: {name: string; emulation: SharedEmulation}[] = [
    {
      name: "raw without optional settings",
      emulation: {...EMULATION, options: {ignoreChksig: false}},
    },
    {
      name: "all account states and uint32 boundaries",
      emulation: {
        ...EMULATION,
        input: {...EMULATION.input, mcSeqnoInput: "0"},
        options: {
          ignoreChksig: false,
          now: 0xff_ff_ff_ff,
          accountStateOverrides: {
            [`0:${"00".repeat(32)}`]: {},
            [`0:${"11".repeat(32)}`]: {balance: "0", state: {type: "uninit"}},
            [`-1:${"22".repeat(32)}`]: {state: {type: "frozen"}},
            [`0:${"33".repeat(32)}`]: {state: {type: "frozen", stateHash: "44".repeat(32)}},
            [`0:${"55".repeat(32)}`]: {state: {type: "active"}},
            [`0:${"66".repeat(32)}`]: {
              balance: "18446744073709551615",
              lastTransactionLt: "18446744073709551615",
              lastTransactionHash: "77".repeat(32),
              state: {
                type: "active",
                codeBoc: EMULATION.input.rawMessage,
                dataBoc: EMULATION.input.rawMessage,
              },
            },
          },
        },
      },
    },
    {
      name: "automatic builder without ABI",
      emulation: {
        ...EMULATION,
        input: {
          ...EMULATION.input,
          inputMode: "builder",
          builder: {
            abiSourceMode: "auto",
            abiEndpoint: "destination",
            messageName: "",
            argsJson: "{}",
          },
        },
      },
    },
  ]

  for (const contract of [
    ...catalog.contracts,
    {
      id: "anonymous ABI with extensions",
      compilerAbi: {
        ...catalog.contracts[0].compilerAbi,
        contract_name: "",
        author: "Author",
        extension: {data: [null, false, 0]},
      },
    },
  ]) {
    cases.push({
      name: contract.id,
      emulation: {
        ...EMULATION,
        input: {
          ...EMULATION.input,
          inputMode: "builder",
          messageTransport: "external",
          mcSeqnoInput: "4294967295",
          builder: {
            abi: contract.compilerAbi,
            abiSourceMode: "manual",
            abiEndpoint: "source",
            messageName: "",
            argsJson: "{}",
          },
        },
      },
    })
  }

  const bucket = new MemoryEmulationShareBucket()
  const fetchMock = spyOn(globalThis, "fetch").mockImplementation((input, init) => {
    const request = new Request(input, init)
    const id = new URL(request.url).pathname.split("/").at(-1)
    const context = createContext(request, bucket, id)

    return request.method === "POST" ? createShare(context) : readShare(context)
  })

  try {
    const outcomes = await Promise.all(
      cases.map(async ({name, emulation}) => {
        const created = await createEmulationShare(
          "https://actonscan.example/api/emulations",
          emulation,
        )
        const restored = await loadEmulationShare(
          "https://actonscan.example/api/emulations",
          created.id,
        )

        return {name, matches: deepEquals(JSON.parse(JSON.stringify(restored)), emulation)}
      }),
    )

    expect({
      catalogCount: catalog.contracts.length,
      storedCount: bucket.objects.size,
      mismatches: outcomes.filter(outcome => !outcome.matches),
    }).toMatchInlineSnapshot(`
      {
        "catalogCount": 288,
        "mismatches": [],
        "storedCount": 292,
      }
    `)
  } finally {
    fetchMock.mockRestore()
  }
})

test("stores and reads an emulation through the R2 binding", async () => {
  const bucket = new MemoryEmulationShareBucket()
  const beforeCreate = Date.now()
  const createdResponse = await createShare(
    createContext(
      new Request("https://actonscan.example/api/emulations", {
        method: "POST",
        headers: {"content-type": "application/json"},
        body: JSON.stringify(EMULATION),
      }),
      bucket,
    ),
  )
  const created = (await createdResponse.json()) as {id: string; expiresAt: number}
  const afterCreate = Date.now()

  const readResponse = await readShare(
    createContext(
      new Request(`https://actonscan.example/api/emulations/${created.id}`),
      bucket,
      created.id,
    ),
  )
  const readBody = (await readResponse.json()) as {
    readonly emulation: unknown
    readonly expiresAt: number
  }

  expect({
    created: {
      status: createdResponse.status,
      idIsUuid: /^[0-9a-f-]{36}$/i.test(created.id),
      expiresInRange:
        created.expiresAt >= beforeCreate + EMULATION_SHARE_TTL_MS &&
        created.expiresAt <= afterCreate + EMULATION_SHARE_TTL_MS,
      cacheControl: createdResponse.headers.get("cache-control"),
    },
    read: {
      status: readResponse.status,
      cacheControl: readResponse.headers.get("cache-control"),
      expiresAtMatches: readBody.expiresAt === created.expiresAt,
      emulation: readBody.emulation,
    },
    storedKeyMatches: [...bucket.objects.keys()].includes(`emulations/${created.id}.json`),
  }).toMatchInlineSnapshot(`
    {
      "created": {
        "cacheControl": "no-store",
        "expiresInRange": true,
        "idIsUuid": true,
        "status": 201,
      },
      "read": {
        "cacheControl": "no-store",
        "emulation": {
          "input": {
            "bounce": true,
            "inputMode": "raw",
            "mcSeqnoInput": "42",
            "messageTransport": "internal",
            "messageValue": "0.5",
            "rawMessage": "te6ccgEBAQEAAgAAAA==",
            "sourceAddress": "",
            "targetAddress": "",
          },
          "options": {
            "accountStateOverrides": {
              "EQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAM9c": {
                "balance": "500000000",
                "state": {
                  "dataBoc": "b5ee9c72",
                  "type": "active",
                },
              },
            },
            "ignoreChksig": true,
            "now": 1753444800,
          },
          "version": 1,
        },
        "expiresAtMatches": true,
        "status": 200,
      },
      "storedKeyMatches": true,
    }
  `)
})

test("rejects unavailable storage, invalid payloads, and unknown shares", async () => {
  const bucket = new MemoryEmulationShareBucket()
  const unavailable = await createShare({
    request: new Request("https://actonscan.example/api/emulations", {method: "POST"}),
    env: {},
  })
  const malformed = await createShare(
    createContext(
      new Request("https://actonscan.example/api/emulations", {
        method: "POST",
        headers: {"content-type": "application/json"},
        body: "{}",
      }),
      bucket,
    ),
  )
  const unpinned = await createShare(
    createContext(
      new Request("https://actonscan.example/api/emulations", {
        method: "POST",
        headers: {"content-type": "application/json"},
        body: JSON.stringify({
          ...EMULATION,
          input: {...EMULATION.input, mcSeqnoInput: ""},
        }),
      }),
      bucket,
    ),
  )
  const missing = await readShare(
    createContext(
      new Request("https://actonscan.example/api/emulations/00000000-0000-4000-8000-000000000000"),
      bucket,
      "00000000-0000-4000-8000-000000000000",
    ),
  )

  expect({
    unavailable: {status: unavailable.status, body: await unavailable.json()},
    malformed: {status: malformed.status, body: await malformed.json()},
    unpinned: {status: unpinned.status, body: await unpinned.json()},
    missing: {status: missing.status, body: await missing.json()},
  }).toMatchInlineSnapshot(`
    {
      "malformed": {
        "body": {
          "error": "Request body is not a valid emulation",
        },
        "status": 400,
      },
      "missing": {
        "body": {
          "error": "Emulation share not found",
        },
        "status": 404,
      },
      "unavailable": {
        "body": {
          "error": "Emulation sharing is not configured",
        },
        "status": 503,
      },
      "unpinned": {
        "body": {
          "error": "Request body is not a valid emulation",
        },
        "status": 400,
      },
    }
  `)
})

test("expires shared emulations and removes their R2 objects", async () => {
  const bucket = new MemoryEmulationShareBucket()
  const createdResponse = await createShare(
    createContext(
      new Request("https://actonscan.example/api/emulations", {
        method: "POST",
        headers: {"content-type": "application/json"},
        body: JSON.stringify(EMULATION),
      }),
      bucket,
    ),
  )
  const created = (await createdResponse.json()) as {id: string}
  const storedKey = [...bucket.objects.keys()][0]
  const stored = JSON.parse(bucket.objects.get(storedKey) ?? "{}") as Record<string, unknown>
  bucket.objects.set(storedKey, JSON.stringify({...stored, expiresAt: 2}))
  const backgroundTasks: Promise<unknown>[] = []
  const expiredResponse = await readShare({
    ...createContext(
      new Request(`https://actonscan.example/api/emulations/${created.id}`),
      bucket,
      created.id,
    ),
    waitUntil(promise: Promise<unknown>) {
      backgroundTasks.push(promise)
    },
  })
  await Promise.all(backgroundTasks)

  expect({
    status: expiredResponse.status,
    body: await expiredResponse.json(),
    storedKeys: [...bucket.objects.keys()],
  }).toMatchInlineSnapshot(`
    {
      "body": {
        "error": "Emulation share has expired",
      },
      "status": 410,
      "storedKeys": [],
    }
  `)
})

function createContext(request: Request, bucket: EmulationShareBucket, id?: string) {
  return {
    request,
    env: {EMULATION_SHARES: bucket},
    params: id ? {id} : undefined,
  }
}

class MemoryEmulationShareBucket implements EmulationShareBucket {
  readonly objects = new Map<string, string>()

  get(key: string) {
    const value = this.objects.get(key)
    return Promise.resolve(value === undefined ? null : {text: () => Promise.resolve(value)})
  }

  put(key: string, value: string) {
    this.objects.set(key, value)
    return Promise.resolve({})
  }

  delete(key: string) {
    this.objects.delete(key)
    return Promise.resolve()
  }
}
