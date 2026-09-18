// Vite's base types come from tsconfig; reject undeclared keys instead of falling back to any.
interface ViteTypeOptions {
  readonly strictImportMetaEnv: unknown
}

interface ImportMetaEnv {
  readonly VITE_ACTONSCAN_BACKEND_URL?: string
  readonly VITE_EXPLORER_TONCENTER_API_KEY?: string
  readonly VITE_EXPLORER_TONCENTER_API_V2_URL?: string
  readonly VITE_EXPLORER_TONCENTER_API_V3_URL?: string
  readonly VITE_EXPLORER_MAINNET_TONCENTER_API_KEY?: string
  readonly VITE_EXPLORER_MAINNET_TONCENTER_API_V2_URL?: string
  readonly VITE_EXPLORER_MAINNET_TONCENTER_API_V3_URL?: string
  readonly VITE_EXPLORER_TESTNET_TONCENTER_API_KEY?: string
  readonly VITE_EXPLORER_TESTNET_TONCENTER_API_V2_URL?: string
  readonly VITE_EXPLORER_TESTNET_TONCENTER_API_V3_URL?: string
}
