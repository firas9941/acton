# TON P2P block client

This crate discovers peers over ADNL UDP and DHT, then downloads blocks and
proofs through RLDP2.

`Client::message_sender()` returns a cloneable handle for external-message
broadcasts. Pass the original signed BoC to `ExternalMessage::new`, then call
`MessageSender::send`. The handle shares the UDP transport and caches discovered
peers for each destination workchain. It can send while the client downloads
blocks, and keeps the transport alive after the client is dropped.

Messages use the public masterchain or basechain overlay. Larger messages use
FEC broadcasts. Validation checks the BoC and message envelope, not contract
execution, signatures, expiry, or wallet sequence numbers. Submission queues
packets to discovered peers without a delivery receipt. Observe the transaction
to confirm inclusion. Peers can discard repeated messages.

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
