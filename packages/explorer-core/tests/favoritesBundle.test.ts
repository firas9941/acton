import {describe, expect, test} from "bun:test"

import {createFavoritesBundle, parseFavoritesBundle} from "../src/hooks/favoritesBundle"

const address = `0:${"11".repeat(32)}`
const transactionHash = "ab".repeat(32)

describe("favorites bundles", () => {
  test("round-trips favorites and local address names", () => {
    const bundle = createFavoritesBundle({
      network: "mainnet",
      accounts: [{address, savedAt: 10}],
      blocks: [{workchain: 0, shard: "8000000000000000", seqno: 42, savedAt: 20}],
      transactions: [{hash: transactionHash, account: address, savedAt: 30}],
      addressNames: [
        {address, name: "  Treasury  "},
        {address, name: "Duplicate"},
      ],
    })

    const parsed = parseFavoritesBundle(JSON.stringify(bundle))

    expect(parsed.network).toBe("mainnet")
    expect(parsed.accounts).toHaveLength(1)
    expect(parsed.blocks).toHaveLength(1)
    expect(parsed.transactions).toHaveLength(1)
    expect(parsed.addressNames).toEqual([{address, name: "Duplicate"}])
  })

  test("rejects unsupported bundle formats", () => {
    expect(() => parseFavoritesBundle(JSON.stringify({format: "other", version: 1}))).toThrow(
      "supported Acton favorites bundle",
    )
  })

  test("rejects invalid JSON and malformed sections", () => {
    expect(() => parseFavoritesBundle("not json")).toThrow("not valid JSON")
    expect(() => parseFavoritesBundle("[]")).toThrow("The JSON root must be an object")
    expect(() =>
      parseFavoritesBundle(
        JSON.stringify({
          format: "acton-favorites",
          version: 1,
          network: "mainnet",
          accounts: {},
        }),
      ),
    ).toThrow("accounts section must be an array")
  })

  test("requires a bundle network", () => {
    expect(() =>
      parseFavoritesBundle(JSON.stringify({format: "acton-favorites", version: 1, network: "  "})),
    ).toThrow("does not specify a network")
  })
})
