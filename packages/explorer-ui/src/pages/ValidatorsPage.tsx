import {Button} from "@acton/ui"
import {
  NetworkDashboardContent,
  NetworkDashboardSkeleton,
  type ElectionObservation,
  type NetworkView,
  type ValidatorSetObservation,
} from "@acton/localton-ui"
import {Clock3} from "lucide-react"
import {useEffect, useState, type FC} from "react"

import type {TonClient, ValidatorCycle} from "@acton/explorer-core/api/client"
import type {ValidatorSetConfiguration} from "@acton/explorer-core/api/config"
import {
  validatorElectionFromConfig,
  validatorElectionStage,
  type ValidatorElection,
} from "@acton/explorer-core/api/validatorElections"
import {ExplorerBreadcrumbs} from "@acton/explorer-core/components/ExplorerBreadcrumbs"

import styles from "./ValidatorsPage.module.css"

interface ValidatorsPageProps {
  readonly client: TonClient
}

type ValidatorsLoadState =
  | {readonly status: "loading"}
  | {
      readonly status: "success"
      readonly cycle?: ValidatorCycle
      readonly election?: ValidatorElection
    }
  | {readonly status: "error"; readonly message: string}

const CLOCK_REFRESH_MS = 1000
const CONFIG_REFRESH_MS = 30_000

/** Connects Actonscan config data to Localton's shared validator dashboard. */
export const ValidatorsPage: FC<ValidatorsPageProps> = ({client}) => {
  const [loadState, setLoadState] = useState<ValidatorsLoadState>({status: "loading"})
  const [now, setNow] = useState(() => Math.floor(Date.now() / 1000))
  const [retry, setRetry] = useState(0)

  useEffect(() => {
    const interval = globalThis.setInterval(
      () => setNow(Math.floor(Date.now() / 1000)),
      CLOCK_REFRESH_MS,
    )
    return () => globalThis.clearInterval(interval)
  }, [])

  // biome-ignore lint/correctness/useExhaustiveDependencies: Retry restarts the config refresh cycle after a failed request.
  useEffect(() => {
    let active = true
    setLoadState({status: "loading"})

    let timeout: ReturnType<typeof globalThis.setTimeout> | undefined
    const loadConfig = () => {
      void client
        .getNetworkConfig()
        .then(async config => {
          const election = validatorElectionFromConfig(config)
          const cycle = await loadValidatorCycle(client, election)
          if (!active) return
          setLoadState({status: "success", cycle, election})
        })
        .catch((error: unknown) => {
          if (!active) return
          setLoadState({
            status: "error",
            message: error instanceof Error ? error.message : String(error),
          })
        })
        .finally(() => {
          if (active) timeout = globalThis.setTimeout(loadConfig, CONFIG_REFRESH_MS)
        })
    }
    loadConfig()

    return () => {
      active = false
      globalThis.clearTimeout(timeout)
    }
  }, [client, retry])

  return (
    <section className={styles.container}>
      <ExplorerBreadcrumbs items={[{label: "Elections"}]} />

      <ValidatorsPageContent loadState={loadState} now={now} onRetry={() => setRetry(retry + 1)} />
    </section>
  )
}

async function loadValidatorCycle(
  client: TonClient,
  election: ValidatorElection | undefined,
): Promise<ValidatorCycle | undefined> {
  if (!election) return undefined

  try {
    return await client.getValidatorCycle(election.current.utimeSince)
  } catch {
    return undefined
  }
}

function ValidatorsPageContent({
  loadState,
  now,
  onRetry,
}: {
  readonly loadState: ValidatorsLoadState
  readonly now: number
  readonly onRetry: () => void
}) {
  if (loadState.status === "loading") {
    return (
      <NetworkDashboardSkeleton
        showValidatorPerformance={false}
        showValidatorProduction={false}
        view="validators"
      />
    )
  }

  if (loadState.status === "error") {
    return (
      <section className={styles.error} role="alert">
        <h2>Election data is unavailable</h2>
        <p>{loadState.message}</p>
        <Button size="sm" onClick={onRetry}>
          Retry
        </Button>
      </section>
    )
  }

  if (!loadState.election) {
    return (
      <div className={styles.notice}>
        <Clock3 size={16} aria-hidden="true" />
        <span>Election data is not available from the current network configuration</span>
      </div>
    )
  }

  return (
    <NetworkDashboardContent
      network={toNetworkView(loadState.election, loadState.cycle, now)}
      now={now}
      showValidatorPerformance={false}
      showValidatorProduction={false}
      tps={undefined}
      view="validators"
    />
  )
}

function toNetworkView(
  election: ValidatorElection,
  cycle: ValidatorCycle | undefined,
  now: number,
): NetworkView {
  return {
    protocol_version: 1,
    network_id: "actonscan",
    generated_at: now,
    chain: null,
    shards: [],
    election: toElectionObservation(election, cycle, now),
    totals: {
      observers: 0,
      online_observers: 0,
      nodes: 0,
      online_nodes: 0,
      synchronized_nodes: 0,
      catching_up_nodes: 0,
      configured_validators: election.current.total,
      active_validators: election.current.total,
      full_nodes: 0,
      masterchain_blocks: 0,
      shard_blocks: 0,
    },
    observers: [],
    nodes: [],
    production: [],
  }
}

function toElectionObservation(
  election: ValidatorElection,
  cycle: ValidatorCycle | undefined,
  now: number,
): ElectionObservation {
  return {
    stage: validatorElectionStage(election, now),
    elections_open_at: election.current.utimeUntil - election.timing.electionsStartBefore,
    elections_close_at: election.current.utimeUntil - election.timing.electionsEndBefore,
    validators_elected_for: election.timing.validatorsElectedFor,
    stake_held_for: election.timing.stakeHeldFor,
    min_stake_nano: election.stake ? election.stake.minStake.toString() : "0",
    max_stake_nano: election.stake ? election.stake.maxStake.toString() : "0",
    min_validators: election.count ? election.count.minValidators : 0,
    max_validators: election.count ? election.count.maxValidators : election.current.total,
    max_main_validators: election.count ? election.count.maxMainValidators : election.current.main,
    previous: election.previous ? toValidatorSetObservation(election.previous) : null,
    current: toValidatorSetObservation(election.current, cycle),
    next: election.next ? toValidatorSetObservation(election.next) : null,
  }
}

function toValidatorSetObservation(
  set: ValidatorSetConfiguration,
  cycle?: ValidatorCycle,
): ValidatorSetObservation {
  return {
    round_id: set.utimeSince,
    validation_started_at: set.utimeSince,
    validation_ended_at: set.utimeUntil,
    validators: set.total,
    main_validators: set.main,
    total_weight: (
      set.totalWeight ?? set.validators.reduce((total, validator) => total + validator.weight, 0n)
    ).toString(),
    ...(cycle
      ? {
          stake: {
            total_nano: cycle.total_stake,
            minimum_nano: cycle.min_stake,
            maximum_nano: cycle.max_stake,
          },
        }
      : {}),
    members: set.validators.map(validator => ({
      public_key: validator.publicKey,
      adnl_address: validator.adnlAddress ?? null,
      weight: validator.weight.toString(),
      efficiency: null,
    })),
  }
}
