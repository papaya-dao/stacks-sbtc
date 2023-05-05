use crate::process::Process;
use crate::util::find_port;
use std::process::{Command, Stdio};
use ureq::serde::Serialize;
use ureq::serde_json::Value;

const STACKS_NODE_URL: &str = "http://efgh:efgh@localhost";

pub struct StacksNodeProcess {
    //datadir: PathBuf, //Should retreive this from the toml or set it in the toml and delete when done (working_dir)
    pub process: Process,
}

impl StacksNodeProcess {
    pub fn new() -> Self {
        let port = find_port().unwrap();
        let name = "stacks-node".to_string();

        let stacks_node_child = Command::new(&name)
            .arg("start")
            .arg("--config Stacks.toml")
            .stdout(Stdio::null())
            .spawn()
            .expect(&format!("{name} failed to start"));

        let process = Process::new(STACKS_NODE_URL, port, name, stacks_node_child);

        Self {
            process,
            //datadir,
        }
    }

    pub fn rpc(&self, method: &str, params: impl Serialize) -> Value {
        self.process.rpc(method, params)
    }
}

// impl Drop for StacksNodeProcess {
//     fn drop(&mut self) {
//         remove_dir_all(&self.datadir).unwrap();
//     }
// }

//TODO: add something like the stacks-blockchain repos' integration_test_get_info()
