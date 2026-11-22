#!/usr/bin/env sh
# Pre-filter every text extract to power-related lines (with original line numbers) for the sweep agents.
# Usage: tools/facts/make_hits.sh <refs_dir>
# Output: <refs_dir>/facts/sweep/hits/<name>.txt. Long space runs are shortened to 3 spaces.
set -e
R="$1"
P='RPDU|SPDU|ELCU|ELCC|ELCF|SSPC|\bSSR\b|\bC[EKT][0-9]{7}\b|\bP(100|150|200|300|400|500|600|700|800|49)\b|powered (from|by)|power (from|to)\b|suppl(y|ies) (power|28|115|235|270)|28 ?v ?dc|115 ?v ?ac|235 ?v ?ac|270 ?v ?dc|hot bat|battery bus|standby (bus|power)|instrument bus|ground (handling|service) bus|circuit breaker|\bTCB\b|\bECB\b|[0-9.]+ ?(amps?|A)\b'
mkdir -p "$R/facts/sweep/hits"
for f in "$R"/text/*.txt "$R"/ocr/fig_*.txt; do
  [ -f "$f" ] || continue
  b=$(basename "$f" .txt)
  grep -n -i -E -C1 "$P" "$f" | sed -e 's/\r$//' -e 's/    \+/   /g' > "$R/facts/sweep/hits/$b.txt" || true
done
