# Docker Deployment

Production deployments should pull the CI-built image:

```bash
docker pull ghcr.io/ton-blockchain/verifier:latest
```

Local development can build the image:

```bash
docker build -f apps/verifier/Dockerfile -t ton-verifier:local .
```

Initialize an empty source repository before deploying the verifier:

```bash
git clone <source-repository-url> source-repo
# Set source_repository.path = "source-repo" in config.toml.
apps/verifier/scripts/prepare-source-repository.sh config.toml
git -C source-repo push origin HEAD:main
```

The preparation script creates the required root commit with the source-storage
Git attributes. The verifier refuses to start when this commit is missing or
the current `.gitattributes` no longer contains
`<source_repository.storage_root>/** -text`.

Or use the local build override:

```bash
docker compose \
  -f apps/verifier/docker-compose.yml \
  -f apps/verifier/docker-compose.local.yml \
  up -d --build
```

Run with generated config:

```bash
docker run --rm -p 3000:3000 \
  -e VERIFIER_TONCENTER_MAINNET_BASE_URL=https://toncenter.com \
  -e VERIFIER_TONCENTER_TESTNET_BASE_URL=https://testnet.toncenter.com \
  -e VERIFIER_PAYMENT_PRIMARY_NETWORK=testnet \
  -e 'VERIFIER_PAYMENT_ADDRESS=0:<64-hex-character-wallet-address>' \
  -e VERIFIER_PAYMENT_MIN_AMOUNT_NANO=500000000 \
  -e SOURCE_REPOSITORY_URL=https://github.com/i582/test-verify-repo \
  -e SOURCE_REPOSITORY_AUTH_MODE=none \
  -e SOURCE_REPOSITORY_STORAGE_ROOT=sources \
  -e SOURCE_REPOSITORY_BRANCH=main \
  -e SOURCE_REPOSITORY_COMMIT_ENABLED=true \
  -e SOURCE_REPOSITORY_PUSH_ENABLED=true \
  -v verifier-source-repo:/var/lib/verifier/source-repo \
  -v verifier-registry-index:/var/lib/verifier/registry-index \
  -v verifier-payment-ledger:/var/lib/verifier/payment-ledger \
  ghcr.io/ton-blockchain/verifier:latest
```

The verifier accepts payments on TON mainnet or testnet, selected with
`VERIFIER_PAYMENT_PRIMARY_NETWORK`. The payment address must use raw basechain
form. At startup, the service rebuilds the payment ledger from the selected
network's wallet history and reports `503` until the scan is complete. This
example sets the minimum payment to `0.5 GRAM`.

Set `VERIFIER_READ_ONLY=true` to reject tickets and submissions for new code
hashes while keeping verified source and metadata lookups available.

Or mount a full TOML config:

```bash
docker run --rm -p 3000:3000 \
  -e VERIFIER_CONFIG=/etc/verifier/config.toml \
  -v ./config.toml:/etc/verifier/config.toml:ro \
  -v verifier-source-repo:/var/lib/verifier/source-repo \
  -v verifier-registry-index:/var/lib/verifier/registry-index \
  -v verifier-payment-ledger:/var/lib/verifier/payment-ledger \
  ghcr.io/ton-blockchain/verifier:latest
```

For SSH Git remotes, mount a deploy key and pass:

```bash
-e SOURCE_REPOSITORY_AUTH_MODE=ssh
-e SOURCE_REPOSITORY_URL=git@github.com:i582/test-verify-repo.git
-e SOURCE_REPOSITORY_SSH_KEY_FILE=/run/secrets/source_repo_key
-v ./source_repo_key:/run/secrets/source_repo_key:ro
```

For an HTTPS remote with credentials embedded in its URL, pass:

```bash
-e SOURCE_REPOSITORY_AUTH_MODE=url
-e SOURCE_REPOSITORY_URL=https://x-access-token:<token>@github.com/owner/repository.git
```

The `url` mode stores the credential in the checkout's Git remote configuration.
Avoid it for short-lived credentials, and make sure deployment output and error
logs do not expose the URL. Use `none` only for repositories that need no Git
credentials.

For a GitHub App installation, mount its private key and pass the App and
installation IDs explicitly:

```bash
-e SOURCE_REPOSITORY_AUTH_MODE=github_app
-e SOURCE_REPOSITORY_URL=https://github.com/owner/repository.git
-e SOURCE_REPOSITORY_GITHUB_APP_ID=<app-id>
-e SOURCE_REPOSITORY_GITHUB_APP_INSTALLATION_ID=<installation-id>
-e SOURCE_REPOSITORY_GITHUB_APP_PRIVATE_KEY_FILE=/run/secrets/github-app.pem
-v ./github-app.pem:/run/secrets/github-app.pem:ro
```

The verifier requests the installation token directly from the configured
installation ID whenever Git requests credentials. It does not store the token
in the remote URL or look up the installation by repository.

The image contains:

- `verifier` Rust backend
- Node.js runtime
- `compiler-worker/compile.mjs`
- Static NPM compiler packages for supported FunC, Tact, and Tolk versions
- Git and OpenSSH client for source storage commit/push
