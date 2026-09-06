import type {
  ElectionTimingConfiguration,
  NetworkConfig,
  ValidatorCountConfiguration,
  ValidatorSetConfiguration,
  ValidatorStakeConfiguration,
} from "./config"

export interface ValidatorElection {
  readonly timing: ElectionTimingConfiguration
  readonly count?: ValidatorCountConfiguration
  readonly stake?: ValidatorStakeConfiguration
  readonly previous?: ValidatorSetConfiguration
  readonly current: ValidatorSetConfiguration
  readonly next?: ValidatorSetConfiguration
}

/** Builds the election view using only the network configuration. */
export function validatorElectionFromConfig(config: NetworkConfig): ValidatorElection | undefined {
  const parameters = new Map(config.parameters.map(parameter => [parameter.id, parameter]))
  const timing = parameters.get(15)?.electionTiming
  const current = parameters.get(35)?.validatorSet ?? parameters.get(34)?.validatorSet
  if (!timing || !current) return undefined

  return {
    timing,
    count: parameters.get(16)?.validatorCount,
    stake: parameters.get(17)?.validatorStake,
    previous: parameters.get(33)?.validatorSet ?? parameters.get(32)?.validatorSet,
    current,
    next: parameters.get(37)?.validatorSet ?? parameters.get(36)?.validatorSet,
  }
}

/** These stages describe configured windows; they do not inspect Elector contract state. */
export function validatorElectionStage(election: ValidatorElection, now: number) {
  if (now >= election.current.utimeUntil) return "activation_overdue"
  if (election.next) return "next_set_ready"
  if (now < election.current.utimeUntil - election.timing.electionsStartBefore) {
    return "validation"
  }
  if (now < election.current.utimeUntil - election.timing.electionsEndBefore) {
    return "accepting_entries"
  }
  return "finalizing"
}
