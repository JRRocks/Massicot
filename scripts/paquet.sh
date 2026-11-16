#!/usr/bin/env bash
# Assembles the Windows package Massicot.<version>.zip from a release build (run by the CI).
# Usage: paquet.sh <version> <massicot.exe> <models dir>
set -euo pipefail
version="$1" exe="$2" models="$3"
dir="paquet/Massicot.$version"
rm -rf paquet && mkdir -p "$dir/modeles" "$dir/licences"
cp "$exe" LICENSE-MIT LICENSE-APACHE NOTICE ATTRIBUTION.md "$dir/"
cp README.md "$dir/LISEZMOI.md"
cp "$models"/text-detection.onnx "$models"/text-recognition.onnx "$dir/modeles/"
cp assets/fonts/OFL-*.txt licences/*.txt "$dir/licences/"
cp vendor/hayro-interpret/assets/LICENSE_FOXIT vendor/hayro-interpret/assets/CGATS_LICENSE.txt "$dir/licences/"
cp vendor/hayro/LICENSE-MIT "$dir/licences/hayro-LICENSE-MIT.txt"
cp vendor/vello_cpu/LICENSE-MIT "$dir/licences/vello-LICENSE-MIT.txt"
cp vendor/lopdf/LICENSE "$dir/licences/lopdf-LICENSE.txt"
python3 scripts/dependances.py "$dir/licences/dependances.md" || python scripts/dependances.py "$dir/licences/dependances.md"
(cd paquet && 7z a -tzip -bd "../Massicot.$version.zip" "Massicot.$version" > /dev/null)
