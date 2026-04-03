use hw_probe::HardwareProfile;

#[derive(Debug, Clone)]
pub struct AppState {
    pub profile: Option<HardwareProfile>,
    pub selected_tab: usize,
    pub is_loading: bool,
    pub error: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            profile: None,
            selected_tab: 0,
            is_loading: false,
            error: None,
        }
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    pub fn set_profile(&mut self, profile: HardwareProfile) {
        self.profile = Some(profile);
        self.is_loading = false;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.is_loading = false;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
