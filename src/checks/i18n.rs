//! Проверки i18n: наличие языковых каталогов, переключение языков, полнота переводов.

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

/// Проверить SSR на разных языках — проверяем что lang cookie работает.
async fn check_lang_switch(svc: &crate::config::Service, config: &ServiceConfig) -> TestResult {
    let url = config.base_url_http(svc);
    let client = http_client();

    // Get default page (should be ru)
    let resp_ru = client
        .get(&url)
        .header("Accept-Language", "ru")
        .send()
        .await;

    let resp_en = client
        .get(&url)
        .header("Accept-Language", "en")
        .send()
        .await;

    match (resp_ru, resp_en) {
        (Ok(ru), Ok(en)) => {
            let ru_body = ru.text().await.unwrap_or_default();
            let en_body = en.text().await.unwrap_or_default();

            // Check if lang attribute changes
            let ru_lang = ru_body.contains("lang=\"ru\"");
            let en_lang = en_body.contains("lang=\"en\"");

            // Check for different text content
            let ru_has_cyrillic = ru_body.chars().any(|c| c >= '\u{0400}' && c <= '\u{04FF}');
            let en_has_english = en_body.contains("Login") || en_body.contains("Sign in") || en_body.contains("Repositories");

            if ru_lang && (en_lang || en_has_english) {
                TestResult::pass(format!("{} lang switch", svc.name)).category(Category::I18n)
            } else if ru_has_cyrillic {
                TestResult::warn(
                    format!("{} lang switch", svc.name),
                    "Russian content found but English may not work",
                )
                .category(Category::I18n)
                .fix_hint("Check Accept-Language header handling in SSR")
            } else {
                TestResult::warn(
                    format!("{} lang switch", svc.name),
                    "Language switching not detected",
                )
                .category(Category::I18n)
            }
        }
        _ => TestResult::skip(format!("{} lang switch", svc.name), "Service unreachable")
            .category(Category::I18n),
    }
}

/// Проверить что основные UI-строки переведены (не hardcoded English).
async fn check_no_hardcoded_strings(
    svc: &crate::config::Service,
    config: &ServiceConfig,
) -> TestResult {
    let url = config.base_url_http(svc);
    let client = http_client();

    match client.get(&url).send().await {
        Ok(resp) => {
            let body = resp.text().await.unwrap_or_default();

            // For Russian-default services, check if they have Russian text
            if svc.comment_lang == "ru" {
                let has_cyrillic = body.chars().any(|c| c >= '\u{0400}' && c <= '\u{04FF}');
                if has_cyrillic {
                    TestResult::pass(format!("{} i18n strings", svc.name)).category(Category::I18n)
                } else {
                    TestResult::warn(
                        format!("{} i18n strings", svc.name),
                        "Expected Russian UI text but found none — may be using English defaults",
                    )
                    .category(Category::I18n)
                }
            } else {
                TestResult::pass(format!("{} i18n strings", svc.name)).category(Category::I18n)
            }
        }
        Err(_) => TestResult::skip(format!("{} i18n strings", svc.name), "Service unreachable")
            .category(Category::I18n),
    }
}

/// Проверить наличие lang picker с 19 языками.
async fn check_full_lang_set(
    svc: &crate::config::Service,
    config: &ServiceConfig,
) -> TestResult {
    let url = config.base_url_http(svc);
    let client = http_client();

    match client.get(&url).send().await {
        Ok(resp) => {
            let body = resp.text().await.unwrap_or_default();

            // Expected languages
            let expected = [
                "ru", "en", "tt", "ba", "uk", "be", "kk", "uz", "az", "ky", "hy", "ka", "de",
                "fr", "es", "it", "pt", "nl", "pl",
            ];

            let mut missing = Vec::new();
            for lang in &expected {
                if !body.contains(&format!("value=\"{}\"", lang)) {
                    missing.push(*lang);
                }
            }

            if missing.is_empty() {
                TestResult::pass(format!("{} full lang set (19)", svc.name)).category(Category::I18n)
            } else if missing.len() <= 5 {
                TestResult::warn(
                    format!("{} full lang set", svc.name),
                    format!("Missing languages: {}", missing.join(", ")),
                )
                .category(Category::I18n)
                .fix_hint("Add missing language options to lang picker")
            } else {
                TestResult::fail(
                    format!("{} full lang set", svc.name),
                    format!("Missing {} languages: {}...", missing.len(), missing[..5].join(", ")),
                )
                .category(Category::I18n)
                .fix_hint("Implement all 19 languages in i18n catalogs")
            }
        }
        Err(_) => TestResult::skip(format!("{} full lang set", svc.name), "Service unreachable")
            .category(Category::I18n),
    }
}

/// Запустить все i18n checks.
pub async fn run_all(config: &ServiceConfig) -> Vec<TestResult> {
    let mut results = Vec::new();

    for svc in &config.services {
        if svc.has_i18n {
            results.push(check_lang_switch(svc, config).await);
            results.push(check_no_hardcoded_strings(svc, config).await);
            results.push(check_full_lang_set(svc, config).await);
        }
    }

    results
}
