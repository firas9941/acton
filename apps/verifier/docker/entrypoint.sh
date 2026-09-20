#!/bin/sh
set -eu

config_path="${VERIFIER_CONFIG:-/etc/verifier/config.toml}"
mode="${VERIFIER_MODE:-serve}"

toml_escape() {
    printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

write_optional_string() {
    key="$1"
    value="$2"
    if [ -n "$value" ]; then
        printf '%s = "%s"\n' "$key" "$(toml_escape "$value")"
    fi
}

write_optional_int() {
    key="$1"
    value="$2"
    if [ -n "$value" ]; then
        printf '%s = %s\n' "$key" "$value"
    fi
}

fail() {
    echo "verifier-entrypoint: $*" >&2
    exit 1
}

validate_boolean() {
    name="$1"
    value="$2"
    case "$value" in
        true|false) ;;
        *) fail "$name must be true or false" ;;
    esac
}

url_contains_credentials() {
    case "$1" in
        https://*:*@*) return 0 ;;
        *) return 1 ;;
    esac
}

configure_git_auth() {
    auth_mode="${SOURCE_REPOSITORY_AUTH_MODE:-none}"
    repo_url="${SOURCE_REPOSITORY_URL:-}"
    ssh_key_file="${SOURCE_REPOSITORY_SSH_KEY_FILE:-}"

    case "$auth_mode" in
        none)
            if [ -n "$ssh_key_file" ]; then
                fail "SOURCE_REPOSITORY_SSH_KEY_FILE requires SOURCE_REPOSITORY_AUTH_MODE=ssh"
            fi
            if url_contains_credentials "$repo_url"; then
                fail "credentials in SOURCE_REPOSITORY_URL require SOURCE_REPOSITORY_AUTH_MODE=url"
            fi
            ;;
        url)
            if [ -n "$ssh_key_file" ]; then
                fail "SOURCE_REPOSITORY_AUTH_MODE=url cannot be combined with SOURCE_REPOSITORY_SSH_KEY_FILE"
            fi
            if [ -z "$repo_url" ]; then
                fail "SOURCE_REPOSITORY_AUTH_MODE=url requires SOURCE_REPOSITORY_URL"
            fi
            if ! url_contains_credentials "$repo_url"; then
                fail "SOURCE_REPOSITORY_AUTH_MODE=url requires credentials in an HTTPS SOURCE_REPOSITORY_URL"
            fi
            ;;
        ssh)
            if [ -z "$ssh_key_file" ]; then
                fail "SOURCE_REPOSITORY_AUTH_MODE=ssh requires SOURCE_REPOSITORY_SSH_KEY_FILE"
            fi
            if [ ! -r "$ssh_key_file" ]; then
                fail "SOURCE_REPOSITORY_SSH_KEY_FILE is not readable"
            fi
            if url_contains_credentials "$repo_url"; then
                fail "SOURCE_REPOSITORY_AUTH_MODE=ssh cannot be combined with credentials in SOURCE_REPOSITORY_URL"
            fi

            strict_host_key_checking="${SOURCE_REPOSITORY_SSH_STRICT_HOST_KEY_CHECKING:-accept-new}"
            case "$strict_host_key_checking" in
                yes|no|ask|accept-new) ;;
                *) fail "invalid SOURCE_REPOSITORY_SSH_STRICT_HOST_KEY_CHECKING value" ;;
            esac
            export GIT_SSH_COMMAND="ssh -i ${ssh_key_file} -o IdentitiesOnly=yes -o StrictHostKeyChecking=${strict_host_key_checking}"
            ;;
        github_app)
            if [ -n "$ssh_key_file" ]; then
                fail "SOURCE_REPOSITORY_AUTH_MODE=github_app cannot be combined with SOURCE_REPOSITORY_SSH_KEY_FILE"
            fi
            if [ -z "$repo_url" ]; then
                fail "SOURCE_REPOSITORY_AUTH_MODE=github_app requires SOURCE_REPOSITORY_URL"
            fi
            if url_contains_credentials "$repo_url"; then
                fail "SOURCE_REPOSITORY_AUTH_MODE=github_app requires a URL without credentials"
            fi
            case "$repo_url" in
                https://github.com/*) ;;
                *) fail "SOURCE_REPOSITORY_AUTH_MODE=github_app requires an HTTPS github.com URL" ;;
            esac

            github_app_id="${SOURCE_REPOSITORY_GITHUB_APP_ID:-}"
            github_installation_id="${SOURCE_REPOSITORY_GITHUB_APP_INSTALLATION_ID:-}"
            github_private_key_file="${SOURCE_REPOSITORY_GITHUB_APP_PRIVATE_KEY_FILE:-}"
            case "$github_app_id" in
                ''|*[!0-9]*) fail "SOURCE_REPOSITORY_GITHUB_APP_ID must be a positive integer" ;;
            esac
            case "$github_installation_id" in
                ''|*[!0-9]*) fail "SOURCE_REPOSITORY_GITHUB_APP_INSTALLATION_ID must be a positive integer" ;;
            esac
            if [ "$github_app_id" = "0" ]; then
                fail "SOURCE_REPOSITORY_GITHUB_APP_ID must be a positive integer"
            fi
            if [ "$github_installation_id" = "0" ]; then
                fail "SOURCE_REPOSITORY_GITHUB_APP_INSTALLATION_ID must be a positive integer"
            fi
            if [ -z "$github_private_key_file" ] || [ ! -r "$github_private_key_file" ]; then
                fail "SOURCE_REPOSITORY_GITHUB_APP_PRIVATE_KEY_FILE must be readable"
            fi

            export GIT_CONFIG_COUNT=3
            export GIT_CONFIG_KEY_0=credential.helper
            export GIT_CONFIG_VALUE_0=
            export GIT_CONFIG_KEY_1=credential.helper
            export GIT_CONFIG_VALUE_1="!/usr/local/bin/github-app-credential"
            export GIT_CONFIG_KEY_2=credential.useHttpPath
            export GIT_CONFIG_VALUE_2=true
            export GIT_TERMINAL_PROMPT=0
            ;;
        *)
            fail "SOURCE_REPOSITORY_AUTH_MODE must be one of: none, url, ssh, github_app"
            ;;
    esac
}

ensure_source_repository() {
    repo_path="${SOURCE_REPOSITORY_PATH:-}"
    repo_url="${SOURCE_REPOSITORY_URL:-}"
    branch="${SOURCE_REPOSITORY_BRANCH:-}"
    remote="${SOURCE_REPOSITORY_REMOTE:-origin}"

    if [ -z "$repo_path" ] || [ -z "$repo_url" ]; then
        return 0
    fi

    if [ -d "$repo_path/.git" ]; then
        if git -C "$repo_path" remote get-url "$remote" >/dev/null 2>&1; then
            git -C "$repo_path" remote set-url "$remote" "$repo_url"
        else
            git -C "$repo_path" remote add "$remote" "$repo_url"
        fi
        return 0
    fi

    if [ -e "$repo_path" ] && [ "$(find "$repo_path" -mindepth 1 -maxdepth 1 | wc -l | tr -d ' ')" != "0" ]; then
        echo "source repository path exists and is not an empty git repository: $repo_path" >&2
        exit 1
    fi

    mkdir -p "$(dirname "$repo_path")"
    git clone "$repo_url" "$repo_path"
    if [ -n "$branch" ]; then
        git -C "$repo_path" switch "$branch" 2>/dev/null || git -C "$repo_path" switch -c "$branch"
    fi
}

write_generated_config() {
    mkdir -p "$(dirname "$config_path")"

    {
        printf '[server]\n'
        printf 'bind_addr = "%s"\n' "$(toml_escape "${VERIFIER_BIND_ADDR:-0.0.0.0:3000}")"
        write_optional_string api_key "${VERIFIER_API_KEY:-}"
        printf 'read_only = %s\n' "${VERIFIER_READ_ONLY:-false}"
        printf '\n'

        printf '[logging]\n'
        printf 'level = "%s"\n\n' "$(toml_escape "${VERIFIER_LOG_LEVEL:-info}")"

        printf '[toncenter]\n'
        write_optional_string mainnet_base_url "${VERIFIER_TONCENTER_MAINNET_BASE_URL:-}"
        write_optional_string mainnet_api_key "${VERIFIER_TONCENTER_MAINNET_API_KEY:-}"
        write_optional_string testnet_base_url "${VERIFIER_TONCENTER_TESTNET_BASE_URL:-}"
        write_optional_string testnet_api_key "${VERIFIER_TONCENTER_TESTNET_API_KEY:-}"
        printf '\n'

        printf '[payment]\n'
        printf 'primary_network = "%s"\n' "$(toml_escape "${VERIFIER_PAYMENT_PRIMARY_NETWORK:-testnet}")"
        write_optional_string address "${VERIFIER_PAYMENT_ADDRESS:-}"
        write_optional_int min_amount_nano "${VERIFIER_PAYMENT_MIN_AMOUNT_NANO:-}"
        write_optional_string ledger_path "${VERIFIER_PAYMENT_LEDGER_PATH:-/var/lib/verifier/payment-ledger/payment-ledger.sqlite3}"
        printf '\n'

        printf '[source_repository]\n'
        write_optional_string path "${SOURCE_REPOSITORY_PATH:-/var/lib/verifier/source-repo}"
        write_optional_string remote "${SOURCE_REPOSITORY_REMOTE:-origin}"
        write_optional_string storage_root "${SOURCE_REPOSITORY_STORAGE_ROOT:-sources}"
        write_optional_string branch "${SOURCE_REPOSITORY_BRANCH:-main}"
        printf 'commit_enabled = %s\n' "${SOURCE_REPOSITORY_COMMIT_ENABLED:-true}"
        printf 'push_enabled = %s\n' "${SOURCE_REPOSITORY_PUSH_ENABLED:-true}"
        write_optional_string author_name "${SOURCE_REPOSITORY_AUTHOR_NAME:-ton-verifier}"
        write_optional_string author_email "${SOURCE_REPOSITORY_AUTHOR_EMAIL:-ton-verifier@example.invalid}"
        printf '\n'

        printf '[registry_index]\n'
        write_optional_string path "${VERIFIER_REGISTRY_INDEX_PATH:-/var/lib/verifier/registry-index/registry-index.sqlite3}"
        printf '\n'

        printf '[compiler]\n'
        write_optional_string node_bin "${VERIFIER_COMPILER_NODE_BIN:-node}"
        write_optional_string worker_path "${VERIFIER_COMPILER_WORKER_PATH:-/app/compiler-worker/compile.mjs}"
        write_optional_int timeout_ms "${VERIFIER_COMPILER_TIMEOUT_MS:-10000}"
        write_optional_int max_concurrent_compilations "${VERIFIER_COMPILER_MAX_CONCURRENT_COMPILATIONS:-1}"
        printf '\n'

        printf '[upload_limits]\n'
        write_optional_int max_request_bytes "${VERIFIER_UPLOAD_MAX_REQUEST_BYTES:-524288}"
    } > "$config_path"
}

validate_boolean SOURCE_REPOSITORY_COMMIT_ENABLED "${SOURCE_REPOSITORY_COMMIT_ENABLED:-true}"
validate_boolean SOURCE_REPOSITORY_PUSH_ENABLED "${SOURCE_REPOSITORY_PUSH_ENABLED:-true}"
case "$mode" in
    serve|init) ;;
    *) fail "VERIFIER_MODE must be one of: serve, init" ;;
esac
configure_git_auth
ensure_source_repository

if [ ! -f "$config_path" ] || [ "${VERIFIER_FORCE_GENERATE_CONFIG:-0}" = "1" ]; then
    write_generated_config
fi

case "$mode" in
    serve) exec "$@" ;;
    init) exec verifier-prepare-source-repository --push "$config_path" ;;
esac
