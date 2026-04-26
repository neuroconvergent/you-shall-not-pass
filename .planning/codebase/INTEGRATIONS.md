# External Integrations

**Analysis Date:** 2026-04-26

## APIs & External Services

**None detected.**

The project is a standalone desktop application with no network-facing API calls, cloud services, or third-party SaaS integrations. All functionality is local-only.

## Data Storage

**Databases:**
- Not applicable — no database usage

**File Storage:**
- Local filesystem only
- Planned: XDG-compliant configuration files (`~/.config/you-shall-not-pass/`)
- No cloud storage or remote file sync integrations

**Caching:**
- None

## Authentication & Identity

**Auth Provider:**
- Not applicable — no user accounts, login flows, or identity providers
- The application generates secrets locally; no authentication is performed against external services

## Monitoring & Observability

**Error Tracking:**
- None

**Logs:**
- Standard `println!` / `eprintln!` or tracing (not yet implemented)
- No structured logging framework or external log aggregation

**Metrics:**
- Not applicable

## CI/CD & Deployment

**Hosting:**
- Local desktop application (no hosted service)

**CI Pipeline:**
- **GitHub Actions** — `.github/workflows/opencode.yml`
  - Trigger: `issue_comment` and `pull_request_review_comment` events
  - Condition: comments containing `/oc` or `/opencode`
  - Runner: `ubuntu-latest`
  - Action: `anomalyco/opencode/github@latest`
  - Secrets: `OPENCODE_API_KEY` (GitHub repository secret)

**Release / Packaging:**
- Not yet configured
- Planned: Flatpak or AppImage for Linux distribution (per `PLAN.md`)

## Environment Configuration

**Required env vars (CI only):**
- `OPENCODE_API_KEY` — GitHub Actions secret for the opencode AI agent integration

**Secrets location:**
- GitHub repository secrets (`secrets.OPENCODE_API_KEY`)
- No local `.env` files or secret directories in the repository

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## Notable Absence of Integrations

Given the security-focused nature of this password generator, the following are **intentionally absent** (and that is a design feature, not a gap):

- No cloud password storage (KeepsPass, 1Password, Bitwarden APIs)
- No network telemetry or update checks
- No external entropy sources (relies solely on OS CSPRNG via `getrandom`)
- No OAuth, SSO, or identity federation
- No analytics or crash reporting services

---

*Integration audit: 2026-04-26*
