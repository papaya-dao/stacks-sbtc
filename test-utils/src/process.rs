use libc::pid_t;
use std::process::Child;
use std::thread;
use std::time::{Duration, SystemTime};
use ureq::serde::Serialize;
use url::Url;

use ctrlc::Signal;
use nix::sys::signal;
use nix::unistd::Pid;
use ureq::serde_json::Value;
use ureq::{self, json, post};

pub struct Process {
    name: String,
    url: Url,
    child: Child,
}

impl Process {
    pub fn new(url: &str, port: u16, name: String, child: Child) -> Self {
        let mut url: Url = url.parse().unwrap();
        url.set_port(Some(port)).unwrap();
        let this = Self { name, url, child };
        this.set_handler();
        this.connectivity_check().unwrap();
        this
    }

    fn set_handler(&self) {
        let pid = self.child.id() as pid_t;
        let name = self.name.clone();
        ctrlc::set_handler(move || {
            println!("Killing {} pid {:?}...", name, pid);

            signal::kill(Pid::from_raw(pid), Signal::SIGTERM)
                .map_err(|e| println!("Warning: signaling {} {} failed {:?}", name, pid, e))
                .unwrap();
        })
        .expect("Error setting Ctrl-C handler");
    }

    pub fn rpc(&self, method: &str, params: impl Serialize) -> Value {
        let rpc = json!({"jsonrpc": "1.0", "id": "tst", "method": method, "params": params});

        match post(self.url.as_str()).send_json(&rpc) {
            Ok(response) => {
                let json = response.into_json::<Value>().unwrap();
                let result = json.as_object().unwrap().get("result").unwrap().clone();

                result
            }
            Err(err) => {
                let err_str = err.to_string();
                let err_obj_opt = match err.into_response() {
                    Some(r) => r.into_json::<Value>().unwrap(),
                    None => json!({ "error": &err_str }),
                };

                println!("{} -> {}", rpc, err_obj_opt);

                err_obj_opt
            }
        }
    }

    fn connectivity_check(&self) -> Result<f32, String> {
        let now = SystemTime::now();

        for _tries in 1..120 {
            let uptime = self.rpc("uptime", ());

            if uptime.is_number() {
                return Ok(now.elapsed().unwrap().as_secs_f32());
            } else {
                thread::sleep(Duration::from_millis(500));
            }
        }

        Err("connection timeout".to_string())
    }

    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    pub fn port(&self) -> u16 {
        self.url.port().unwrap()
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        self.child.kill().unwrap();
    }
}
