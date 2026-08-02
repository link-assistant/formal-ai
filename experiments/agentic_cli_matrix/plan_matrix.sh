#!/usr/bin/env bash
# Render the CI/local execution plan from the seed registry plus clients.lock.

source "$(cd "$(dirname "$0")" && pwd)/lib.sh"
set -euo pipefail

base="${MATRIX_PORT_BASE:-8900}"
stride="${MATRIX_PORT_STRIDE:-60}"
plan='[]'
index=0
while read -r client installer _spec; do
  [ -n "$client" ] || continue
  surface="$(matrix_client_field "$client" verification.surface)"
  display=false
  [ "$surface" = gui ] && display=true
  user_namespaces="$(matrix_client_field "$client" verification.sandbox_user_namespaces)"
  entry="$(jq -nc \
    --arg client "$client" \
    --arg installer "$installer" \
    --argjson base_port "$((base + index * stride))" \
    --argjson display "$display" \
    --argjson user_namespaces "$user_namespaces" \
    '{client: $client, installer: $installer, base_port: $base_port,
      display: $display, user_namespaces: $user_namespaces}')"
  plan="$(jq -nc --argjson plan "$plan" --argjson entry "$entry" '$plan + [$entry]')"
  index=$((index + 1))
done < <(awk '!/^#/ && NF >= 3 { print $1, $2, $3 }' "$LOCKFILE")

printf '%s\n' "$plan"
