//! UI/UX аудит: структура HTML, accessibility, design consistency, responsive design.

use crate::checks::{Category, TestResult};
use crate::config::ServiceConfig;
use reqwest::Client;
use scraper::{Html, Selector};

fn http_client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

/// Проверить наличие viewport meta tag (responsive design).
async fn check_viewport(svc: &crate::config::Service, config: &ServiceConfig) -> TestResult {
    let url = config.base_url_http(svc);
    let client = http_client();

    match client.get(&url).send().await {
        Ok(resp) => {
            let body = resp.text().await.unwrap_or_default();
            if body.contains("viewport") {
                TestResult::pass(format!("{} viewport meta", svc.name)).category(Category::Ui)
            } else {
                TestResult::fail(
                    format!("{} viewport meta", svc.name),
                    "Missing viewport meta tag — mobile layout will break",
                )
                .category(Category::Ui)
                .fix_hint("Add <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"> to SSR shell")
            }
        }
        Err(_) => TestResult::skip(format!("{} viewport meta", svc.name), "Service unreachable")
            .category(Category::Ui),
    }
}

/// Проверить наличие nav элемента (навигация).
async fn check_navigation(svc: &crate::config::Service, config: &ServiceConfig) -> TestResult {
    let url = config.base_url_http(svc);
    let client = http_client();

    match client.get(&url).send().await {
        Ok(resp) => {
            let body = resp.text().await.unwrap_or_default();
            let has_nav = body.contains("<nav") || body.contains("class=\"nav\"");
            let has_brand = body.contains("nav-brand") || body.contains("brand");
            let has_links = body.matches("<a ").count() > 2;

            let mut issues = Vec::new();
            if !has_nav {
                issues.push("no <nav> element");
            }
            if !has_brand {
                issues.push("no brand/logo");
            }
            if !has_links {
                issues.push("few navigation links");
            }

            if issues.is_empty() {
                TestResult::pass(format!("{} navigation", svc.name)).category(Category::Ui)
            } else {
                TestResult::warn(
                    format!("{} navigation", svc.name),
                    format!("Issues: {}", issues.join(", ")),
                )
                .category(Category::Ui)
            }
        }
        Err(_) => TestResult::skip(format!("{} navigation", svc.name), "Service unreachable")
            .category(Category::Ui),
    }
}

/// Проверить наличие theme toggle (dark/light mode).
async fn check_theme_toggle(svc: &crate::config::Service, config: &ServiceConfig) -> TestResult {
    let url = config.base_url_http(svc);
    let client = http_client();

    match client.get(&url).send().await {
        Ok(resp) => {
            let body = resp.text().await.unwrap_or_default();
            let has_theme_toggle =
                body.contains("theme-toggle") || body.contains("data-theme") || body.contains("ThemeToggle");
            let has_theme_script = body.contains("suite-theme") || body.contains("data-theme");

            if has_theme_toggle && has_theme_script {
                TestResult::pass(format!("{} theme toggle", svc.name)).category(Category::Ui)
            } else if has_theme_script {
                TestResult::warn(
                    format!("{} theme toggle", svc.name),
                    "Theme script present but no visible toggle button",
                )
                .category(Category::Ui)
                .fix_hint("Add ThemeToggle component to navbar")
            } else {
                TestResult::warn(
                    format!("{} theme toggle", svc.name),
                    "No theme toggle or data-theme attribute found",
                )
                .category(Category::Ui)
                .fix_hint("Add theme toggle with localStorage persistence")
            }
        }
        Err(_) => TestResult::skip(format!("{} theme toggle", svc.name), "Service unreachable")
            .category(Category::Ui),
    }
}

/// Проверить app-switcher (меню переключения между приложениями).
async fn check_app_switcher(svc: &crate::config::Service, config: &ServiceConfig) -> TestResult {
    let url = config.base_url_http(svc);
    let client = http_client();

    match client.get(&url).send().await {
        Ok(resp) => {
            let body = resp.text().await.unwrap_or_default();
            let has_switcher =
                body.contains("suite-switcher") || body.contains("suite-menu") || body.contains("app-switcher");

            if has_switcher {
                // Count how many apps are listed
                let app_count = body.matches("home.local").count();
                if app_count >= 20 {
                    TestResult::pass(format!("{} app switcher", svc.name))
                        .category(Category::Ui)
                        .fix_hint(format!("Lists {} apps", app_count))
                } else {
                    TestResult::warn(
                        format!("{} app switcher", svc.name),
                        format!("Only {} apps in switcher (expected 27)", app_count),
                    )
                    .category(Category::Ui)
                    .fix_hint("Check SUITE_APPS env variable — may be missing some apps")
                }
            } else {
                TestResult::warn(
                    format!("{} app switcher", svc.name),
                    "No app switcher found on page",
                )
                .category(Category::Ui)
                .fix_hint("Add SUITE_APPS dropdown to navbar")
            }
        }
        Err(_) => TestResult::skip(format!("{} app switcher", svc.name), "Service unreachable")
            .category(Category::Ui),
    }
}

/// Проверить accessibility: наличие lang атрибута, alt на img, aria-labels.
async fn check_accessibility(svc: &crate::config::Service, config: &ServiceConfig) -> TestResult {
    let url = config.base_url_http(svc);
    let client = http_client();

    match client.get(&url).send().await {
        Ok(resp) => {
            let body = resp.text().await.unwrap_or_default();
            let doc = Html::parse_document(&body);

            let mut issues = Vec::new();

            // Check <html lang="...">
            let html_selector = Selector::parse("html").unwrap();
            if let Some(html_el) = doc.select(&html_selector).next() {
                if html_el.value().attr("lang").is_none() {
                    issues.push("<html> missing lang attribute");
                }
            } else {
                issues.push("no <html> element found");
            }

            // Check for images without alt
            let img_selector = Selector::parse("img").unwrap();
            let imgs: Vec<_> = doc.select(&img_selector).collect();
            let imgs_no_alt = imgs
                .iter()
                .filter(|el| el.value().attr("alt").is_none())
                .count();
            let img_msg;
            if imgs_no_alt > 0 {
                img_msg = format!("{} <img> without alt", imgs_no_alt);
                issues.push(img_msg.as_str());
            }

            // Check for buttons without accessible name
            let btn_selector = Selector::parse("button").unwrap();
            let btns: Vec<_> = doc.select(&btn_selector).collect();
            let btns_no_label = btns
                .iter()
                .filter(|el| {
                    el.value().attr("aria-label").is_none()
                        && el.text().collect::<String>().trim().is_empty()
                })
                .count();
            let btn_msg;
            if btns_no_label > 0 {
                btn_msg = format!("{} <button> without label", btns_no_label);
                issues.push(btn_msg.as_str());
            }

            if issues.is_empty() {
                TestResult::pass(format!("{} accessibility", svc.name)).category(Category::Ui)
            } else {
                TestResult::warn(
                    format!("{} accessibility", svc.name),
                    format!("Issues: {}", issues.join(", ")),
                )
                .category(Category::Ui)
                .fix_hint("Add aria-label to buttons, alt to images, lang to <html>")
            }
        }
        Err(_) => TestResult::skip(format!("{} accessibility", svc.name), "Service unreachable")
            .category(Category::Ui),
    }
}

/// Проверить наличие help button/contextual help.
async fn check_help_system(svc: &crate::config::Service, config: &ServiceConfig) -> TestResult {
    let url = config.base_url_http(svc);
    let client = http_client();

    match client.get(&url).send().await {
        Ok(resp) => {
            let body = resp.text().await.unwrap_or_default();
            let has_help = body.contains("help-toggle") || body.contains("HelpButton") || body.contains("Помощь") || body.contains("Help");

            if has_help {
                TestResult::pass(format!("{} help system", svc.name)).category(Category::Ui)
            } else {
                TestResult::warn(
                    format!("{} help system", svc.name),
                    "No help button found on page",
                )
                .category(Category::Ui)
                .fix_hint("Add HelpButton to navbar and HelpPanel to shell")
            }
        }
        Err(_) => TestResult::skip(format!("{} help system", svc.name), "Service unreachable")
            .category(Category::Ui),
    }
}

/// Проверить lang picker (переключатель языков).
async fn check_lang_picker(svc: &crate::config::Service, config: &ServiceConfig) -> TestResult {
    let url = config.base_url_http(svc);
    let client = http_client();

    match client.get(&url).send().await {
        Ok(resp) => {
            let body = resp.text().await.unwrap_or_default();
            let has_picker = body.contains("lang-picker") || body.contains("lang-select");

            if has_picker {
                // Count language options
                let lang_count = body.matches("<option").count();
                if lang_count >= 2 {
                    TestResult::pass(format!("{} lang picker", svc.name))
                        .category(Category::Ui)
                        .fix_hint(format!("{} languages available", lang_count))
                } else {
                    TestResult::warn(
                        format!("{} lang picker", svc.name),
                        format!("Only {} language options", lang_count),
                    )
                    .category(Category::Ui)
                }
            } else {
                TestResult::warn(
                    format!("{} lang picker", svc.name),
                    "No language picker found",
                )
                .category(Category::Ui)
                .fix_hint("Add language picker to navbar")
            }
        }
        Err(_) => TestResult::skip(format!("{} lang picker", svc.name), "Service unreachable")
            .category(Category::Ui),
    }
}

/// Проверить наличие CSS custom properties (design tokens).
async fn check_design_tokens(svc: &crate::config::Service, config: &ServiceConfig) -> TestResult {
    let css_url = format!("{}/pkg/{}.css", config.base_url_http(svc), svc.name);
    let client = http_client();

    match client.get(&css_url).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                return TestResult::skip(
                    format!("{} design tokens", svc.name),
                    "CSS file not found",
                )
                .category(Category::Ui);
            }
            let body = resp.text().await.unwrap_or_default();
            let has_vars = body.contains("--") && body.contains(":");
            let has_theme = body.contains("[data-theme") || body.contains("prefers-color-scheme");

            if has_vars && has_theme {
                TestResult::pass(format!("{} design tokens", svc.name)).category(Category::Ui)
            } else if has_vars {
                TestResult::warn(
                    format!("{} design tokens", svc.name),
                    "CSS variables found but no theme support",
                )
                .category(Category::Ui)
            } else {
                TestResult::warn(
                    format!("{} design tokens", svc.name),
                    "No CSS custom properties found",
                )
                .category(Category::Ui)
                .fix_hint("Use CSS custom properties from docs/design/suite.css")
            }
        }
        Err(_) => TestResult::skip(format!("{} design tokens", svc.name), "CSS unreachable")
            .category(Category::Ui),
    }
}

/// Запустить все UI checks.
pub async fn run_all(config: &ServiceConfig) -> Vec<TestResult> {
    let mut results = Vec::new();

    for svc in &config.services {
        results.push(check_viewport(svc, config).await);
        results.push(check_navigation(svc, config).await);
        results.push(check_theme_toggle(svc, config).await);
        results.push(check_app_switcher(svc, config).await);
        results.push(check_accessibility(svc, config).await);
        results.push(check_help_system(svc, config).await);
        results.push(check_lang_picker(svc, config).await);
        results.push(check_design_tokens(svc, config).await);
    }

    results
}
