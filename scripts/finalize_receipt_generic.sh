#!/usr/bin/env bash
# =============================================================================
# finalize_receipt_generic.sh — MKTd02 Receipt Finalization Recovery Tool (Phases B + C)
#
# Purpose:
#   Recovery tool for the failure case where Phase A completed (Pending receipt)
#   but Phase B/C did not. This is NOT the normal user flow; normal flow is
#   frontend orchestration (A→B→C) with recovery-on-load.
#
# Modes:
#   1) --factory-canister <id> : calls finalize_profile_receipt on a controller proxy (DaffyDefs-style)
#   2) --direct               : calls mktd_finalize_receipt directly on the profile canister (requires controller identity)
#
# Usage:
#   ./scripts/finalize_receipt_generic.sh --profile <profile_canister_id> [--network local|ic] \
#       [--factory-canister <factory_id>] [--direct] [--identity <dfx_identity>]
#
# Examples:
#   # DaffyDefs-style (factory proxy):
#   ./scripts/finalize_receipt_generic.sh --profile be2us-... --factory-canister br5f7-... --network local
#
#   # Direct finalize (controller identity required):
#   ./scripts/finalize_receipt_generic.sh --profile <id> --direct --identity <controller_identity> --network ic
#
# Requirements:
#   - dfx CLI
#   - python3 (parses dfx JSON output)
# =============================================================================
set -euo pipefail

NETWORK="local"
PROFILE=""
FACTORY=""
MODE="factory"   # factory | direct
IDENTITY=""

die() { echo "ERROR: $*" >&2; exit 1; }

usage() {
  cat <<EOF
Usage:
  $0 --profile <profile_canister_id> [--network local|ic] [--factory-canister <id> | --direct] [--identity <dfx_identity>]

Options:
  --profile <id>           Profile canister id (required)
  --network <net>          dfx network name (default: local)
  --factory-canister <id>  Factory/controller-proxy canister id (Phase C via finalize_profile_receipt)
  --direct                 Call mktd_finalize_receipt directly on profile canister (requires controller identity)
  --identity <name>        dfx identity to use (optional; recommended for --direct)
  -h, --help               Show help

EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile) PROFILE="${2:-}"; shift 2;;
    --network) NETWORK="${2:-}"; shift 2;;
    --factory-canister) FACTORY="${2:-}"; MODE="factory"; shift 2;;
    --direct) MODE="direct"; shift 1;;
    --identity) IDENTITY="${2:-}"; shift 2;;
    -h|--help) usage; exit 0;;
    *) die "Unknown argument: $1";;
  esac
done

[[ -n "$PROFILE" ]] || die "--profile is required"
if [[ "$MODE" == "factory" && -z "$FACTORY" ]]; then
  die "Factory mode requires --factory-canister <id> (or use --direct)"
fi

DFX=(dfx --network "$NETWORK")
if [[ -n "$IDENTITY" ]]; then
  "${DFX[@]}" identity use "$IDENTITY" >/dev/null
fi

echo "[*] Network: $NETWORK"
echo "[*] Profile canister: $PROFILE"
echo "[*] Mode: $MODE"
[[ -n "$FACTORY" ]] && echo "[*] Factory canister: $FACTORY"
[[ -n "$IDENTITY" ]] && echo "[*] Identity: $IDENTITY"

echo "[*] Checking pending state..."
PENDING=$("${DFX[@]}" canister call "$PROFILE" mktd_is_pending --output json \
  | python3 - <<'PY'
import json,sys
d=json.load(sys.stdin)
# dfx json output often wraps as {"Ok":...} or raw bool; handle both
if isinstance(d, bool): print("true" if d else "false")
elif isinstance(d, dict):
    # Some canisters may return bool directly; but handle candid json encoding
    # Example: {"Ok": true}
    v=list(d.values())[0] if d else False
    print("true" if v else "false")
else:
    print("false")
PY
)

if [[ "$PENDING" != "true" ]]; then
  echo "[✓] No pending receipt. Nothing to finalize."
  exit 0
fi

echo "[*] Phase B: fetching certificate via mktd_get_certificate()..."
CERT_JSON=$("${DFX[@]}" canister call "$PROFILE" mktd_get_certificate --output json)
# Parse receipt_id and certificate bytes
read -r RECEIPT_ID CERT_VEC <<<"$(python3 - <<'PY'
import json,sys
d=json.loads(sys.argv[1])
# Expect opt record => either null or {"Some": {"receipt_id": "...", "certificate": [...], ...}}
if d is None:
    print("", "")
    sys.exit(0)
if isinstance(d, dict) and "Some" in d:
    rec=d["Some"]
    rid=rec.get("receipt_id","")
    cert=rec.get("certificate",[])
    # cert may be list of ints; output as "vec { ... }" fragment
    if not isinstance(cert, list): cert=[]
    cert_vec="vec { " + "; ".join(str(int(x)) for x in cert) + " }"
    print(rid, cert_vec)
    sys.exit(0)
print("", "")
PY "$CERT_JSON")"

[[ -n "$RECEIPT_ID" ]] || die "mktd_get_certificate returned None/empty; receipt may not be ready yet."

echo "[*] Got receipt_id: $RECEIPT_ID"
echo "[*] Phase C: finalizing..."

if [[ "$MODE" == "factory" ]]; then
  "${DFX[@]}" canister call "$FACTORY" finalize_profile_receipt \
    "(principal \"$PROFILE\", \"$RECEIPT_ID\", $CERT_VEC)" >/dev/null
else
  "${DFX[@]}" canister call "$PROFILE" mktd_finalize_receipt \
    "(\"$RECEIPT_ID\", $CERT_VEC)" >/dev/null
fi

echo "[*] Confirming pending is now false..."
PENDING2=$("${DFX[@]}" canister call "$PROFILE" mktd_is_pending --output json \
  | python3 - <<'PY'
import json,sys
d=json.load(sys.stdin)
if isinstance(d, bool): print("true" if d else "false")
elif isinstance(d, dict):
    v=list(d.values())[0] if d else False
    print("true" if v else "false")
else:
    print("false")
PY
)

if [[ "$PENDING2" == "true" ]]; then
  die "Still pending after finalization attempt. Check logs; you may need retries."
fi

echo "[✓] Finalized. receipt_id=$RECEIPT_ID"
