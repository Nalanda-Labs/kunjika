#!/usr/bin/env bash
# Regenerate the throwaway RSA keypairs used by backend/template.test.json.
#
# Why: backend/src/users/token.rs signs RS256 JWTs with
#     base64_decode(config.access_token_private_key) -> EncodingKey::from_rsa_pem
# i.e. the config value must be a *base64-encoded PEM*, and it must not be
# empty - an empty key makes POST /auth/login answer 502 {"message":"InvalidKeyFormat"}.
#
# These keys are TEST-ONLY and are gitignored (see .gitignore). Regenerating is
# safe: nothing outside the test database depends on them.
#
# Usage: ./scripts/gen_test_keys.sh
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
harness="$(dirname "$here")"
keys="$harness/keys"
cfg="$harness/../template.test.json"

mkdir -p "$keys"
for n in access refresh; do
  openssl genrsa -out "$keys/${n}_private.pem" 2048 2>/dev/null
  openssl rsa -in "$keys/${n}_private.pem" -pubout -out "$keys/${n}_public.pem" 2>/dev/null
done

python3 - "$keys" "$cfg" <<'PY'
import base64, json, pathlib, sys
keys, cfg = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
b64 = lambda n: base64.b64encode((keys / n).read_bytes()).decode()
d = json.loads(cfg.read_text())
d["access_token_private_key"]  = b64("access_private.pem")
d["access_token_public_key"]   = b64("access_public.pem")
d["refresh_token_private_key"] = b64("refresh_private.pem")
d["refresh_token_public_key"]  = b64("refresh_public.pem")
cfg.write_text(json.dumps(d, indent=2) + "\n")
print(f"updated {cfg}")
PY

echo "Done. Restart the backend for the new keys to take effect."