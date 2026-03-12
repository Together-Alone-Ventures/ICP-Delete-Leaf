#!/usr/bin/env bash
# =============================================================================
# finalize_receipt_generic.sh — Generic MKTd02 Recovery Tool (Phases B + C)
#
# Purpose:
#   Recover/finalize a pending MKTd02 receipt after Phase A succeeded but
#   certificate capture / finalization did not complete.
#
# Supports:
#   --factory-canister <id>   Finalize via factory proxy
#   --direct                  Finalize directly on the profile canister
#
# Examples:
#   ./scripts/finalize_receipt_generic.sh \
#     --profile ichn3-7qaaa-aaaaj-qqsya-cai \
#     --factory-canister 5g26e-liaaa-aaaaj-qp4tq-cai \
#     --network ic
#
#   ./scripts/finalize_receipt_generic.sh \
#     --profile be2us-64aaa-aaaaa-qaabq-cai \
#     --direct \
#     --network local
# =============================================================================
set -euo pipefail

PROFILE=""
FACTORY=""
NETWORK="local"
IDENTITY=""
MODE=""

usage() {
  cat <<'EOF'
Usage:
  finalize_receipt_generic.sh --profile <canister_id> [--factory-canister <id> | --direct] [--network <name>] [--identity <name>]

Required:
  --profile <canister_id>        Profile canister id

Mode (choose one):
  --factory-canister <id>        Finalize via factory proxy
  --direct                       Finalize directly on the profile canister

Optional:
  --network <name>               DFX network name (default: local)
  --identity <name>              DFX identity to use
  --help                         Show this help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      PROFILE="${2:-}"
      shift 2
      ;;
    --factory-canister)
      FACTORY="${2:-}"
      MODE="factory"
      shift 2
      ;;
    --direct)
      MODE="direct"
      shift
      ;;
    --network)
      NETWORK="${2:-}"
      shift 2
      ;;
    --identity)
      IDENTITY="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "ERROR: Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ -z "$PROFILE" ]]; then
  echo "ERROR: --profile is required" >&2
  usage >&2
  exit 1
fi

if [[ -z "$MODE" ]]; then
  echo "ERROR: choose either --factory-canister <id> or --direct" >&2
  usage >&2
  exit 1
fi

if [[ "$MODE" == "factory" && -z "$FACTORY" ]]; then
  echo "ERROR: --factory-canister is required in factory mode" >&2
  exit 1
fi

DFX=(dfx)

if [[ -n "$IDENTITY" ]]; then
  echo "[*] Switching identity: $IDENTITY"
  "${DFX[@]}" identity use "$IDENTITY" >/dev/null
fi

echo "[*] Network: $NETWORK"
echo "[*] Profile canister: $PROFILE"
echo "[*] Mode: $MODE"
if [[ "$MODE" == "factory" ]]; then
  echo "[*] Factory canister: $FACTORY"
fi

echo "[*] Checking pending state..."
PENDING_JSON=$("${DFX[@]}" canister --network "$NETWORK" call "$PROFILE" mktd_is_pending --output json --query)

python3 - "$PENDING_JSON" <<'PY'
import json, sys
raw = sys.argv[1]
try:
    data = json.loads(raw)
except Exception as e:
    print(f"ERROR: could not parse mktd_is_pending JSON: {e}", file=sys.stderr)
    print(raw, file=sys.stderr)
    sys.exit(2)

def extract_bool(x):
    if isinstance(x, bool):
        return x
    if isinstance(x, list) and len(x) == 1:
        return extract_bool(x[0])
    if isinstance(x, dict):
        if "Ok" in x:
            return extract_bool(x["Ok"])
        if "ok" in x:
            return extract_bool(x["ok"])
    raise ValueError(f"unexpected shape: {x!r}")

try:
    pending = extract_bool(data)
except Exception as e:
    print(f"ERROR: could not extract pending flag: {e}", file=sys.stderr)
    print(data, file=sys.stderr)
    sys.exit(3)

if not pending:
    print("[*] No pending receipt found. Nothing to finalize.")
    sys.exit(10)

print("[*] Pending receipt detected.")
PY

status=$?
if [[ $status -eq 10 ]]; then
  exit 0
elif [[ $status -ne 0 ]]; then
  exit $status
fi

echo "[*] Fetching certificate..."
CERT_JSON=$("${DFX[@]}" canister --network "$NETWORK" call "$PROFILE" mktd_get_certificate --output json --query)

PARSED=$(python3 - "$CERT_JSON" <<'PY'
import json, sys
raw = sys.argv[1]
try:
    data = json.loads(raw)
except Exception as e:
    print(f"ERROR: could not parse mktd_get_certificate JSON: {e}", file=sys.stderr)
    print(raw, file=sys.stderr)
    sys.exit(2)

def walk(x):
    if isinstance(x, dict):
        if "Ok" in x:
            return walk(x["Ok"])
        if "ok" in x:
            return walk(x["ok"])
        if "Err" in x or "err" in x:
            raise ValueError(f"certificate call returned error: {x!r}")

        # Friendly-field candidate record
        rid_keys = ["receipt_id", "receiptId", "640_735_298"]
        cert_keys = ["certificate", "bls_certificate", "cert", "457_361_687"]

        rid = None
        cert = None
        for k in rid_keys:
            if k in x:
                rid = x[k]
                break
        for k in cert_keys:
            if k in x:
                cert = x[k]
                break

        if rid is not None and cert is not None:
            return rid, cert

        for v in x.values():
            try:
                return walk(v)
            except Exception:
                pass

    elif isinstance(x, list):
        for item in x:
            try:
                return walk(item)
            except Exception:
                pass

    raise ValueError(f"could not locate receipt_id/certificate in: {x!r}")

receipt_id, cert = walk(data)

if not isinstance(receipt_id, str) or not receipt_id:
    raise ValueError(f"invalid receipt_id: {receipt_id!r}")

# cert may arrive as list[int] or hex/text depending on candid/json rendering
if isinstance(cert, list):
    cert_len = len(cert)
    cert_arg = "vec {" + "; ".join(str(int(b)) for b in cert) + "}"
elif isinstance(cert, str):
    # keep as raw text for diagnostics; reject empty
    cert_len = len(cert)
    if cert_len == 0:
      raise ValueError("empty certificate string")
    # If somehow encoded as hex/text, direct canister-call blob syntax won't accept it.
    # Emit a marker so the shell can stop with a useful error.
    cert_arg = "__STRING_CERT__:" + cert
else:
    raise ValueError(f"unexpected certificate shape: {type(cert).__name__}: {cert!r}")

print(receipt_id)
print(cert_len)
print(cert_arg)
PY
)

RECEIPT_ID=$(printf '%s\n' "$PARSED" | sed -n '1p')
CERT_LEN=$(printf '%s\n' "$PARSED" | sed -n '2p')
CERT_ARG=$(printf '%s\n' "$PARSED" | sed -n '3p')

if [[ "$CERT_ARG" == __STRING_CERT__:* ]]; then
  echo "ERROR: mktd_get_certificate returned certificate as string text, not byte vector." >&2
  echo "Raw marker: $CERT_ARG" >&2
  exit 4
fi

if [[ -z "$RECEIPT_ID" || -z "$CERT_LEN" || -z "$CERT_ARG" ]]; then
  echo "ERROR: failed to extract receipt_id/certificate from mktd_get_certificate output" >&2
  exit 5
fi

if [[ "$CERT_LEN" -le 0 ]]; then
  echo "ERROR: certificate blob is empty" >&2
  exit 6
fi

echo "[*] Receipt id: $RECEIPT_ID"
echo "[*] Certificate length: $CERT_LEN bytes"

if [[ "$MODE" == "factory" ]]; then
  echo "[*] Finalizing via factory proxy..."
"${DFX[@]}" canister --network "$NETWORK" call "$FACTORY" finalize_profile_receipt \
  "(principal \"$PROFILE\", \"$RECEIPT_ID\", $CERT_ARG)"

else
  echo "[*] Finalizing directly on profile canister..."
"${DFX[@]}" canister --network "$NETWORK" call "$PROFILE" mktd_finalize_receipt \
  "(\"$RECEIPT_ID\", $CERT_ARG)"
fi

echo "[*] Re-checking pending state..."
PENDING2_JSON=$("${DFX[@]}" canister --network "$NETWORK" call "$PROFILE" mktd_is_pending --output json --query)

python3 - "$PENDING2_JSON" <<'PY'
import json, sys
raw = sys.argv[1]
try:
    data = json.loads(raw)
except Exception as e:
    print(f"ERROR: could not parse final mktd_is_pending JSON: {e}", file=sys.stderr)
    print(raw, file=sys.stderr)
    sys.exit(2)

def extract_bool(x):
    if isinstance(x, bool):
        return x
    if isinstance(x, list) and len(x) == 1:
        return extract_bool(x[0])
    if isinstance(x, dict):
        if "Ok" in x:
            return extract_bool(x["Ok"])
        if "ok" in x:
            return extract_bool(x["ok"])
    raise ValueError(f"unexpected shape: {x!r}")

pending = extract_bool(data)
if pending:
    print("[!] Receipt still pending.")
    sys.exit(11)
else:
    print("[✓] Receipt no longer pending. Finalization appears complete.")
PY
