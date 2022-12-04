use serde_json::Value;
use std::fs;

pub mod user;
use user::Users;

pub mod firmware;
use firmware::Firmware;

#[derive(Clone)]
pub struct App {
    pub data: Value,
    pub users: Users,
    pub firmware: Firmware,
}

impl App {
    pub fn new(conf_file: &str, user_file: &str, firmware_dir: &str) -> Result<App, Box<dyn std::error::Error>> {
        // Read configuration from file
        let content = fs::read(conf_file)?;

        // Desrialize data to Json Value
        let data: Value = serde_json::from_slice(&content)?;
        let users = Users::new(user_file)?;
        let firmware = Firmware::new(firmware_dir);

        Ok(App { data, users, firmware })
    }
}

pub fn path_to_pointer(path: String) -> String {
    let mut pointer = "/".to_owned();
    pointer.push_str(path.as_str());
    if pointer.ends_with("/") {
        pointer.pop();
    }
    pointer
}

pub fn json_merge(a: &mut Value, b: Value) {
    if let Value::Object(a) = a {
        if let Value::Object(b) = b {
            for (k, v) in b {
                if v.is_null() {
                    a.remove(&k);
                }
                else {
                    json_merge(a.entry(k).or_insert(Value::Null), v);
                }
            } 

            return;
        }
    }

    *a = b;
}