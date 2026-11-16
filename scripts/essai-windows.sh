#!/usr/bin/env bash
# Smoke test of a freshly built executable (run by the CI on the Windows runner):
# its version must be the delivery number.
set -euo pipefail
exe="$1"

out="$("$exe" --version | tr -d '\r')"
echo "$out"
test "$out" = "massicot ${MASSICOT_VERSION:?}"
