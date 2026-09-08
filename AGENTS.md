# AGENTS.md — mytest

Integration test suite and UX/UI audit tool for the my* Atlassian-compatible stack.

## Purpose

`mytest` validates the entire 27-service stack after deployment:
- **Health checks**: endpoints, response times, security headers, HTTPS via Caddy
- **SSO/OIDC**: Crowd discovery, JWKS, authorize flow, login buttons
- **UI/UX audit**: viewport, navigation, theme toggle, app-switcher, accessibility, help system, lang picker, design tokens
- **i18n**: language switching, no hardcoded strings, 19-language completeness
- **Integration**: SUITE_APPS links, widget endpoints, OIDC client configs, CORS

Generates:
- `reports/test-report.md` — full test report with score
- `reports/fix-plan.md` — prioritized fix plan with checkboxes

## Build and run

```sh
cargo build --release
./target/release/mytest run                # all checks
./target/release/mytest run --health-only  # health only
./target/release/mytest run --sso-only     # SSO only
./target/release/mytest run --ui-only      # UI/UX only
./target/release/mytest run --i18n-only    # i18n only
./target/release/mytest status             # quick status table
./target/release/mytest plan               # generate fix plan
```

## Stack

- Rust 2021, tokio, reqwest, scraper, clap, handlebars
- No database, no web server — pure CLI tool
- Tests run via HTTP against live services

## Configuration

All service definitions are in `src/config.rs`. Services are accessed via:
- Direct HTTP: `http://{ip}:{port}`
- HTTPS via Caddy: `https://{domain}.home.local`

## Test categories

| Category | Count | Description |
|----------|-------|-------------|
| Health | ~108 | 27 services × 4 checks (healthz, HTTPS, security headers, response time) |
| SSO | ~31 | Crowd OIDC + 27 SSO login buttons |
| UI/UX | ~216 | 27 services × 8 checks (viewport, nav, theme, switcher, a11y, help, lang, tokens) |
| i18n | ~81 | 27 services × 3 checks (lang switch, no hardcoded, full lang set) |
| Integration | ~30+ | SUITE_APPS, widgets, OIDC configs, CORS |

## Output

Reports are written to `reports/` directory:
- `test-report.md` — scored report with failures, warnings, passed, skipped
- `fix-plan.md` — prioritized action items with UX/UI improvement phases
