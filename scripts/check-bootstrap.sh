#!/usr/bin/env bash
# Who the bootstrap lists in each of the seed node's DHT spaces, with each entry's age.
# A new desktop finds the seed node only through these entries (20-minute lifetime).
# usage: bash scripts/check-bootstrap.sh [seed URL] [bootstrap URL]
set -euo pipefail
seed=${1:-https://seed-node-production.up.railway.app}
bootstrap=${2:-https://bootstrap-production-f978.up.railway.app}

READY=$(curl -fsS -m 15 "$seed/readyz") BOOTSTRAP=$bootstrap python3 - <<'PY'
import base64, json, os, time, urllib.request

ready = json.loads(os.environ["READY"])
bootstrap = os.environ["BOOTSTRAP"]
print("seed up", ready["uptimeSecs"], "s")
for role, dna in ready["cells"]["dnas"].items():
    body = dna[1:]
    raw = base64.urlsafe_b64decode(body + "=" * (-len(body) % 4))
    # A kitsune2 space is the DNA hash's 32 core bytes (no type prefix or location).
    space = base64.urlsafe_b64encode(raw[3:35]).decode().rstrip("=")
    with urllib.request.urlopen(f"{bootstrap}/bootstrap/{space}", timeout=15) as r:
        entries = json.load(r)
    print(f"{role}: {len(entries)} agents")
    now = time.time()
    for e in entries:
        a = json.loads(e["agentInfo"])
        age = int(now - int(a["createdAt"]) / 1e6)
        left = int(int(a["expiresAt"]) / 1e6 - now)
        print(f"  {a['agent'][:10]}  age {age} s, expires in {left} s")
PY
