#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: $0 <app-name> <target-triple> <version>"
  exit 1
fi

APP_NAME="$1"
TARGET="$2"
VERSION="$3"

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
STAGE_DIR="${DIST_DIR}/${APP_NAME}-${VERSION}-${TARGET}"
TARGET_ROOT="${CARGO_TARGET_DIR:-${ROOT_DIR}/target}"
BIN_PATH="${TARGET_ROOT}/${TARGET}/release/${APP_NAME}"
if [[ ! -f "${BIN_PATH}" ]]; then
  BIN_PATH="${TARGET_ROOT}/release/${APP_NAME}"
fi
ARCHIVE_NAME="${APP_NAME}-${VERSION}-${TARGET}.tar.gz"
ARCHIVE_PATH="${DIST_DIR}/${ARCHIVE_NAME}"
CHECKSUM_PATH="${DIST_DIR}/${ARCHIVE_NAME}.sha256"

rm -rf "${STAGE_DIR}"
mkdir -p "${STAGE_DIR}"

cp "${BIN_PATH}" "${STAGE_DIR}/${APP_NAME}"
cp "${ROOT_DIR}/README.md" "${STAGE_DIR}/README.md"
cp -R "${ROOT_DIR}/migrations" "${STAGE_DIR}/migrations"

tar -C "${DIST_DIR}" -czf "${ARCHIVE_PATH}" "$(basename "${STAGE_DIR}")"
shasum -a 256 "${ARCHIVE_PATH}" > "${CHECKSUM_PATH}"

echo "packaged ${ARCHIVE_PATH}"
