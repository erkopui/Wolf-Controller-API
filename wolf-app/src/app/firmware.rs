use axum::body::Bytes;
use std::fs;

#[derive(Clone)]
pub struct Firmware {
    dir: String,
}

impl Firmware {
    pub fn new(file_path: &str) -> Firmware {
        Firmware {
            dir: file_path.into(),
        }
    }

    pub fn save(&self, filename: &str, data: Bytes) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = self.dir.clone();
        file.push_str(filename);
        match fs::write(&file, data) {
            Ok(_) => return Ok(()),
            Err(e) => return Err(format!("Failed to write file {} to disk : {}", &file, e).into()),
        }
    }
}
