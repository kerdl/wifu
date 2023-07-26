pub struct Operator {
    chosen: Option<String>
}
impl Operator {
    pub fn get(&self) -> Option<&str> {
        self.chosen.as_ref().map(|s| s.as_str())
    }

    fn set(&mut self, ssid: String) {
        self.chosen = Some(ssid)
    }

    pub fn is_chosen(&self) -> bool {
        self.chosen.is_some()
    }

    pub fn is_ssid_chosen(&self, ssid: &str) -> bool {
        self.chosen.as_ref().map(|chosen| chosen == ssid).unwrap_or(false)
    }

    pub async fn choose(&mut self) -> Option<&str> {
        unimplemented!()
    }

    pub async fn unchoose(&mut self) -> Result<(), ()> {
        if self.chosen.is_none() {
            return Err(())
        }

        self.chosen = None;

        Ok(())
    }
}
impl Default for Operator {
    fn default() -> Self {
        Self { chosen: None }
    }
}