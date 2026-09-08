//! Конфигурация всех сервисов my* для тестирования.

/// Описание одного сервиса.
#[derive(Debug, Clone)]
pub struct Service {
    pub name: &'static str,
    pub display_name: &'static str,
    pub domain: &'static str,
    pub port: u16,
    pub has_oidc: bool,
    pub has_api: bool,
    pub has_i18n: bool,
    pub comment_lang: &'static str, // "en" or "ru"
}

/// Конфигурация всего стека.
#[derive(Debug)]
pub struct ServiceConfig {
    pub services: Vec<Service>,
    pub base_ip: &'static str,
    pub crowd_url: &'static str,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            base_ip: "192.168.1.66",
            crowd_url: "http://192.168.1.66:8080",
            services: vec![
                Service { name: "mycrowd", display_name: "Crowd (IdP)", domain: "crowd.home.local", port: 8080, has_oidc: false, has_api: true, has_i18n: true, comment_lang: "en" },
                Service { name: "myconf", display_name: "Confluence", domain: "conf.home.local", port: 3100, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "en" },
                Service { name: "myjira", display_name: "Jira", domain: "jira.home.local", port: 3001, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "en" },
                Service { name: "mybitbucket", display_name: "Bitbucket", domain: "bitbucket.home.local", port: 3002, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "en" },
                Service { name: "mybamboo", display_name: "Bamboo", domain: "bamboo.home.local", port: 3003, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "en" },
                Service { name: "myportal", display_name: "Portal", domain: "portal.home.local", port: 3004, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "en" },
                Service { name: "myopsgenie", display_name: "Opsgenie", domain: "opsgenie.home.local", port: 3005, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
                Service { name: "myservicedesk", display_name: "Service Desk", domain: "servicedesk.home.local", port: 3006, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
                Service { name: "mycompass", display_name: "Compass", domain: "compass.home.local", port: 3007, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
                Service { name: "mycalendars", display_name: "Calendars", domain: "calendars.home.local", port: 3008, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
                Service { name: "mystatuspage", display_name: "Statuspage", domain: "status.home.local", port: 8090, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "en" },
                Service { name: "mycrm", display_name: "CRM", domain: "crm.home.local", port: 8070, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
                Service { name: "myrovo", display_name: "Rovo (AI)", domain: "rovo.home.local", port: 3010, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
                Service { name: "myanalytics", display_name: "Analytics", domain: "analytics.home.local", port: 3011, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
                Service { name: "mymarketplace", display_name: "Marketplace", domain: "marketplace.home.local", port: 3012, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
                Service { name: "myalign", display_name: "Align", domain: "align.home.local", port: 3013, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
                Service { name: "mynotifications", display_name: "Notifications", domain: "notifications.home.local", port: 3014, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
                Service { name: "mychat", display_name: "Chat", domain: "chat.home.local", port: 3015, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
                Service { name: "mytrello", display_name: "Trello", domain: "trello.home.local", port: 3016, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
                Service { name: "mydiscovery", display_name: "Discovery", domain: "discovery.home.local", port: 3017, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
                Service { name: "myatlas", display_name: "Atlas", domain: "atlas.home.local", port: 3018, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
                Service { name: "myflow", display_name: "Flow (Automation)", domain: "flow.home.local", port: 3020, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
                Service { name: "mysearch", display_name: "Search", domain: "search.home.local", port: 3021, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
                Service { name: "myjam", display_name: "Jam (Whiteboard)", domain: "jam.home.local", port: 3022, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
                Service { name: "myrunbook", display_name: "Runbook", domain: "runbook.home.local", port: 3023, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
                Service { name: "mytimesheets", display_name: "Timesheets", domain: "timesheets.home.local", port: 3024, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
                Service { name: "myforms", display_name: "Forms", domain: "forms.home.local", port: 3025, has_oidc: true, has_api: true, has_i18n: true, comment_lang: "ru" },
            ],
        }
    }
}

impl ServiceConfig {
    pub fn by_name(&self, name: &str) -> Option<&Service> {
        self.services.iter().find(|s| s.name == name)
    }

    pub fn base_url_https(&self, svc: &Service) -> String {
        format!("https://{}", svc.domain)
    }

    pub fn base_url_http(&self, svc: &Service) -> String {
        format!("http://{}:{}", self.base_ip, svc.port)
    }

    pub fn health_url(&self, svc: &Service) -> String {
        format!("http://{}:{}/healthz", self.base_ip, svc.port)
    }
}
