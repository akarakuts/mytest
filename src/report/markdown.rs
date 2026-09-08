//! Генерация Markdown-отчёта по результатам тестирования.

use crate::checks::{Category, Status, TestResult};
use chrono::Utc;
use std::io::Write;

pub fn generate(results: &[TestResult], path: &str) -> anyhow::Result<()> {
    let mut f = std::fs::File::create(path)?;

    let now = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let passed = results.iter().filter(|r| r.status == Status::Pass).count();
    let failed = results.iter().filter(|r| r.status == Status::Fail).count();
    let warned = results.iter().filter(|r| r.status == Status::Warn).count();
    let skipped = results.iter().filter(|r| r.status == Status::Skip).count();
    let total = results.len();

    writeln!(f, "# my* Suite Test Report")?;
    writeln!(f)?;
    writeln!(f, "**Date:** {}", now)?;
    writeln!(f, "**Total:** {} checks | **Passed:** {} | **Failed:** {} | **Warnings:** {} | **Skipped:** {}",
        total, passed, failed, warned, skipped)?;
    writeln!(f)?;

    // Score
    let score = if total > 0 {
        ((passed as f64 / total as f64) * 100.0) as u32
    } else {
        0
    };
    writeln!(f, "## Score: {}%", score)?;
    writeln!(f)?;

    if score >= 90 {
        writeln!(f, "> **Excellent** — the stack is in great shape.")?;
    } else if score >= 70 {
        writeln!(f, "> **Good** — some issues need attention.")?;
    } else if score >= 50 {
        writeln!(f, "> **Fair** — significant issues found.")?;
    } else {
        writeln!(f, "> **Needs work** — critical issues found.")?;
    }
    writeln!(f)?;

    // Per-category summary
    writeln!(f, "## Summary by Category")?;
    writeln!(f)?;
    writeln!(f, "| Category | Passed | Failed | Warnings | Skipped | Total |")?;
    writeln!(f, "|----------|--------|--------|----------|---------|-------|")?;

    for cat in &[
        Category::Health,
        Category::Sso,
        Category::Ui,
        Category::I18n,
        Category::Integration,
    ] {
        let cat_results: Vec<_> = results.iter().filter(|r| r.category == *cat).collect();
        if cat_results.is_empty() {
            continue;
        }
        let c_passed = cat_results.iter().filter(|r| r.status == Status::Pass).count();
        let c_failed = cat_results.iter().filter(|r| r.status == Status::Fail).count();
        let c_warned = cat_results.iter().filter(|r| r.status == Status::Warn).count();
        let c_skipped = cat_results.iter().filter(|r| r.status == Status::Skip).count();
        writeln!(
            f,
            "| {} | {} | {} | {} | {} | {} |",
            cat, c_passed, c_failed, c_warned, c_skipped, cat_results.len()
        )?;
    }
    writeln!(f)?;

    // Failed tests
    let failures: Vec<_> = results.iter().filter(|r| r.status == Status::Fail).collect();
    if !failures.is_empty() {
        writeln!(f, "## Failures")?;
        writeln!(f)?;
        for r in &failures {
            writeln!(f, "### :x: {}", r.name)?;
            if let Some(detail) = &r.detail {
                writeln!(f)?;
                writeln!(f, "{}", detail)?;
            }
            if let Some(hint) = &r.fix_hint {
                writeln!(f)?;
                writeln!(f, "**Fix:** {}", hint)?;
            }
            writeln!(f)?;
        }
    }

    // Warnings
    let warnings: Vec<_> = results.iter().filter(|r| r.status == Status::Warn).collect();
    if !warnings.is_empty() {
        writeln!(f, "## Warnings")?;
        writeln!(f)?;
        for r in &warnings {
            writeln!(f, "### :warning: {}", r.name)?;
            if let Some(detail) = &r.detail {
                writeln!(f)?;
                writeln!(f, "{}", detail)?;
            }
            if let Some(hint) = &r.fix_hint {
                writeln!(f)?;
                writeln!(f, "**Suggestion:** {}", hint)?;
            }
            writeln!(f)?;
        }
    }

    // Passed tests
    writeln!(f, "## Passed")?;
    writeln!(f)?;
    for r in results.iter().filter(|r| r.status == Status::Pass) {
        let duration = if r.duration_ms > 0 {
            format!(" ({}ms)", r.duration_ms)
        } else {
            String::new()
        };
        writeln!(f, "- :white_check_mark: {}{}", r.name, duration)?;
    }
    writeln!(f)?;

    // Skipped
    if skipped > 0 {
        writeln!(f, "## Skipped")?;
        writeln!(f)?;
        for r in results.iter().filter(|r| r.status == Status::Skip) {
            if let Some(detail) = &r.detail {
                writeln!(f, "- :large_blue_circle: {} — {}", r.name, detail)?;
            } else {
                writeln!(f, "- :large_blue_circle: {}", r.name)?;
            }
        }
        writeln!(f)?;
    }

    // UX/UI improvement suggestions
    writeln!(f, "## UX/UI Improvement Suggestions")?;
    writeln!(f)?;
    writeln!(f, "### High Priority")?;
    writeln!(f)?;
    writeln!(f, "1. **Consistent navbar across all apps** — ensure every app has: brand, nav links, app-switcher, theme toggle, help button, lang picker")?;
    writeln!(f, "2. **Mobile responsiveness** — verify viewport meta, test on 375px/768px/1024px widths")?;
    writeln!(f, "3. **Loading states** — every async operation should show spinner/skeleton, not blank page")?;
    writeln!(f, "4. **Error boundaries** — wrap Suspense components to prevent full-page crashes")?;
    writeln!(f)?;
    writeln!(f, "### Medium Priority")?;
    writeln!(f)?;
    writeln!(f, "1. **Keyboard navigation** — Tab order, focus indicators, Escape to close modals")?;
    writeln!(f, "2. **Toast notifications** — show success/error feedback for mutations")?;
    writeln!(f, "3. **Empty states** — meaningful illustrations/messages when lists are empty")?;
    writeln!(f, "4. **Search across all apps** — global search bar in portal that queries mysearch")?;
    writeln!(f)?;
    writeln!(f, "### Low Priority")?;
    writeln!(f)?;
    writeln!(f, "1. **Animations** — subtle transitions for page loads, modals, dropdowns")?;
    writeln!(f, "2. **Keyboard shortcuts** — Ctrl+K palette in all apps (already in some)")?;
    writeln!(f, "3. **Breadcrumbs** — for deep navigation in myconf/myjira/myservicedesk")?;
    writeln!(f, "4. **Drag-and-drop** — for kanban boards, file uploads, list reordering")?;
    writeln!(f)?;

    Ok(())
}
