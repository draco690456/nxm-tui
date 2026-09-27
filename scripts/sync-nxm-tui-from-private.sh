#!/usr/bin/env bash
#
# sync-nxm-tui-from-private.sh — align the PUBLIC nxm-tui mirror from the
# PRIVATE source of truth.
#
# Same policy as the engine-mlx sync: public is ALWAYS just a sanitized copy of
# private. This script enforces the one-way copy AND refuses to publish if any
# forbidden term slips through.
#
# Direction: PRIVATE  ->  PUBLIC   (never the other way)
#
# Usage:
#   ./sync-nxm-tui-from-private.sh            # dry-run: show what would change
#   ./sync-nxm-tui-from-private.sh --apply    # actually sync
#
# NOTE (Tappa 3 MIGRATION-PLAN): this script is a CANDIDATE for consolidation
# into nxm-sync (ADR-005). When nxm-sync is implemented, per-product sync scripts
# like this should be removed and replaced by a single unified publisher.
#
set -euo pipefail

# ── Paths ────────────────────────────────────────────────────────────────────
PRIVATE_SRC="${PRIVATE_SRC:-$HOME/Projects/nxm-private/nxm-tui}"
PUBLIC_DST="${PUBLIC_DST:-$HOME/Projects/nxm-public/nxm-tui}"

# ── Forbidden terms — the public mirror must NEVER contain these ─────────────
# (internal names, proprietary tech, internal infra references, local paths)
FORBIDDEN='nexum|draco\.granata|Projects_Tmp|gitlab|nxm-ai|NXM_PAT|turboquant|TurboQuant|pflash|PFlash|kv_spill|nxm-memory|wayfinder|Danieles-MacBook|/Users/'

# ── What never gets copied ───────────────────────────────────────────────────
EXCLUDES=(
  --exclude='.git'          # public keeps its OWN clean history
  --exclude='target'        # Rust build artifacts (often 1 GB+)
  --exclude='.poolside'     # Poolside AI agent config
  --exclude='.cargo/config.toml'  # local-only Cargo config (git-ignored)
  --exclude='.cargo/.crates.toml'
  --exclude='*.log'
)

log()  { printf '\033[0;36m[sync]\033[0m %s\n' "$*"; }
err()  { printf '\033[0;31m[sync] ERROR:\033[0m %s\n' "$*" >&2; }

# ── Preconditions ────────────────────────────────────────────────────────────
[[ -d "$PRIVATE_SRC" ]] || { err "private source not found: $PRIVATE_SRC"; exit 1; }

APPLY=0
[[ "${1:-}" == "--apply" ]] && APPLY=1

# ── Guard: scan the PRIVATE source for forbidden terms before copying ────────
# We only scan text sources that would be published (never target/.git).
log "scanning private source for forbidden terms..."
if hits=$(grep -rinE "$FORBIDDEN" "$PRIVATE_SRC" \
            --include='*.rs' --include='*.toml' --include='*.md' \
            --include='*.sh' --include='*.yml' --include='*.yaml' \
            --include='*.json' --include='*.lock' \
            2>/dev/null | grep -v '/target/' | grep -v '/.git/' \
                        | grep -v '/.cargo/config.toml'); then
  err "forbidden terms found in private source — refusing to sync:"
  echo "$hits" | head -30 >&2
  err "sanitize the private source first, then re-run."
  exit 2
fi
log "clean — no forbidden terms."

# ── Sync ─────────────────────────────────────────────────────────────────────
mkdir -p "$PUBLIC_DST"
RSYNC_FLAGS=(-a --delete "${EXCLUDES[@]}")
if [[ "$APPLY" -eq 0 ]]; then
  RSYNC_FLAGS+=(--dry-run)
  log "DRY RUN (no changes written). Re-run with --apply to sync."
fi

log "private -> public"
log "  from: $PRIVATE_SRC/"
log "  to:   $PUBLIC_DST/"
rsync "${RSYNC_FLAGS[@]}" "$PRIVATE_SRC/" "$PUBLIC_DST/"

# ── Post-sync verification (only when applied) ───────────────────────────────
if [[ "$APPLY" -eq 1 ]]; then
  log "verifying public mirror is clean..."
  if hits=$(grep -rinE "$FORBIDDEN" "$PUBLIC_DST" \
              --include='*.rs' --include='*.toml' --include='*.md' \
              --include='*.sh' --include='*.yml' --include='*.yaml' \
              --include='*.json' --include='*.lock' \
              2>/dev/null | grep -v '/target/' | grep -v '/.git/'); then
    err "forbidden terms present in public AFTER sync — investigate:"
    echo "$hits" | head -30 >&2
    exit 3
  fi
  log "public mirror verified clean."
  log "done. Review, commit and push from: $PUBLIC_DST"
fi
