use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    // Launch args
    pub graphics: bool,
    pub skiptobnet: bool,
    pub sndbkg: bool,
}

impl Settings {
    pub fn compose_args(&self) -> Vec<&str> {
        let mut args = Vec::new();

        if self.graphics {
            args.push("-ddraw");
        } else {
            args.push("-3dfx");
        }

        if self.skiptobnet {
            args.push("-skiptobnet");
        }

        if self.sndbkg {
            args.push("-sndbkg");
        }

        args
    }
}
