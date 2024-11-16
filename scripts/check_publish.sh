#!/bin/bash
#
# Checks that the current Cargo.toml package version
# matches the git tag. If the versions match, publish the crate
# to crates.io.
#
set -ex

CRATE="$(cargo read-manifest | jq '.name' -r)"
VERSION="$(cargo read-manifest | jq '.version' -r)"

# Test crate isn't published already
if test $(cargo search "$CRATE" | grep "$CRATE =" | awk '{print $3}') = "\"$VERSION\""; then
    echo "Crate has been published already, aborting"
    exit 1
else
    echo "Crate has not been published"
fi

if ! test "v$VERSION" = $(git describe --exact-match --tags); then
    echo "Latest git tag does not match crate version, skipping"
    exit 0
else
    echo "All checks passed, publishing crate"
fi

if [[ ! -z "${CRATES_IO_TOKEN}" ]]; then
    cargo login "${CRATES_IO_TOKEN}"
fi

cargo publish
