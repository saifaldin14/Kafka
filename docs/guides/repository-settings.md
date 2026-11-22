# Repository settings

GitHub settings that aren't files in the repository, recorded here so they can be checked and rebuilt. When you change one, change this page in the same PR.

The repository has one maintainer, so the settings protect `main` with checks rather than with a second person's review. The [deviations](#deviations-for-a-single-maintainer) say what changes when a second maintainer joins.

## Branch protection: the `main` ruleset

The ruleset is exported to [`.github/rulesets/main.json`](../../.github/rulesets/main.json), and the live ruleset is created from that file:

```bash
gh api -X POST repos/saifaldin14/Kafka/rulesets --input .github/rulesets/main.json
# or, to update the existing one:
gh api -X PUT repos/saifaldin14/Kafka/rulesets/<id> --input .github/rulesets/main.json
```

It applies to the default branch and nobody may bypass it:

| Rule | Setting | Why |
|---|---|---|
| Required status check | `ci-success`, from GitHub Actions only (`integration_id` 15368), and the branch must be up to date with `main` | One aggregate check ([ci.yml](../../.github/workflows/ci.yml)), so adding a CI job never needs a settings change. Pinning the app stops another app from posting a passing `ci-success` |
| Pull request | Required, with 0 approvals; stale approvals are dismissed on push; review threads must be resolved; merge by rebase or squash | Every change goes through CI on a PR, and nothing lands with an open review thread |
| Linear history | Required | `git log` and `git bisect` see one line of tested commits |
| Force pushes | Blocked | History on `main` is never rewritten |
| Deletion | Blocked | `main` can't be deleted |

## Repository settings

| Setting | Value |
|---|---|
| Merge methods | Rebase and squash; merge commits off (linear history) |
| Delete head branches after merge | On |
| Actions `GITHUB_TOKEN` default permissions | Read only; Actions can't approve PRs. Each workflow asks for more per job (`check-workflows` checks that every job sets `permissions`) |
| Fork pull requests | Workflows wait for a maintainer's approval for every outside contributor |
| Dependabot alerts | On; version updates come from [`dependabot.yml`](../../.github/dependabot.yml) |
| Secret scanning and push protection | On |

To read the live values:

```bash
R=repos/saifaldin14/Kafka
gh api $R/rulesets/<id>
gh api $R --jq '{allow_merge_commit, allow_squash_merge, allow_rebase_merge, delete_branch_on_merge, security_and_analysis}'
gh api $R/actions/permissions/workflow
gh api $R/actions/permissions/fork-pr-contributor-approval
gh api $R/vulnerability-alerts -i   # 204 when alerts are on
```

## Deviations for a single maintainer

The backlog task (FND-0061) also lists these, which aren't set:

- **One approval and code-owner review.** GitHub never lets an author approve their own PR, so with one maintainer every PR would be blocked. When a second maintainer joins, set `required_approving_review_count` to 1 and `require_code_owner_review` to true in the JSON, and apply it. [CODEOWNERS](../../.github/CODEOWNERS) already lists the security-sensitive paths.
- **Merge queue.** GitHub offers merge queues only to repositories owned by an organization. Until then, the "up to date with `main`" requirement gives the same guarantee, that the merged code passed CI as it will exist on `main`, at the cost of updating a branch before merging.
