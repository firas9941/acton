import {afterEach, beforeEach, expect, mock, spyOn, test} from "bun:test"

import {
  disconnectFaucetSession,
  exchangeGitHubGrant,
  requestFaucetAuthStatus,
  requestFaucetChallenge,
  requestFaucetSession,
  submitFaucetClaim,
  type FaucetSession,
} from "../src/faucetClient"

const originalGlobals = new Map(
  ["location", "localStorage", "sessionStorage"].map(key => [
    key,
    Object.getOwnPropertyDescriptor(globalThis, key),
  ]),
)

const localValues = new Map<string, string>()
const sessionValues = new Map<string, string>()
const sessionKey = "actonscanFaucetSession"
const token = "test-session-token".repeat(3)

const authenticatedSession = {
  authenticated: true,
  githubUserId: 123,
  login: "test-user",
  tier: "verified",
  maxRequests: 5,
  accountAgeDays: 300,
  publicRepos: 4,
  followers: 0,
  expiresAt: null,
  token: null,
}

beforeEach(() => {
  localValues.clear()
  sessionValues.clear()

  for (const [key, values] of [
    ["localStorage", localValues],
    ["sessionStorage", sessionValues],
  ] as const) {
    Object.defineProperty(globalThis, key, {
      configurable: true,
      value: {
        getItem: (name: string) => values.get(name) ?? null,
        setItem: (name: string, value: string) => values.set(name, value),
        removeItem: (name: string) => values.delete(name),
      },
    })
  }

  Object.defineProperty(globalThis, "location", {
    configurable: true,
    value: {origin: "https://faucet.example"},
  })
})

afterEach(() => {
  mock.restore()

  for (const [key, descriptor] of originalGlobals) {
    if (descriptor) {
      Object.defineProperty(globalThis, key, descriptor)
    } else {
      Reflect.deleteProperty(globalThis, key)
    }
  }
})

test("validates auth and challenge responses while preserving defaults and claim messages", async () => {
  const fetchMock = spyOn(globalThis, "fetch")
    .mockResolvedValueOnce(
      Response.json({
        enabled: true,
        guestMaxRequests: 2,
        verifiedMaxRequests: 5,
        establishedMaxRequests: 10,
        windowSeconds: 3600,
        extra: "ignored",
      }),
    )
    .mockResolvedValueOnce(Response.json({version: 1, challenge: "challenge", difficulty: 0}))
    .mockResolvedValueOnce(Response.json({message: "  "}))

  const auth = await requestFaucetAuthStatus(undefined, "https://faucet.example/")
  const challenge = await requestFaucetChallenge("address", {baseUrl: "https://faucet.example/"})
  const claim = await submitFaucetClaim("address", challenge, 0, {
    baseUrl: "https://faucet.example/",
  })

  expect({
    auth,
    challenge,
    claim,
    paths: fetchMock.mock.calls.map(([url]) => new URL(String(url)).pathname),
  }).toMatchInlineSnapshot(`
    {
      "auth": {
        "enabled": true,
        "establishedMaxRequests": 10,
        "guestMaxRequests": 2,
        "verifiedMaxRequests": 5,
        "windowSeconds": 3600,
      },
      "challenge": {
        "challenge": "challenge",
        "difficulty": 0,
        "maxNonceAttempts": 1000000000,
        "maxSolveTtlSeconds": 60,
        "version": 1,
      },
      "claim": {
        "message": "Your testnet claim has been queued",
      },
      "paths": [
        "/auth/status",
        "/challenge",
        "/claim",
      ],
    }
  `)
})

test("rejects malformed challenges before starting proof of work", async () => {
  const fetchMock = spyOn(globalThis, "fetch")
  const valid = {version: 1, challenge: "challenge", difficulty: 8}
  const invalidResponses = [
    null,
    [],
    {...valid, version: 2},
    {...valid, challenge: ""},
    {...valid, difficulty: 257},
    {...valid, difficulty: "8"},
    {...valid, max_solve_ttl_seconds: 0},
    {...valid, max_nonce_attempts: Number.MAX_SAFE_INTEGER + 1},
  ]

  const errors: string[] = []
  for (const response of invalidResponses) {
    fetchMock.mockResolvedValue(Response.json(response))

    errors.push(
      // biome-ignore lint/performance/noAwaitInLoops: Each response replaces the same transport mock
      await requestFaucetChallenge("address").then(
        () => "accepted",
        (error: Error) => error.message,
      ),
    )
  }

  expect(errors).toMatchInlineSnapshot(`
    [
      "Faucet returned an invalid JSON response",
      "Faucet returned an invalid JSON response",
      "Unsupported faucet challenge version: 2",
      "Faucet returned an invalid PoW challenge",
      "Faucet returned an invalid PoW difficulty: 257",
      "Faucet returned an invalid PoW difficulty: 8",
      "Faucet returned an invalid PoW solve time limit: 0",
      "Faucet returned an invalid PoW nonce limit: 9007199254740992",
    ]
  `)
})

test("accepts backend zero limits, nullable session fields, and explicit PoW limits", async () => {
  const fetchMock = spyOn(globalThis, "fetch").mockResolvedValueOnce(
    Response.json({
      enabled: false,
      guestMaxRequests: 0,
      verifiedMaxRequests: 0,
      establishedMaxRequests: 0,
      windowSeconds: 0,
    }),
  )

  const auth = await requestFaucetAuthStatus()
  const sessions: (FaucetSession | undefined)[] = []
  sessionValues.set(sessionKey, token)

  for (const tier of ["guest", "verified", "established"]) {
    fetchMock.mockResolvedValueOnce(
      Response.json({
        ...authenticatedSession,
        tier,
        maxRequests: 0,
        accountAgeDays: 0,
        publicRepos: 0,
      }),
    )

    // biome-ignore lint/performance/noAwaitInLoops: Each tier uses the same fetch mock
    sessions.push(await requestFaucetSession())
  }

  fetchMock.mockResolvedValueOnce(
    Response.json({
      version: 1,
      challenge: "test-challenge",
      difficulty: 256,
      max_solve_ttl_seconds: 300,
      max_nonce_attempts: 1_000_000_000,
    }),
  )

  const challenge = await requestFaucetChallenge("address")

  expect({
    auth,
    sessions,
    challenge,
    tokenRetained: sessionValues.get(sessionKey) === token,
  }).toMatchInlineSnapshot(`
      {
        "auth": {
          "enabled": false,
          "establishedMaxRequests": 0,
          "guestMaxRequests": 0,
          "verifiedMaxRequests": 0,
          "windowSeconds": 0,
        },
        "challenge": {
          "challenge": "test-challenge",
          "difficulty": 256,
          "maxNonceAttempts": 1000000000,
          "maxSolveTtlSeconds": 300,
          "version": 1,
        },
        "sessions": [
          {
            "accountAgeDays": 0,
            "followers": 0,
            "githubUserId": 123,
            "login": "test-user",
            "maxRequests": 0,
            "publicRepos": 0,
            "tier": "guest",
          },
          {
            "accountAgeDays": 0,
            "followers": 0,
            "githubUserId": 123,
            "login": "test-user",
            "maxRequests": 0,
            "publicRepos": 0,
            "tier": "verified",
          },
          {
            "accountAgeDays": 0,
            "followers": 0,
            "githubUserId": 123,
            "login": "test-user",
            "maxRequests": 0,
            "publicRepos": 0,
            "tier": "established",
          },
        ],
        "tokenRetained": true,
      }
    `)
})

test("stores a GitHub token only after the entire session validates", async () => {
  const fetchMock = spyOn(globalThis, "fetch").mockResolvedValueOnce(
    Response.json({...authenticatedSession, followers: -1, token}),
  )

  const failure = await exchangeGitHubGrant("grant").then(
    () => ({name: "accepted", message: "", exposesIssues: false}),
    (error: Error) => ({
      name: error.name,
      message: error.message,
      exposesIssues: "issues" in error,
    }),
  )
  const storedAfterFailure = sessionValues.has(sessionKey)

  fetchMock.mockResolvedValueOnce(Response.json({...authenticatedSession, token, extra: "ignored"}))

  const session = await exchangeGitHubGrant("grant")

  expect({
    failure,
    storedAfterFailure,
    session,
    tokenMatches: sessionValues.get(sessionKey) === token,
  }).toMatchInlineSnapshot(`
      {
        "failure": {
          "exposesIssues": false,
          "message": "Faucet returned an invalid follower count: -1",
          "name": "Error",
        },
        "session": {
          "accountAgeDays": 300,
          "followers": 0,
          "githubUserId": 123,
          "login": "test-user",
          "maxRequests": 5,
          "publicRepos": 4,
          "tier": "verified",
        },
        "storedAfterFailure": false,
        "tokenMatches": true,
      }
    `)
})

test("keeps a session on transient failures and clears it on confirmed unauthorized responses", async () => {
  sessionValues.set(sessionKey, token)

  const fetchMock = spyOn(globalThis, "fetch").mockResolvedValueOnce(
    Response.json({error: "Temporarily unavailable"}, {status: 503}),
  )

  const transientError = await disconnectFaucetSession().then(
    () => "accepted",
    (error: Error) => error.message,
  )
  const retained = sessionValues.get(sessionKey) === token

  fetchMock.mockResolvedValueOnce(Response.json({error: "Session expired"}, {status: 401}))

  const session = await requestFaucetSession()

  expect({
    transientError,
    retained,
    session,
    cleared: !sessionValues.has(sessionKey),
  }).toMatchInlineSnapshot(`
      {
        "cleared": true,
        "retained": true,
        "session": undefined,
        "transientError": "Temporarily unavailable",
      }
    `)
})
