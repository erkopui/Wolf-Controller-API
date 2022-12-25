use gpio_cdev::{Chip, Line, LineHandle, LineRequestFlags};
use serde_json::Value;
use std::fs::{self, read_to_string};
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::sync::{Arc, RwLock};

static GPIO_INPUTS: [(&'static str, u32); 8] = [
    ("b0004040.nuc980-gpio", 6),
    ("b0004040.nuc980-gpio", 4),
    ("b0004040.nuc980-gpio", 7),
    ("b0004040.nuc980-gpio", 5),
    ("b0004140.nuc980-gpio", 1),
    ("b0004140.nuc980-gpio", 0),
    ("b0004140.nuc980-gpio", 2),
    ("b0004140.nuc980-gpio", 3),
];

static GPIO_OUTPUTS: [(&'static str, u32); 8] = [
    ("b0004100.nuc980-gpio", 12),
    ("b0004100.nuc980-gpio", 10),
    ("b0004140.nuc980-gpio", 5),
    ("b0004140.nuc980-gpio", 6),
    ("b0004140.nuc980-gpio", 9),
    ("b0004140.nuc980-gpio", 7),
    ("b0004140.nuc980-gpio", 8),
    ("b0004140.nuc980-gpio", 10),
];

static GPIO_PULLUPS: [(&'static str, u32); 8] = [
    ("74hc595", 0),
    ("74hc595", 1),
    ("74hc595", 2),
    ("74hc595", 3),
    ("74hc595", 4),
    ("74hc595", 5),
    ("74hc595", 6),
    ("74hc595", 7),
];

static RELAY: (&'static str, u32) = ("b0004140.nuc980-gpio", 4);

#[derive(Clone, PartialEq)]
enum GPIOMode {
    Input,
    Output,
    NTC,
}

#[derive(Clone)]
pub struct GPIOStatus {
    input: Arc<Vec<LineHandle>>,
    pullup: Arc<Vec<LineHandle>>,
    output: Arc<Vec<LineHandle>>,
    adc: Vec<String>,
    adc_beta: [u16; 8],
    adc_read_state: usize,
    pub relay: Arc<LineHandle>,
    relay_inverted: bool,
    inverted: [bool; 8],
    mode: [GPIOMode; 8],
}

#[derive(Clone)]
pub struct GPIO {
    pub status: GPIOStatus,
}

impl GPIO {
    pub fn new(
        store: Arc<RwLock<Value>>,
        event: &mut super::Event,
    ) -> Result<GPIO, Box<dyn std::error::Error>> {
        let mut store = store.write().unwrap();
        let mut input = Vec::new();

        // Input
        for i in 0..8 {
            input.push(gpio_get_line(GPIO_INPUTS[i])?.request(
                LineRequestFlags::INPUT,
                0,
                "wolf-app",
            )?);
        }

        // Inverted
        let mut inverted: [bool; 8] = [false; 8];
        for i in 0..8 {
            let p = format!("/io/conf/io{}/inverted", i + 1);
            if let Some(inv) = store.pointer(p.as_str()) {
                if let Some(val) = inv.as_bool() {
                    inverted[i] = val;
                }
            }
        }

        // Pull-up
        let mut pu = Vec::new();
        for i in 0..8 {
            pu.push(gpio_get_line(GPIO_PULLUPS[i])?.request(
                LineRequestFlags::OUTPUT,
                0,
                "wolf-app",
            )?);
        }
        for i in 0..8 {
            let p = format!("/io/conf/io{}/pullup", i + 1);
            let mut val = 0;
            if let Some(pullup) = store.pointer(p.as_str()) {
                if let Some(v) = pullup.as_bool() {
                    if v {
                        val = 1;
                    };
                }
            }
            pu[i].set_value(val)?;
        }

        // Output
        let mut out = Vec::new();
        for i in 0..8 {
            out.push(gpio_get_line(GPIO_OUTPUTS[i])?.request(
                LineRequestFlags::OUTPUT,
                0,
                "wolf-app",
            )?);
        }

        //Relay
        let relay_handle =
            gpio_get_line(RELAY)?.request(LineRequestFlags::OUTPUT, 0, "wolf-app")?;

        let mut relay_inverted = false;
        if let Some(_rel) = store.pointer("/io/conf/relay/inverted") {
            if let Some(v) = _rel.as_bool() {
                if v {
                    relay_inverted = true;
                }
            }
        }
        let mut _val = 0;
        if relay_inverted {
            _val ^= 1;
        }
        let _ = relay_handle.set_value(_val);
        store["io"]["status"]["relay"] = serde_json::json!(false);

        // ADC
        let iio_path = get_iio_adc_path()?;
        let mut adc: Vec<String> = Vec::new();
        for i in 0..8 {
            let file = format!("{}/in_voltage{}_raw", iio_path, i);
            adc.push(file);
        }
        let mut adc_beta: [u16; 8] = [3435; 8];
        for i in 0..8 {
            let _p = format!("/io/conf/io{}/beta", i + 1);
            if let Some(_betaj) = store.pointer(_p.as_str()) {
                if let Some(beta) = _betaj.as_u64() {
                    adc_beta[i] = beta as u16;
                }
            }
        }

        // GPIO Mode
        let mut mode: [GPIOMode; 8] = [
            GPIOMode::Input,
            GPIOMode::Input,
            GPIOMode::Input,
            GPIOMode::Input,
            GPIOMode::Input,
            GPIOMode::Input,
            GPIOMode::Input,
            GPIOMode::Input,
        ];

        for i in 0..8 {
            let _p = format!("/io/conf/io{}/mode", i + 1);
            if let Some(_modej) = store.pointer(_p.as_str()) {
                if let Some(_mode) = _modej.as_str() {
                    match _mode {
                        "output" => {
                            mode[i] = GPIOMode::Output;
                            let mut _val = 0;
                            if inverted[i] {
                                _val = 1;
                            }
                            out[i].set_value(_val)?;
                            store["io"]["status"][format!("out{}", i + 1).as_str()] =
                                serde_json::json!(cast_as_bool(_val));
                        },
                        "ntc" => mode[i] = GPIOMode::NTC,
                        _ => mode[i] = GPIOMode::Input,
                    }
                }
            }
        }

        event
            .subscribe("/io/status/relay".to_string(), Arc::new(GPIO::output_set))
            .subscribe("/io/status/out1".to_string(), Arc::new(GPIO::output_set))
            .subscribe("/io/status/out2".to_string(), Arc::new(GPIO::output_set))
            .subscribe("/io/status/out3".to_string(), Arc::new(GPIO::output_set))
            .subscribe("/io/status/out4".to_string(), Arc::new(GPIO::output_set))
            .subscribe("/io/status/out5".to_string(), Arc::new(GPIO::output_set))
            .subscribe("/io/status/out6".to_string(), Arc::new(GPIO::output_set))
            .subscribe("/io/status/out7".to_string(), Arc::new(GPIO::output_set))
            .subscribe("/io/status/out8".to_string(), Arc::new(GPIO::output_set));

        Ok(GPIO {
            status: GPIOStatus {
                input: Arc::new(input),
                inverted,
                pullup: Arc::new(pu),
                output: Arc::new(out),
                relay: Arc::new(relay_handle),
                relay_inverted,
                adc,
                adc_beta,
                adc_read_state: 0,
                mode,
            },
        })
    }

    pub fn poll(&self, store: &Arc<RwLock<Value>>) {
        let mut store = store.write().unwrap();
        for i in 0..8 {
            if self.status.mode[i] != GPIOMode::Input {
                continue;
            }
            if let Ok(v) = self.status.input[i].get_value() {
                let mut bool = true;
                if v == 0 {
                    bool = false;
                }
                if self.status.inverted[i] {
                    bool = !bool;
                }
                store["io"]["status"][format!("in{}", (i + 1).to_string())] =
                    serde_json::json!(bool)
            }
        }
    }

    pub fn adc_poll(&mut self, store: &Arc<RwLock<Value>>) {
        let mut store = store.write().unwrap();

        // NTC Mode - turn pullup on
        if self.status.mode[self.status.adc_read_state] == GPIOMode::NTC {
            let _r = self.status.pullup[self.status.adc_read_state].set_value(1);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let f = match read_to_string(&self.status.adc[self.status.adc_read_state]) {
            Ok(f) => f,
            Err(_) => return,
        };
        match f.trim().parse::<f64>() {
            Ok(v) => {
                let mut voltage = v * 0.002679199218_f64;
                if self.status.mode[self.status.adc_read_state] == GPIOMode::Input {
                    store["io"]["status"]
                        [format!("adc{}", (self.status.adc_read_state + 1).to_string())] =
                        serde_json::json!(voltage);
                } else if self.status.mode[self.status.adc_read_state] == GPIOMode::NTC {
                    // https://www.jameco.com/Jameco/workshop/TechTip/temperature-measurement-ntc-thermistors.html
                    if voltage > 10.0 {
                        voltage = 10.0;
                    }
                    let temperature = 1.0_f64
                        / (((10.0_f64 / (10.0 - voltage) - 1.0_f64).ln() * 1.0_f64 / self.status.adc_beta[self.status.adc_read_state] as f64)
                            + (1.0_f64 / 298.15_f64))
                        - 273.15_f64;
                    store["io"]["status"][format!(
                        "adc{}",
                        (self.status.adc_read_state + 1).to_string()
                    )] = serde_json::json!(temperature);
                    // NTC Mode - turn pullup off
                    let _r = self.status.pullup[self.status.adc_read_state].set_value(0);
                }
            }
            Err(_) => return,
        }

        self.status.adc_read_state += 1;
        if self.status.adc_read_state > 7 {
            self.status.adc_read_state = 0;
        }
    }

    fn output_set(app: &super::App, path: &str, j: &Value) {
        let mut val: u8 = match j.as_bool() {
            Some(v) => {
                let mut n = 0;
                if v {
                    n = 1;
                }
                n
            }
            _ => return,
        };

        
        let r = &app.gpio.as_ref().unwrap().read().unwrap().status;

        match path {
            "/io/status/out1" => {
                if r.mode[0] != GPIOMode::Output {
                    return;
                }
                if r.inverted[0] {
                    val ^= 1;
                }
                let _ = r.output[0].set_value(val);
            }
            "/io/status/out2" => {
                if r.mode[1] != GPIOMode::Output {
                    return;
                }
                if r.inverted[1] {
                    val ^= 1;
                }
                let _ = r.output[1].set_value(val);
            }
            "/io/status/out3" => {
                if r.mode[2] != GPIOMode::Output {
                    return;
                }
                if r.inverted[2] {
                    val ^= 1;
                }
                let _ = r.output[2].set_value(val);
            }
            "/io/status/out4" => {
                if r.mode[3] != GPIOMode::Output {
                    return;
                }
                if r.inverted[3] {
                    val ^= 1;
                }
                let _ = r.output[3].set_value(val);
            }
            "/io/status/out5" => {
                if r.mode[4] != GPIOMode::Output {
                    return;
                }
                if r.inverted[4] {
                    val ^= 1;
                }
                let _ = r.output[4].set_value(val);
            }
            "/io/status/out6" => {
                if r.mode[5] != GPIOMode::Output {
                    return;
                }
                if r.inverted[5] {
                    val ^= 1;
                }
                let _ = r.output[5].set_value(val);
            }
            "/io/status/out7" => {
                if r.mode[6] != GPIOMode::Output {
                    return;
                }
                if r.inverted[6] {
                    val ^= 1;
                }
                let _ = r.output[6].set_value(val);
            }
            "/io/status/out8" => {
                if r.mode[7] != GPIOMode::Output {
                    return;
                }
                if r.inverted[7] {
                    val ^= 1;
                }
                let _ = r.output[7].set_value(val);
            }
            "/io/status/relay" => {
                if r.relay_inverted {
                    val ^= 1;
                }
                let _ = r.relay.set_value(val);
            }
            _ => {}
        }
    }
}

fn cast_as_bool(val: u8) -> bool {
    if val > 0 {
        return true;
    }
    false
}

fn gpio_get_line(gpio: (&str, u32)) -> Result<Line, Box<dyn std::error::Error>> {
    let mut name = String::new();

    let chip_iterator = match gpio_cdev::chips() {
        Ok(chips) => chips,
        Err(e) => return Err(format!("Failed to get chip iterator {:?}", e).into()),
    };

    for chip in chip_iterator {
        let chip = match chip {
            Ok(chip) => chip,
            Err(err) => return Err(format!("Failed to open the chip: {:?}", err).into()),
        };
        if chip.label() == gpio.0 {
            name.push_str(&chip.path().to_string_lossy());
            break;
        }
    }

    if name.len() == 0 {
        return Err(format!("failed to find the chip: {}", gpio.0).into());
    }

    let mut chip = match Chip::new(name) {
        Ok(chip) => chip,
        Err(e) => return Err(format!("Failed to open chip, {}", e).into()),
    };

    let line = chip.get_line(gpio.1)?;

    Ok(line)
}

fn get_iio_adc_path() -> Result<String, Box<dyn std::error::Error>> {
    let dir_root = Path::new("/sys/bus/iio/devices");
    for entry in fs::read_dir(dir_root)? {
        let entry = entry?;
        //let path = entry.path().push(Path::new("/name"));
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let mut path_file = path.clone();
        path_file.push("name");
        if let Ok(content) = read_to_string(&path_file) {
            if content.trim() == "adc108s102" {
                match path.to_str() {
                    Some(v) => return Ok(v.to_string()),
                    None => return Err(format!("Failed to parse OsString").into()),
                }
            }
        }
    }

    Err(Error::new(ErrorKind::NotFound, "path not found!").into())
}
