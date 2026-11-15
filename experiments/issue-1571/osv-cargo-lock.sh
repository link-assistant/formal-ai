#!/usr/bin/env bash
# Issue #1571: list RustSec/OSV advisories for every package in a Cargo.lock
# without building cargo-audit. Usage: experiments/issue-1571/osv-cargo-lock.sh <Cargo.lock>
set -euo pipefail
lock="${1:?usage: osv-cargo-lock.sh <Cargo.lock>}"
D="$(mktemp -d)"
trap 'rm -rf "${D:?}"' EXIT
awk '/^name = /{n=$3}/^version = /{print n" "$3}' "$lock" | tr -d '"' \
  | node -e 'const l=require("fs").readFileSync(0,"utf8").trim().split("\n").map(x=>x.split(" "));process.stdout.write(JSON.stringify({queries:l.map(([n,v])=>({package:{name:n,ecosystem:"crates.io"},version:v}))}))' > "$D/q.json"
curl -sf -X POST -H 'content-type: application/json' --data @"$D/q.json" https://api.osv.dev/v1/querybatch -o "$D/r.json"
node -e 'const q=require(process.argv[1]).queries,r=require(process.argv[2]).results;let n=0;r.forEach((x,i)=>{if(x.vulns){n++;console.log(q[i].package.name,q[i].version,x.vulns.map(v=>v.id).join(","))}});console.log(`${n} vulnerable package version(s) in ${process.argv[3]}`)' "$D/q.json" "$D/r.json" "$lock"
