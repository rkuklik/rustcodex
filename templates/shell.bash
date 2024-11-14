#!/usr/bin/env bash
# __SOURCE__
set -eumo pipefail
payload='__PAYLOAD__'
file=$(mktemp)
chmod 700 "$file"
echo "$payload" | base64 -di | gzip -dc >>"$file"
exec -a binary "$file" $@
