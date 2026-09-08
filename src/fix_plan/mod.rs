//! Генерация плана исправлений на основе результатов тестирования.

use crate::checks::{Status, TestResult};
use chrono::Utc;
use std::io::Write;

/// Приоритет исправления.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

struct FixItem {
    priority: Priority,
    name: String,
    detail: String,
    fix_hint: String,
    affected_services: Vec<String>,
}

pub fn generate(results: &[TestResult], path: &str) -> anyhow::Result<()> {
    let mut fixes: Vec<FixItem> = Vec::new();

    // Collect failures as critical/high priority fixes
    for r in results.iter().filter(|r| r.status == Status::Fail) {
        let priority = if r.name.contains("healthz") || r.name.contains("SSO") || r.name.contains("OIDC") {
            Priority::Critical
        } else {
            Priority::High
        };

        let service = r
            .name
            .split_whitespace()
            .next()
            .unwrap_or("unknown")
            .to_string();

        fixes.push(FixItem {
            priority,
            name: r.name.clone(),
            detail: r.detail.clone().unwrap_or_default(),
            fix_hint: r.fix_hint.clone().unwrap_or_default(),
            affected_services: vec![service],
        });
    }

    // Collect warnings as medium/low priority
    for r in results.iter().filter(|r| r.status == Status::Warn) {
        let priority = if r.name.contains("security") || r.name.contains("SSO") {
            Priority::Medium
        } else {
            Priority::Low
        };

        let service = r
            .name
            .split_whitespace()
            .next()
            .unwrap_or("unknown")
            .to_string();

        fixes.push(FixItem {
            priority,
            name: r.name.clone(),
            detail: r.detail.clone().unwrap_or_default(),
            fix_hint: r.fix_hint.clone().unwrap_or_default(),
            affected_services: vec![service],
        });
    }

    // Sort by priority
    fixes.sort_by(|a, b| a.priority.cmp(&b.priority));

    // Write plan
    let mut f = std::fs::File::create(path)?;
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    writeln!(f, "# Fix Plan — my* Suite")?;
    writeln!(f)?;
    writeln!(f, "**Generated:** {}", now)?;
    writeln!(f, "**Total items:** {}", fixes.len())?;
    writeln!(f)?;

    // Summary
    let critical = fixes.iter().filter(|f| f.priority == Priority::Critical).count();
    let high = fixes.iter().filter(|f| f.priority == Priority::High).count();
    let medium = fixes.iter().filter(|f| f.priority == Priority::Medium).count();
    let low = fixes.iter().filter(|f| f.priority == Priority::Low).count();

    writeln!(f, "| Priority | Count |")?;
    writeln!(f, "|----------|-------|")?;
    if critical > 0 {
        writeln!(f, "| :red_circle: Critical | {} |", critical)?;
    }
    if high > 0 {
        writeln!(f, "| :orange_circle: High | {} |", high)?;
    }
    if medium > 0 {
        writeln!(f, "| :yellow_circle: Medium | {} |", medium)?;
    }
    if low > 0 {
        writeln!(f, "| :blue_circle: Low | {} |", low)?;
    }
    writeln!(f)?;

    // Group by priority
    for (priority, icon, label) in &[
        (Priority::Critical, ":red_circle:", "Critical"),
        (Priority::High, ":orange_circle:", "High"),
        (Priority::Medium, ":yellow_circle:", "Medium"),
        (Priority::Low, ":blue_circle:", "Low"),
    ] {
        let items: Vec<_> = fixes.iter().filter(|f| f.priority == *priority).collect();
        if items.is_empty() {
            continue;
        }

        writeln!(f, "## {} {} Priority", icon, label)?;
        writeln!(f)?;

        for (i, item) in items.iter().enumerate() {
            writeln!(f, "### {}. {}", i + 1, item.name)?;
            writeln!(f)?;
            writeln!(f, "**Affected:** {}", item.affected_services.join(", "))?;
            if !item.detail.is_empty() {
                writeln!(f)?;
                writeln!(f, "**Problem:** {}", item.detail)?;
            }
            if !item.fix_hint.is_empty() {
                writeln!(f)?;
                writeln!(f, "**Fix:** {}", item.fix_hint)?;
            }
            writeln!(f)?;
            writeln!(f, "- [ ] Not started")?;
            writeln!(f)?;
        }
    }

    // UX/UI improvement plan
    writeln!(f, "---")?;
    writeln!(f)?;
    writeln!(f, "## UX/UI Improvement Plan")?;
    writeln!(f)?;
    writeln!(f, "### Phase 1: Consistency (Week 1)")?;
    writeln!(f)?;
    writeln!(f, "- [ ] Verify navbar components in all 27 apps (brand, nav, switcher, theme, help, lang)")?;
    writeln!(f, "- [ ] Verify viewport meta in all SSR shells")?;
    writeln!(f, "- [ ] Verify all apps use docs/design/suite.css")?;
    writeln!(f, "- [ ] Fix any missing CSS custom properties")?;
    writeln!(f)?;
    writeln!(f, "### Phase 2: Accessibility (Week 2)")?;
    writeln!(f)?;
    writeln!(f, "- [ ] Add aria-labels to all buttons without text")?;
    writeln!(f, "- [ ] Add alt attributes to all images")?;
    writeln!(f, "- [ ] Verify focus indicators on interactive elements")?;
    writeln!(f, "- [ ] Test keyboard navigation flow")?;
    writeln!(f)?;
    writeln!(f, "### Phase 3: Polish (Week 3)")?;
    writeln!(f)?;
    writeln!(f, "- [ ] Add loading spinners/skeletons to all async pages")?;
    writeln!(f, "- [ ] Add empty state messages to all list views")?;
    writeln!(f, "- [ ] Add toast notifications for CRUD operations")?;
    writeln!(f, "- [ ] Add error boundaries around Suspense components")?;
    writeln!(f)?;
    writeln!(f, "### Phase 4: Advanced (Week 4)")?;
    writeln!(f)?;
    writeln!(f, "- [ ] Add breadcrumb navigation to deep pages")?;
    writeln!(f, "- [ ] Add Ctrl+K command palette to all apps")?;
    writeln!(f, "- [ ] Add subtle animations/transitions")?;
    writeln!(f, "- [ ] Add drag-and-drop where appropriate")?;
    writeln!(f)?;

    Ok(())
}
