# TON P2P block client

This crate discovers peers over ADNL UDP and DHT, then downloads blocks and
proofs through RLDP2.

`Client` owns the UDP transport and an exclusively locked block cache.
`next_masterchain` downloads one block, saves its proof, and advances the download
checkpoint. It prefetches one successor while the caller processes saved data.
`download_shards` loads exact block IDs supplied by the caller, using the cache
before querying peers. Discovery refreshes run in the background after startup.

The client checks file hashes, root hashes, block headers, masterchain predecessor
links, and proof roots. It does not check validator signatures or execute state
transitions. The configured starting block is a trusted anchor.

Consumers own their processing checkpoints. `ton-indexer-p2p` adapts this client
to complete indexing batches. `apps/ton-sync` provides the CLI.
