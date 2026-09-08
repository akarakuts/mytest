//! Модули проверок: health, SSO, UI, i18n, integration.

pub mod health;
pub mod i18n;
pub mod integration;
pub mod sso;
pub mod ui;

/// Результат одного теста.
#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub category: Category,
    pub status: Status,
    pub detail: Option<String>,
    pub duration_ms: u64,
    pub fix_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Pass,
    Fail,
    Warn,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Category {
    Health,
    Sso,
    Ui,
    I18n,
    Integration,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Health => write!(f, "Health"),
            Category::Sso => write!(f, "SSO"),
            Category::Ui => write!(f, "UI/UX"),
            Category::I18n => write!(f, "i18n"),
            Category::Integration => write!(f, "Integration"),
        }
    }
}

impl TestResult {
    pub fn pass(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category: Category::Health,
            status: Status::Pass,
            detail: None,
            duration_ms: 0,
            fix_hint: None,
        }
    }

    pub fn fail(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category: Category::Health,
            status: Status::Fail,
            detail: Some(detail.into()),
            duration_ms: 0,
            fix_hint: None,
        }
    }

    pub fn warn(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category: Category::Health,
            status: Status::Warn,
            detail: Some(detail.into()),
            duration_ms: 0,
            fix_hint: None,
        }
    }

    pub fn skip(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category: Category::Health,
            status: Status::Skip,
            detail: Some(reason.into()),
            duration_ms: 0,
            fix_hint: None,
        }
    }

    pub fn category(mut self, cat: Category) -> Self {
        self.category = cat;
        self
    }

    pub fn fix_hint(mut self, hint: impl Into<String>) -> Self {
        self.fix_hint = Some(hint.into());
        self
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }
}
