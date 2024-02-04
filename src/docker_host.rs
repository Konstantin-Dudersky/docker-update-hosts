#[derive(Debug)]
pub struct DockerHost {
    pub hostname: String,
    pub ip_address: String,
}

impl DockerHost {
    pub fn new(hostname: Option<String>, ip_address: Option<String>) -> Option<Self> {
        let hostname = match hostname {
            Some(val) => val,
            None => return None,
        };
        let ip_address = match ip_address {
            Some(val) => val,
            None => return None,
        };
        if ip_address == "" {
            return None;
        }
        let s = Self {
            hostname,
            ip_address,
        };
        Some(s)
    }

    pub fn into_file_line(&self) -> String {
        format!("{:<16}{}", self.ip_address, self.hostname)
    }
}
