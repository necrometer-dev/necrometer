#!/bin/sh
# Entrypoint (BusyBox ash). Zero required env vars:
#   PUID=99 PGID=100 UMASK=022 by default (Unraid nobody:users).
#
# Apply UMASK, exec daemon as PUID:PGID via su-exec.

set -eu

PUID=${PUID:-99}
PGID=${PGID:-100}
UMASK=${UMASK:-022}

umask "$UMASK"

exec su-exec "$PUID:$PGID" /usr/local/bin/necrometer "$@"
