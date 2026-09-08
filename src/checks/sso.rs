//! Проверки SSO / OIDC: Crowd discovery, token exchange, cross-app SSO flow.

use crate::checks::{Category, TestResult};
use crate::config::ServiceConfig;
use reqwest::Client;

fn http_client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

/// Проверить Crowd OIDC discovery endpoint.
async fn check_crowd_discovery(config: &ServiceConfig) -> TestResult {
    let url = format!("{}/.well-known/openid-configuration", config.crowd_url);
    let client = http_client();

    match client.get(&url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                let issuer = body.get("issuer").and_then(|v| v.as_str()).unwrap_or("");
                let auth_endpoint = body
                    .get("authorization_endpoint")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if issuer.is_empty() || auth_endpoint.is_empty() {
                    TestResult::fail(
                        "Crowd OIDC discovery",
                        "Discovery document missing issuer or authorization_endpoint",
                    )
                    .category(Category::Sso)
                    .fix_hint("Check Crowd OAuth module — discovery document incomplete")
                } else {
                    TestResult::pass("Crowd OIDC discovery")
                        .category(Category::Sso)
                        .fix_hint(format!("issuer={}, auth={}", issuer, auth_endpoint))
                }
            } else {
                TestResult::fail(
                    "Crowd OIDC discovery",
                    format!("HTTP {}", resp.status().as_u16()),
                )
                .category(Category::Sso)
                .fix_hint("Check Crowd service and PUBLIC_BASE_URL config")
            }
        }
        Err(e) => TestResult::fail("Crowd OIDC discovery", format!("Connection failed: {}", e))
            .category(Category::Sso)
            .fix_hint("Start Crowd service on port 8080"),
    }
}

/// Проверить Crowd JWKS endpoint.
async fn check_crowd_jwks(config: &ServiceConfig) -> TestResult {
    let url = format!("{}/oauth/jwks", config.crowd_url);
    let client = http_client();

    match client.get(&url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                let keys = body.get("keys").and_then(|v| v.as_array());
                match keys {
                    Some(k) if !k.is_empty() => {
                        let kid = k[0].get("kid").and_then(|v| v.as_str()).unwrap_or("?");
                        let alg = k[0].get("alg").and_then(|v| v.as_str()).unwrap_or("?");
                        TestResult::pass("Crowd JWKS")
                            .category(Category::Sso)
                            .fix_hint(format!("kid={}, alg={}", kid, alg))
                    }
                    _ => TestResult::fail("Crowd JWKS", "No keys in JWKS response")
                        .category(Category::Sso)
                        .fix_hint("Check Crowd signing key configuration"),
                }
            } else {
                TestResult::fail("Crowd JWKS", format!("HTTP {}", resp.status().as_u16()))
                    .category(Category::Sso)
            }
        }
        Err(e) => TestResult::fail("Crowd JWKS", format!("Connection failed: {}", e))
            .category(Category::Sso),
    }
}

/// Проверить Crowd login endpoint.
async fn check_crowd_login(config: &ServiceConfig) -> TestResult {
    let url = format!("{}/api/Login", config.crowd_url);
    let client = http_client();

    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(r#"{"input":{"username":"test-nonexistent","password":"test"}}"#)
        .send()
        .await;

    match resp {
        Ok(r) => {
            let status = r.status();
            let body = r.text().await.unwrap_or_default();
            if status.is_success() || body.contains("invalid credentials") {
                TestResult::pass("Crowd login endpoint").category(Category::Sso)
            } else {
                TestResult::fail(
                    "Crowd login endpoint",
                    format!("HTTP {}: {}", status.as_u16(), &body[..body.len().min(200)]),
                )
                .category(Category::Sso)
            }
        }
        Err(e) => TestResult::fail("Crowd login endpoint", format!("Connection failed: {}", e))
            .category(Category::Sso),
    }
}

/// Проверить OIDC authorize flow (без реального логина).
async fn check_oidc_authorize(config: &ServiceConfig) -> TestResult {
    // Test authorize with invalid client_id — should return error, not crash
    let url = format!(
        "{}/oauth/authorize?response_type=code&client_id=invalid&redirect_uri=http://localhost&scope=openid&state=test",
        config.crowd_url
    );
    let client = http_client();

    match client.get(&url).send().await {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            if status.is_client_error() && body.contains("error") {
                TestResult::pass("Crowd authorize error handling").category(Category::Sso)
            } else if status.is_success() {
                TestResult::warn(
                    "Crowd authorize error handling",
                    "Accepted invalid client_id without error",
                )
                .category(Category::Sso)
                .fix_hint("Crowd should reject authorize requests with invalid client_id")
            } else {
                TestResult::warn(
                    "Crowd authorize error handling",
                    format!("HTTP {}: {}", status.as_u16(), &body[..body.len().min(200)]),
                )
                .category(Category::Sso)
            }
        }
        Err(e) => {
            TestResult::fail("Crowd authorize error handling", format!("{}", e)).category(Category::Sso)
        }
    }
}

/// Проверить наличие OIDC кнопки на странице логина.
async fn check_sso_login_button(svc: &crate::config::Service, config: &ServiceConfig) -> TestResult {
    let url = format!("{}/login", config.base_url_http(svc));
    let client = http_client();

    match client.get(&url).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                return TestResult::skip(
                    format!("{} SSO login button", svc.name),
                    format!("Login page returned HTTP {}", resp.status().as_u16()),
                )
                .category(Category::Sso);
            }
            let body = resp.text().await.unwrap_or_default();
            if body.contains("oidc") || body.contains("sso") || body.contains("SSO") {
                TestResult::pass(format!("{} SSO login button", svc.name)).category(Category::Sso)
            } else {
                TestResult::warn(
                    format!("{} SSO login button", svc.name),
                    "No OIDC/SSO link found on login page",
                )
                .category(Category::Sso)
                .fix_hint(format!(
                    "Add SSO login button to {} login page — look for /auth/oidc/login link",
                    svc.name
                ))
            }
        }
        Err(_) => TestResult::skip(
            format!("{} SSO login button", svc.name),
            "Service unreachable",
        )
        .category(Category::Sso),
    }
}

/// Запустить все SSO checks.
pub async fn run_all(config: &ServiceConfig) -> Vec<TestResult> {
    let mut results = Vec::new();

    results.push(check_crowd_discovery(config).await);
    results.push(check_crowd_jwks(config).await);
    results.push(check_crowd_login(config).await);
    results.push(check_oidc_authorize(config).await);

    // Check SSO login buttons on OIDC-enabled services
    for svc in &config.services {
        if svc.has_oidc {
            results.push(check_sso_login_button(svc, config).await);
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use crate::checks::Status;
    use super::*;
    use crate::config::ServiceConfig;

    #[tokio::test]
    async fn crowd_discovery_pass() {
        let cfg = ServiceConfig::default();
        let r = check_crowd_discovery(&cfg).await;
        assert_eq!(r.status, Status::Pass);
        assert!(r.name.contains("discovery"));
    }

    #[tokio::test]
    async fn crowd_jwks_pass() {
        let cfg = ServiceConfig::default();
        let r = check_crowd_jwks(&cfg).await;
        assert_eq!(r.status, Status::Pass);
        assert!(r.fix_hint.is_some());
    }

    #[tokio::test]
    async fn crowd_login_pass() {
        let cfg = ServiceConfig::default();
        let r = check_crowd_login(&cfg).await;
        assert_eq!(r.status, Status::Pass);
    }

    #[tokio::test]
    async fn crowd_authorize_error_handling() {
        let cfg = ServiceConfig::default();
        let r = check_oidc_authorize(&cfg).await;
        // Should pass — Crowd correctly rejects invalid client_id
        assert_eq!(r.status, Status::Pass);
    }

    #[tokio::test]
    async fn sso_login_button_on_oidc_services() {
        let cfg = ServiceConfig::default();
        // Test a few OIDC-enabled services
        for name in &["myjira", "mybitbucket", "myportal"] {
            let svc = cfg.by_name(name).unwrap();
            let r = check_sso_login_button(svc, &cfg).await;
            assert_eq!(r.status, Status::Pass, "{} should have SSO login button", name);
        }
    }

    #[tokio::test]
    async fn sso_login_button_on_crowd() {
        let cfg = ServiceConfig::default();
        let crowd = cfg.by_name("mycrowd").unwrap();
        // Crowd is the IdP, not an OIDC client — skip
        // (has_oidc is false, so it won't be checked in run_all)
        assert!(!crowd.has_oidc);
    }

    #[tokio::test]
    async fn run_all_returns_correct_count() {
        let cfg = ServiceConfig::default();
        let results = run_all(&cfg).await;
        // 4 Crowd checks + 26 OIDC services × 1 check = 30
        assert_eq!(results.len(), 30);
    }

    #[tokio::test]
    async fn run_all_crowd_checks_pass() {
        let cfg = ServiceConfig::default();
        let results = run_all(&cfg).await;
        let crowd_results: Vec<_> = results.iter().filter(|r| r.name.contains("Crowd")).collect();
        assert_eq!(crowd_results.len(), 4);
        for r in crowd_results {
            assert_eq!(r.status, Status::Pass, "{} failed", r.name);
        }
    }
}
