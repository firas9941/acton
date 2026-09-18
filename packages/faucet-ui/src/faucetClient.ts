// biome-ignore lint/performance/noNamespaceImport: Valibot supports tree shaking through its namespace API
import * as v from "valibot"

const DEFAULT_FAUCET_URL = "https://faucet.ton.org"
const DEFAULT_MAX_SOLVE_TTL_SECONDS = 60
const DEFAULT_MAX_NONCE_ATTEMPTS = 1_000_000_000
const FAUCET_CLIENT_HEADER = "actonscan/1.0.0"
const FAUCET_DEVICE_UID_STORAGE_KEY = "actonscanFaucetDeviceUid"
const FAUCET_SESSION_STORAGE_KEY = "actonscanFaucetSession"

export type FaucetTier = "guest" | "verified" | "established"

export interface FaucetChallenge {
  readonly version: number
  readonly challenge: string
  readonly difficulty: number
  readonly maxSolveTtlSeconds: number
  readonly maxNonceAttempts: number
}

export interface FaucetClaim {
  readonly message: string
}

export interface FaucetAuthStatus {
  readonly enabled: boolean
  readonly guestMaxRequests: number
  readonly verifiedMaxRequests: number
  readonly establishedMaxRequests: number
  readonly windowSeconds: number
}

export interface FaucetRequestOptions {
  readonly signal?: AbortSignal
  readonly baseUrl?: string
  readonly authorized?: boolean
}

export interface FaucetSession {
  readonly githubUserId: number
  readonly login: string
  readonly tier: FaucetTier
  readonly maxRequests: number
  readonly accountAgeDays: number
  readonly publicRepos: number
  readonly followers: number
}

export class FaucetRequestError extends Error {
  readonly status: number

  constructor(message: string, status: number) {
    super(message)
    this.name = "FaucetRequestError"
    this.status = status
  }
}

const JsonObjectSchema = v.pipe(
  v.unknown(),
  v.check(value => !Array.isArray(value)),
  v.record(v.string(), v.unknown()),
)

const AuthStatusSchema = v.object({
  enabled: v.fallback(v.boolean(), false),
  // The backend permits zero limits and a zero window when the limiter is disabled.
  guestMaxRequests: integerSchema("guest request limit", 0),
  verifiedMaxRequests: integerSchema("verified request limit", 0),
  establishedMaxRequests: integerSchema("established request limit", 0),
  windowSeconds: integerSchema("request window", 0),
})

const SessionResponseSchema = v.object({
  authenticated: v.literal(true, "Faucet returned an invalid GitHub session"),
  login: v.pipe(
    v.string("Faucet returned an invalid GitHub session"),
    v.nonEmpty("Faucet returned an invalid GitHub session"),
  ),
  tier: v.picklist(
    ["guest", "verified", "established"],
    "Faucet returned an invalid GitHub session",
  ),
  githubUserId: integerSchema("GitHub user ID", 1),
  maxRequests: integerSchema("request limit", 0),
  accountAgeDays: integerSchema("account age", 0),
  publicRepos: integerSchema("public repository count", 0),
  followers: integerSchema("follower count", 0),
})

const SessionSchema = v.pipe(
  SessionResponseSchema,
  v.transform(({authenticated: _, ...session}) => session),
)

const GrantSchema = v.pipe(
  v.object({
    token: v.pipe(
      v.string("Faucet returned an invalid GitHub session token"),
      v.minLength(32, "Faucet returned an invalid GitHub session token"),
    ),
    ...SessionResponseSchema.entries,
  }),
  v.transform(({token, authenticated: _, ...session}) => ({token, session})),
)

const ChallengeSchema = v.pipe(
  v.object({
    max_solve_ttl_seconds: v.optional(
      integerSchema("PoW solve time limit", 1),
      DEFAULT_MAX_SOLVE_TTL_SECONDS,
    ),
    max_nonce_attempts: v.optional(integerSchema("PoW nonce limit", 1), DEFAULT_MAX_NONCE_ATTEMPTS),
    version: v.literal(1, issue => `Unsupported faucet challenge version: ${String(issue.input)}`),
    challenge: v.pipe(
      v.string("Faucet returned an invalid PoW challenge"),
      v.nonEmpty("Faucet returned an invalid PoW challenge"),
    ),
    difficulty: v.message(
      v.pipe(v.number(), v.integer(), v.minValue(0), v.maxValue(256)),
      issue => `Faucet returned an invalid PoW difficulty: ${String(issue.input)}`,
    ),
  }),
  v.transform(({max_solve_ttl_seconds, max_nonce_attempts, ...challenge}) => ({
    ...challenge,
    maxSolveTtlSeconds: max_solve_ttl_seconds,
    maxNonceAttempts: max_nonce_attempts,
  })),
)

const NonblankStringSchema = v.pipe(
  v.string(),
  v.check(value => value.trim().length > 0),
)

const ClaimSchema = v.object({
  message: v.fallback(NonblankStringSchema, "Your testnet claim has been queued"),
})

const ErrorSchema = v.object({
  error: v.fallback(v.optional(NonblankStringSchema), undefined),
  message: v.fallback(v.optional(NonblankStringSchema), undefined),
})

export async function requestFaucetAuthStatus(
  signal?: AbortSignal,
  baseUrl?: string,
): Promise<FaucetAuthStatus> {
  const payload = await faucetGet("auth/status", signal, false, baseUrl)

  return parseFaucetResponse(AuthStatusSchema, payload)
}

/**
 * OAuth stores the query UID verbatim, so it must match the normalized UID
 * that the faucet's request middleware supplies when exchanging the grant
 */
export function githubAuthorizationUrl(baseUrl?: string): string {
  const url = new URL("auth/github/start", faucetBaseUrl(baseUrl))
  url.searchParams.set("device_uid", faucetDeviceUid().replaceAll("-", "").toLowerCase())
  return url.toString()
}

export async function exchangeGitHubGrant(
  grant: string,
  signal?: AbortSignal,
  baseUrl?: string,
): Promise<FaucetSession> {
  const payload = await faucetRequest(
    "auth/exchange",
    {grant},
    {signal, authorized: false, baseUrl},
  )

  const {session, token} = parseFaucetResponse(GrantSchema, payload)
  writeFaucetSessionToken(token)

  return session
}

export async function requestFaucetSession(
  signal?: AbortSignal,
  baseUrl?: string,
): Promise<FaucetSession | undefined> {
  if (!readFaucetSessionToken()) return undefined

  try {
    const payload = await faucetGet("auth/session", signal, true, baseUrl)

    return parseFaucetResponse(SessionSchema, payload)
  } catch (error) {
    if (error instanceof FaucetRequestError && error.status === 401) {
      clearFaucetSession()
      return undefined
    }
    throw error
  }
}

export async function disconnectFaucetSession(
  signal?: AbortSignal,
  baseUrl?: string,
): Promise<void> {
  try {
    if (readFaucetSessionToken()) {
      await faucetFetch("auth/session", {method: "DELETE", signal, baseUrl})
    }
  } catch (error) {
    if (!(error instanceof FaucetRequestError && error.status === 401)) {
      throw error
    }
  }

  // A successful delete and a confirmed unauthorized response both mean there
  // is no usable server-side session left. Transient failures keep the token.
  clearFaucetSession()
}

export function clearFaucetSession(): void {
  try {
    sessionStorage.removeItem(FAUCET_SESSION_STORAGE_KEY)
  } catch {
    // The page remains usable as a guest when browser storage is unavailable
  }
}

export async function requestFaucetChallenge(
  address: string,
  options: FaucetRequestOptions = {},
): Promise<FaucetChallenge> {
  const payload = await faucetRequest("challenge", {address, type: 1}, options)

  return parseFaucetResponse(ChallengeSchema, payload)
}

export async function submitFaucetClaim(
  address: string,
  challenge: FaucetChallenge,
  nonce: number,
  options: FaucetRequestOptions = {},
): Promise<FaucetClaim> {
  const payload = await faucetRequest(
    "claim",
    {
      address,
      version: challenge.version,
      challenge: challenge.challenge,
      nonce,
      type: 1,
    },
    options,
  )

  return parseFaucetResponse(ClaimSchema, payload)
}

function faucetRequest(
  path: string,
  payload: Record<string, unknown>,
  options: FaucetRequestOptions = {},
): Promise<unknown> {
  return faucetFetch(path, {
    method: "POST",
    body: JSON.stringify(payload),
    signal: options.signal,
    authorized: options.authorized,
    contentType: true,
    baseUrl: options.baseUrl,
  })
}

function faucetGet(
  path: string,
  signal?: AbortSignal,
  authorized = true,
  baseUrl?: string,
): Promise<unknown> {
  return faucetFetch(path, {method: "GET", signal, authorized, baseUrl})
}

interface FaucetFetchOptions {
  readonly method: "GET" | "POST" | "DELETE"
  readonly body?: string
  readonly signal?: AbortSignal
  readonly authorized?: boolean
  readonly contentType?: boolean
  readonly baseUrl?: string
}

async function faucetFetch(path: string, options: FaucetFetchOptions): Promise<unknown> {
  const headers: Record<string, string> = {
    "x-acton-client": FAUCET_CLIENT_HEADER,
    "x-device-uid": faucetDeviceUid(),
  }
  if (options.contentType) headers["content-type"] = "application/json"
  if (options.authorized !== false) {
    const sessionToken = readFaucetSessionToken()
    if (sessionToken) headers.authorization = `Bearer ${sessionToken}`
  }

  const response = await fetch(new URL(path, faucetBaseUrl(options.baseUrl)), {
    method: options.method,
    headers,
    body: options.body,
    signal: options.signal,
  })
  const text = await response.text()
  const parsed = parseJson(text)

  if (!response.ok) {
    if (response.status === 401 && options.authorized !== false) clearFaucetSession()
    throw new FaucetRequestError(faucetErrorMessage(parsed, text, response.status), response.status)
  }
  if (response.status === 204) return undefined
  if (!v.is(JsonObjectSchema, parsed)) {
    throw new Error("Faucet returned an invalid JSON response")
  }

  return parsed
}

function faucetBaseUrl(override?: string): string {
  const configured = override?.trim() || import.meta.env.VITE_FAUCET_URL?.trim()
  const value = configured || DEFAULT_FAUCET_URL
  const resolved = new URL(value, globalThis.location.origin).toString()
  return resolved.endsWith("/") ? resolved : `${resolved}/`
}

function faucetDeviceUid(): string {
  try {
    const stored = localStorage.getItem(FAUCET_DEVICE_UID_STORAGE_KEY)
    if (stored && isValidDeviceUid(stored)) {
      return stored
    }

    const generated = generateDeviceUid()
    localStorage.setItem(FAUCET_DEVICE_UID_STORAGE_KEY, generated)
    return generated
  } catch {
    return "default"
  }
}

function readFaucetSessionToken(): string | undefined {
  try {
    const token = sessionStorage.getItem(FAUCET_SESSION_STORAGE_KEY)?.trim()
    return token || undefined
  } catch {
    return undefined
  }
}

function writeFaucetSessionToken(token: string): void {
  try {
    sessionStorage.setItem(FAUCET_SESSION_STORAGE_KEY, token)
  } catch {
    // Requests continue as guest when browser storage is unavailable
  }
}

function generateDeviceUid(): string {
  if (typeof crypto.randomUUID === "function") {
    return crypto.randomUUID()
  }

  const bytes = crypto.getRandomValues(new Uint8Array(16))
  return Array.from(bytes, byte => byte.toString(16).padStart(2, "0")).join("")
}

function isValidDeviceUid(value: string): boolean {
  return value === "default" || value.length === 32 || value.length === 36
}

function integerSchema(label: string, minimum: number) {
  return v.message(
    v.pipe(v.number(), v.safeInteger(), v.minValue(minimum)),
    issue => `Faucet returned an invalid ${label}: ${String(issue.input)}`,
  )
}

// Do not expose Valibot's issue objects: they can contain the OAuth token from the response.
function parseFaucetResponse<TSchema extends v.GenericSchema>(
  schema: TSchema,
  payload: unknown,
): v.InferOutput<TSchema> {
  const result = v.safeParse(schema, payload, {abortEarly: true})
  if (!result.success) {
    throw new Error(result.issues[0].message)
  }

  return result.output
}

function parseJson(value: string): unknown {
  if (!value) return undefined

  try {
    return JSON.parse(value)
  } catch {
    return undefined
  }
}

function faucetErrorMessage(parsed: unknown, raw: string, status: number): string {
  const result = v.safeParse(ErrorSchema, parsed)
  if (result.success) {
    if (result.output.error) return result.output.error
    if (result.output.message) return result.output.message
  }

  if (raw.trim()) return raw.trim()

  return `Faucet request failed with status ${status}`
}
