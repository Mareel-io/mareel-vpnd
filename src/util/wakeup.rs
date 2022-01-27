pub(crate) struct WakeupSender {
    cnc_url: String,
}

impl WakeupSender {
    pub fn new(cnc_url: &str, retry_interval: usize) -> Self {
        WakeupSender {
            cnc_url: cnc_url.to_string(),
        }
    }

    pub fn send_wakeup_msg() {
        //
    }
}
