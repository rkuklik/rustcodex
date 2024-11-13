#!/usr/bin/env bash
set -eumo pipefail
# __SOURCE__
payload='__PAYLOAD__'
file=$(mktemp)
chmod 700 "$file"
echo "$payload" | base64 -di | gzip -dc >>"$file"
exec "$file" $@
