# P2P source for TON indexers

Available since trunk.

`P2pBlockSource` wraps a `ton-p2p::Client` and implements
`ton-indexer-core::BlockSource`. It uses the shared canonical traversal to collect
all shard predecessors between consecutive masterchain blocks, including splits
and merges. It returns a batch only when all required blocks are available.

The consumer owns its indexing checkpoint. Downloaded files can be ahead of that
checkpoint without changing which batch the consumer receives next.

The optional `start::use_latest_block` helper uses LiteServer to choose a recent starting
ID for a new cache. It checks the network zerostate and preserves existing cache
checkpoints. An indexer with an existing processing checkpoint must start from
that ID instead. All block and proof downloads use P2P.

Block integrity checks do not validate validator signatures or state transitions.
