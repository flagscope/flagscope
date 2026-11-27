#!/usr/bin/env bash
# Checks every commit in <base>..<head> against CONTRIBUTING.md (RNF-09.1):
# - it carries a "Signed-off-by:" line (DCO);
# - no Author, Committer, Signed-off-by or Co-authored-by names an AI tool.
# Merge commits created by GitHub are exempt from the sign-off.
# Prints every offending commit and exits 1 if there is any.
set -euo pipefail

base=${1:?usage: check-commits.sh <base> <head>}
head=${2:?usage: check-commits.sh <base> <head>}

# Names and addresses of AI tools, matched without regard to case. Extend the
# list when a new tool appears; a human named like one of these can still
# contribute under a different spelling of their address.
ai_pattern='claude|anthropic|copilot|chatgpt|openai|gemini|codex|devin'

failures=0
while read -r commit; do
  parents=$(git rev-list --parents -n 1 "$commit" | wc -w)
  committer=$(git show -s --format='%cn <%ce>' "$commit")
  subject=$(git show -s --format='%h %s' "$commit")
  is_github_merge=false
  if [ "$parents" -gt 2 ] && [ "$committer" = "GitHub <noreply@github.com>" ]; then
    is_github_merge=true
  fi

  if ! $is_github_merge && ! git show -s --format='%(trailers:key=Signed-off-by)' "$commit" | grep -q .; then
    echo "::error::commit $subject has no Signed-off-by line (git commit -s)"
    failures=$((failures + 1))
  fi

  identities=$(git show -s --format='%an <%ae>%n%cn <%ce>%n%(trailers:key=Signed-off-by,valueonly)%n%(trailers:key=Co-authored-by,valueonly)' "$commit")
  if echo "$identities" | grep -qiE "$ai_pattern"; then
    offender=$(echo "$identities" | grep -iE "$ai_pattern" | head -1)
    echo "::error::commit $subject names an AI tool: $offender"
    failures=$((failures + 1))
  fi
done < <(git rev-list "$base..$head")

if [ "$failures" -gt 0 ]; then
  echo "$failures problem(s) found. See \"Authorship and AI tools\" and \"License and sign-off\" in CONTRIBUTING.md."
  exit 1
fi
echo "Every commit in $base..$head is signed off and names no AI tool."
