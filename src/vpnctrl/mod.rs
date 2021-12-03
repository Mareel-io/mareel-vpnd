mod error;
pub(crate) mod platform_specific;

pub struct WgInterface {
    ifname: String,
    privkey: String,
}

impl WgInterface {
    fn new(name: String, privkey: String) -> WgInterface {
        WgInterface {
            ifname: name,
            privkey,
        }
    }

    fn add_peers(&mut self, peer: WgPeer) {
        //
    }
}

pub struct WgPeer {
    pub pubk: String,
    pub psk: Option<String>,
    pub endpoint: Option<String>,
    pub allowed_ips: Vec<String>,
    pub keepalive: Option<i64>,
}
