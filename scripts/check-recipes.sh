#!/usr/bin/env sh
set -eu

manifests='
recipes/auto-core-citizens/Cargo.toml
recipes/00-overview/modeled-work-order/Cargo.toml
recipes/01-lanes/read-lane/Cargo.toml
recipes/01-lanes/info-lane/Cargo.toml
recipes/01-lanes/parts-lane/Cargo.toml
recipes/01-lanes/service-lane/Cargo.toml
recipes/01-lanes/order-lane/Cargo.toml
recipes/01-lanes/flash-lane/Cargo.toml
recipes/02-sites/xentry-site/Cargo.toml
recipes/02-sites/ista-site/Cargo.toml
recipes/02-sites/vida-site/Cargo.toml
recipes/02-sites/odis-site/Cargo.toml
recipes/02-sites/esitronic-site/Cargo.toml
recipes/02-sites/haynespro-site/Cargo.toml
recipes/02-sites/biluppgifter-se-site/Cargo.toml
recipes/02-sites/mekonomen-pro-site/Cargo.toml
recipes/02-sites/autotuner-site/Cargo.toml
'

for manifest in $manifests; do
  cargo run --quiet --manifest-path "$manifest"
done

printf 'check-recipes: OK (%s recipe binary manifests)\n' "$(printf '%s\n' "$manifests" | sed '/^$/d' | wc -l)"
