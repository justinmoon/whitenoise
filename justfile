shell:
    nix shell nixpkgs#nostr-rs-relay

ios:
    TAURI_DEV_PORT=1421 bun tauri ios dev -- "iPhone 15 Pro"

desktop:
    bun tauri dev

run-blossom:
    BLOSSOM_CONFIG=test/blossom-server-ts-config.yml npx blossom-server-ts

run-relay:
    nostr-rs-relay --config test/nostr-rs-relay-config.toml
