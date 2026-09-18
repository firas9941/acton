import type {TonClient} from "@acton/explorer-core/api/client"
import {AbiCatalog, AbiDetails, useAbiDetails} from "@acton/explorer-core/components/AbiCatalog"
import {ExplorerBreadcrumbs} from "@acton/explorer-core/components/ExplorerBreadcrumbs"
import {SourceCatalog} from "@acton/explorer-core/components/SourceCatalog"
import {useExplorerRoutePaths} from "@acton/explorer-core/hooks/useExplorerRoutePaths"
import {useNetworkInfo} from "@acton/explorer-core/hooks/useNetworkInfo"
import {AddressConverterPage as Converter} from "@acton/explorer-core/pages/AddressConverterPage"
import {CellInspectorPage} from "@acton/explorer-core/pages/CellInspectorPage"
import {EmulatePage} from "@acton/explorer-core/pages/EmulatePage"
import {useNavigate, useParams, useSearchParams} from "react-router"

import styles from "./explorer-pages.module.css"

/** Adds Actonscan's page layout and navigation to the shared ABI catalog. */
export function AbiCatalogPage() {
  return (
    <section className={styles.container}>
      <ExplorerBreadcrumbs items={[{label: "ABI catalog"}]} />
      <header className={styles.hero}>
        <h1 className={styles.title}>ABI catalog</h1>
      </header>
      <AbiCatalog />
    </section>
  )
}

/** Resolves the route slug and breadcrumb title for the shared ABI details view. */
export function AbiDetailsPage() {
  const {slug = ""} = useParams()
  const routes = useExplorerRoutePaths()
  const state = useAbiDetails(slug)
  let title = "ABI"
  if (state.status === "ready") {
    title = state.title
  } else if (state.status === "not-found") {
    title = "ABI not found"
  }

  return (
    <section className={styles.container}>
      <ExplorerBreadcrumbs
        ariaLabel="Contract breadcrumb"
        rootLabel="Contracts"
        rootPath={routes.contractsPath ?? routes.rootPath}
        items={[{label: "ABI catalog", path: routes.abiPath}, {label: title}]}
      />
      <header className={styles.hero}>
        <h1 className={styles.title}>{title}</h1>
      </header>
      <AbiDetails state={state} />
    </section>
  )
}

/** Supplies the active network client and Actonscan navigation to the source catalog. */
export function SourceCatalogPage({client}: {readonly client: TonClient}) {
  return (
    <section className={styles.container}>
      <ExplorerBreadcrumbs items={[{label: "Source artifacts"}]} />
      <header className={styles.hero}>
        <h1 className={styles.title}>Source artifacts</h1>
      </header>
      <SourceCatalog client={client} />
    </section>
  )
}

function ExplorerToolHeader({title}: {readonly title: string}) {
  return (
    <section className={`${styles.container} ${styles.toolHeader}`}>
      <ExplorerBreadcrumbs items={[{label: title}]} />
    </section>
  )
}

/** Adds host navigation to the shared inspector, which owns its content layout. */
export function CellInspectorExplorerPage() {
  return (
    <>
      <ExplorerToolHeader title="Cell inspector" />
      <CellInspectorPage />
    </>
  )
}

/** Connects the shared emulator to Actonscan's emulation-sharing endpoint. */
export function EmulateExplorerPage({client}: {readonly client: TonClient}) {
  return (
    <>
      <ExplorerToolHeader title="Emulate" />
      <EmulatePage client={client} shareApiPath="/api/emulations" />
    </>
  )
}

/** Opens converted addresses using the API client and URL for their selected public network. */
export function AddressConverterPage() {
  const {network} = useNetworkInfo()
  const routes = useExplorerRoutePaths()
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()

  return (
    <>
      <ExplorerToolHeader title="Address converter" />
      <Converter
        onOpenAddress={(address, testOnly) => {
          const search = new URLSearchParams(searchParams)
          search.delete("address")
          let targetNetwork = network.id
          if (testOnly !== undefined) targetNetwork = testOnly ? "testnet" : "mainnet"
          search.set("network", targetNetwork)
          const pathname = routes.addressPath(address)
          // Switching networks must reinitialize the app's API client from the destination URL.
          if (targetNetwork !== network.id) {
            globalThis.location.assign(`${pathname}?${search}`)
            return
          }
          void navigate({pathname, search: search.toString()})
        }}
      />
    </>
  )
}
