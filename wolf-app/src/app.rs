use serde_json::{json, Value};
use serde::{Deserialize};
use std::fs;
use std::process::Command;
use std::sync::{Arc, RwLock};

pub mod user;
use user::Users;

pub mod firmware;
use firmware::Firmware;

pub mod validator;
use validator::Validator;

pub mod event;
use event::Event;

pub mod gpio;
pub use gpio::GPIO;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Conf {
    data_file: String,
    user_file: String,
    firmware_directory: String,
    validator_file: String,
    netd_file: String,
    device: String,
}

#[derive(Clone)]
pub struct App {
    pub conf_file: String,
    pub netd_file: String,
    pub store: Arc<RwLock<Value>>,
    pub users: Users,
    pub firmware: Firmware,
    pub validator: Validator,
    pub event: Event,
    pub gpio: Option<Arc<RwLock<GPIO>>>,
}

impl App {
    pub fn new(
        conf_file: &str,
    ) -> Result<App, Box<dyn std::error::Error>> {
        
        // Read configuration file
        let content = match fs::read(conf_file) {
                Ok(v) => v,
                Err(e) => return Err(format!("Failed to read file: {}m err: {}", conf_file, e).into()),
            };
        
        let conf: Conf = serde_json::from_slice(&content)?;

        // Read configuration from file
        let content = fs::read(conf.data_file)?;

        // Desrialize data to Json Value
        let store = Arc::new(RwLock::new(serde_json::from_slice(&content)?));
        let users = Users::new(&conf.user_file)?;
        let firmware = Firmware::new(&conf.firmware_directory);
        let validator = Validator::new(&conf.validator_file)?;
        let mut event = Event::new();
        let mut gpio = None;
        if conf.device.to_lowercase() == "8AiDio1Ro_RS485".to_lowercase() {
            gpio = Some(Arc::new(RwLock::new(GPIO::new(store.clone(), &mut event)?)));
        }

        Ok(App {
            conf_file: conf_file.into(),
            store,
            users,
            firmware,
            validator,
            event,
            gpio,
            netd_file: conf.netd_file.into(),
        })
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut store = self.store.read().unwrap().clone();
        json_remove_status_map(&mut store);
        fs::write(&self.conf_file, serde_json::to_string_pretty(&store)?)?;
        generate_netd_conf(self, &store)?;

        Ok(())
    }

    pub fn reboot(&self) {
        let _r = self.save();
        let _r = Command::new("reboot").spawn();
    }

    pub fn merge_data(
        app: Arc<RwLock<App>>,
        pointer: String,
        j: &Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let app = app.write().unwrap();
        let store = &mut app.store.write().unwrap();
        let d = match store.pointer_mut(pointer.as_str()) {
            Some(v) => v,
            None => {
                return Err(format!("path {} is not accessible", pointer.clone()).into());
            }
        };

        App::json_merge(&app, pointer, d, j);

        Ok(())
    }

    fn json_merge(app: &App, pointer: String, a: &mut Value, b: &Value) {
        if let Value::Object(a) = a {
            if let Value::Object(b) = b {
                for (k, v) in b {
                    let p = format!("{}/{}", &pointer, &k);
                    if v.is_null() {
                        a.remove(k);
                    } else {
                        App::json_merge(app, p, a.entry(k).or_insert(Value::Null), v);
                    }
                }

                return;
            }
        }

        crate::app::Event::emit(app, pointer, b);
        *a = b.clone();
    }
}

fn json_remove_status_map(store: &mut Value) {
    if let Value::Object(o) = store {
        for (k, v) in o {
            if k == "status" {
                *v = json!(null);
                continue;
            } else if v.is_object() {
                json_remove_status_map(v);
            }
        }
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

fn generate_netd_conf(app: &App, store: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let wwan_enabled = match store.pointer("/net/wwan/enabled") {
        Some(j) => {
            if let Some(v) = j.as_bool() {
                v
            } else {
                false
            }
        }
        None => false,
    };

    let wwan_apn = match store.pointer("/net/wwan/apn") {
        Some(j) => {
            if let Some(v) = j.as_str() {
                v
            } else {
                "internet"
            }
        }
        None => "internet",
    };

    let lan_static = match store.pointer("/net/lan/static") {
        Some(j) => {
            if let Some(v) = j.as_bool() {
                v
            } else {
                false
            }
        }
        None => false,
    };

    let lan_address = match store.pointer("/net/lan/address") {
        Some(j) => {
            if let Some(v) = j.as_str() {
                v
            } else {
                "192.168.2.127"
            }
        }
        None => "192.168.2.127",
    };

    let lan_netmask = match store.pointer("/net/lan/netmask") {
        Some(j) => {
            if let Some(v) = j.as_str() {
                v
            } else {
                "255.255.255.0"
            }
        }
        None => "255.255.255.0",
    };

    let lan_gateway = match store.pointer("/net/lan/gateway") {
        Some(j) => {
            if let Some(v) = j.as_str() {
                v
            } else {
                "192.168.2.1"
            }
        }
        None => "192.168.2.1",
    };

    let modem = if wwan_enabled {
        json!({
            "type": "eg25g",
            "device": {
                "path": "/sys/devices/platform/ahb/b0015000.usb/usb1/1-2/1-2:1.4",
                "enable": "gpiodset pcie_power on",
                "disable": "gpiodset pcie_power off",
                "raw_ip": true
            },
            "configurations": {
                "operator": {
                        "apn": wwan_apn,
                        "ping": {
                                "host": "1.1.1.1",
                                "interval": 300,
                                "error": 3
                        }
                }
            }
        })
    } else {
        json!(null)
    };

    let wwan_priority = if wwan_enabled {
        json!({
            "interface": "wwan",
            "configuration": "operator",
            "priority": 1
        })
    } else {
        json!(null)
    };

    let lan_configuration = if lan_static { "static" } else { "dynamic" };
    let conf = json!({
        "interfaces": {
            "lan": {
                "type": "lan",
                "device": {
                        "name": "eth0"
                },
                "configurations": {
                    "dynamic": {
                        "mode": "dynamic"
                    },
                    "static": {
                        "mode": "static",
                        "netconf": {
                            "address": lan_address,
                            "netmask": lan_netmask,
                            "gateway": lan_gateway
                        }
                    }
                }
            },
            "wwan": modem,
        },
        "priorities": {
                "wwan": wwan_priority,
                "lan": {
                        "interface": "lan",
                        "configuration": lan_configuration,
                        "priority": 0
                }
        },
        "actions": {
                "up": {
                        "type": "up"
                },
                "down": {
                        "type": "down"
                }
        }
    });

    println!("netd conf\n{}", serde_json::to_string_pretty(&conf).unwrap_or("".to_owned()));

    fs::write(&app.netd_file, serde_json::to_string_pretty(&conf)?)?;

    Ok(())
}
