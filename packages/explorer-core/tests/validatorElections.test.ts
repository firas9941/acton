import {describe, expect, test} from "bun:test"
import {readFile} from "node:fs/promises"

import {
  parseNetworkConfig,
  type NetworkConfig,
  type NetworkConfigParameter,
} from "../src/api/config"
import {validatorElectionFromConfig, validatorElectionStage} from "../src/api/validatorElections"

async function readConfig(filename: string): Promise<NetworkConfig> {
  const raw = await readFile(new URL(`./fixtures/${filename}.boc.base64`, import.meta.url), "utf8")
  return parseNetworkConfig(raw.trim())
}

const mainnet = await readConfig("mainnet-config-84773657")

function withParameters(parameters: readonly NetworkConfigParameter[]): NetworkConfig {
  return {...mainnet, parameters}
}

describe("validator elections from network config", () => {
  test("reads timing, limits and exact weights from a Mainnet snapshot", () => {
    const election = validatorElectionFromConfig(mainnet)
    expect(election).toBeDefined()
    expect(election?.timing).toEqual({
      validatorsElectedFor: 65_536,
      electionsStartBefore: 32_768,
      electionsEndBefore: 8192,
      stakeHeldFor: 32_768,
    })
    expect(election?.count).toEqual({maxValidators: 400, maxMainValidators: 100, minValidators: 75})
    expect(election?.stake).toEqual({
      minStake: 300_000_000_000_000n,
      maxStake: 10_000_000_000_000_000n,
      minTotalStake: 75_000_000_000_000_000n,
      maxStakeFactor: 294_912,
    })
    expect(election?.current.total).toBe(396)
    expect(election?.previous?.total).toBe(400)
    expect(election?.next).toBeUndefined()
    expect(
      election?.current.validators.reduce((total, validator) => total + validator.weight, 0n),
    ).toBe(election?.current.totalWeight)
  })

  test("uses Testnet's shorter election schedule", async () => {
    const election = validatorElectionFromConfig(await readConfig("testnet-config-80890417"))
    expect(election?.timing).toEqual({
      validatorsElectedFor: 3600,
      electionsStartBefore: 1800,
      electionsEndBefore: 300,
      stakeHeldFor: 1800,
    })
    expect(election?.current.total).toBe(14)
    expect(election?.count?.maxValidators).toBe(45)
    expect(election?.stake?.minStake).toBe(2_000_000_000_000_000n)
  })

  test("prefers temporary sets and falls back to ordinary sets", () => {
    const ordinary = mainnet.parameters.find(parameter => parameter.id === 34)
    if (!ordinary?.validatorSet) throw new Error("Missing fixture validator set")
    const parameters = mainnet.parameters.filter(
      parameter => parameter.id < 32 || parameter.id > 37,
    )
    const regular = ordinary.validatorSet
    const temporary = {...regular, total: 1, validators: regular.validators.slice(0, 1)}
    for (const id of [32, 34, 36]) {
      parameters.push({...ordinary, id, validatorSet: regular})
      parameters.push({...ordinary, id: id + 1, validatorSet: temporary})
    }
    const election = validatorElectionFromConfig(withParameters(parameters))
    expect(election?.previous).toBe(temporary)
    expect(election?.current).toBe(temporary)
    expect(election?.next).toBe(temporary)

    const fallback = validatorElectionFromConfig(
      withParameters(parameters.filter(parameter => ![33, 35, 37].includes(parameter.id))),
    )
    expect(fallback?.previous).toBe(regular)
    expect(fallback?.current).toBe(regular)
    expect(fallback?.next).toBe(regular)
  })

  test("requires timing and the current set but allows optional parameters to be missing", () => {
    for (const missingId of [15, 34]) {
      expect(
        validatorElectionFromConfig(
          withParameters(mainnet.parameters.filter(parameter => parameter.id !== missingId)),
        ),
      ).toBeUndefined()
    }
    const election = validatorElectionFromConfig(
      withParameters(mainnet.parameters.filter(parameter => [15, 34].includes(parameter.id))),
    )
    expect(election?.current.total).toBe(396)
    expect(election?.timing).toEqual(
      mainnet.parameters.find(parameter => parameter.id === 15)?.electionTiming,
    )
    expect(election?.previous).toBeUndefined()
    expect(election?.next).toBeUndefined()
    expect(election?.count).toBeUndefined()
    expect(election?.stake).toBeUndefined()
  })
})

test("election stages change at the configured window boundaries", () => {
  const election = validatorElectionFromConfig(mainnet)
  if (!election) throw new Error("Missing fixture election")
  const end = election.current.utimeUntil
  const open = end - election.timing.electionsStartBefore
  const close = end - election.timing.electionsEndBefore
  expect(validatorElectionStage(election, open - 1)).toBe("validation")
  expect(validatorElectionStage(election, open)).toBe("accepting_entries")
  expect(validatorElectionStage(election, close - 1)).toBe("accepting_entries")
  expect(validatorElectionStage(election, close)).toBe("finalizing")
  expect(validatorElectionStage(election, end - 1)).toBe("finalizing")
  expect(validatorElectionStage(election, end)).toBe("activation_overdue")
  const ready = {...election, next: {...election.current, utimeSince: end}}
  expect(validatorElectionStage(ready, close)).toBe("next_set_ready")
  expect(validatorElectionStage(ready, end)).toBe("activation_overdue")
})
