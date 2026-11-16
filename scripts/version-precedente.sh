#!/usr/bin/env bash
# Fetches the latest published release for the update test of the installer (run by the CI on
# Windows): its zip, unzipped (a copy installed by hand), and its installer, or, for a release
# that has none, an installer built from its zip with the current script.
# Writes precedente/ and PRECEDENTE=<version> to $GITHUB_ENV.
set -euo pipefail
dir=precedente
rm -rf "$dir" && mkdir -p "$dir"
tag="$(gh release view --json tagName --jq .tagName)"
v="${tag#v}"
gh release download "$tag" --pattern "Massicot.$v.zip" --dir "$dir"
7z x -bd -o"$dir/zip" "$dir/Massicot.$v.zip" > /dev/null
if gh release download "$tag" --pattern "Massicot.$v.exe" --dir "$dir" 2> /dev/null; then
  echo "Release $tag : son installateur est repris tel quel."
else
  echo "Release $tag sans installateur : il est construit depuis son zip."
  bash scripts/installateur.sh "$v" "$dir/zip/Massicot.$v" "$dir"
fi
echo "PRECEDENTE=$v" >> "${GITHUB_ENV:-/dev/null}"
