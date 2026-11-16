#!/usr/bin/env bash
# Downloads the two ocrs OCR models (CC-BY-SA-4.0, see ATTRIBUTION.md) into DIR (default:
# modeles/) and checks their SHA-256. Files already present and intact are kept.
set -euo pipefail
dir="${1:-modeles}"
base="https://ocrs-models.s3-accelerate.amazonaws.com"
mkdir -p "$dir"
while read -r file sum; do
  if ! echo "$sum  $dir/$file" | sha256sum -c --status 2>/dev/null; then
    curl -fsSL --retry 4 --retry-delay 5 -o "$dir/$file" "$base/$file"
  fi
  echo "$sum  $dir/$file" | sha256sum -c -
done <<'LIST'
text-detection.onnx a917b23dbd9524b465df7e922641b3ff2981623df4ded5a0234004ef2fee7cfe
text-recognition.onnx 86c145c2edb96c8caed5b1ebb8f44d706408922211451c309c625157dd6061c5
LIST
