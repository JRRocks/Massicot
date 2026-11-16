#!/usr/bin/env bash
# Builds the Windows installer Massicot.<version>.exe with Inno Setup (run by the CI on Windows,
# where Inno Setup is installed on the runner; installed with Chocolatey if it is not).
# Usage: installateur.sh <version> <package folder> <output folder>
set -euo pipefail
version="$1" source="$2" sortie="$3"

trouver_iscc() {
  for c in "/c/Program Files (x86)/Inno Setup 6/ISCC.exe" "/c/Program Files/Inno Setup 7/ISCC.exe" \
           "/c/Program Files (x86)/Inno Setup 7/ISCC.exe" "/c/Program Files/Inno Setup 6/ISCC.exe"; do
    if [ -f "$c" ]; then echo "$c"; return; fi
  done
  command -v iscc || command -v ISCC || true
}

iscc="$(trouver_iscc)"
if [ -z "$iscc" ]; then
  echo "Inno Setup absent : installation par Chocolatey."
  choco install innosetup -y --no-progress > /dev/null
  iscc="$(trouver_iscc)"
fi
test -n "$iscc"
mkdir -p "$sortie"
# Git Bash would turn the /D... options into paths: no conversion for this call.
MSYS2_ARG_CONV_EXCL='*' "$iscc" /Qp "/DVersion=$version" "/DSource=$(cygpath -wa "$source")" \
  "/DSortie=$(cygpath -wa "$sortie")" "$(cygpath -wa installateur/massicot.iss)"
ls -l "$sortie/Massicot.$version.exe"
