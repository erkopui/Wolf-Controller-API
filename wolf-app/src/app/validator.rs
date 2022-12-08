use serde::Deserialize;
use serde_json::{map::Map, Value};
use std::{collections::HashMap, fs};

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ValidatorDataType {
    String,
    Number,
    Boolean,
}

#[derive(Clone, Deserialize, Debug)]
pub struct ValidatorDataRequires {
    path: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct ValidatorData {
    #[serde(alias = "type")]
    data_type: ValidatorDataType,
    requires: Option<Vec<ValidatorDataRequires>>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Validator {
    #[serde(flatten)]
    pub map: HashMap<String, ValidatorData>,
}

impl Validator {
    pub fn new(file: &str) -> Result<Validator, Box<dyn std::error::Error>> {
        let content = match fs::read(file) {
            Ok(f) => f,
            Err(e) => return Err(format!("Failed to read file {} : {}", file, e).into()),
        };

        Ok(Validator {
            map: match serde_json::from_slice(&content) {
                Ok(m) => {
                    m
                }
                Err(e) => return Err(format!("Failed to parse validator file: {}", e).into()),
            },
        })
    }

    fn validate_data_type(
        &self,
        validator: &ValidatorData,
        data: &Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Checking data type
        match validator.data_type {
            ValidatorDataType::String => {
                if !data.is_string() {
                    return Err(format!("data type mismatch, required string").into());
                }
            }
            ValidatorDataType::Number => {
                if !data.is_number() {
                    return Err(format!("data type mismatch, required number").into());
                }
            }
            ValidatorDataType::Boolean => {
                if !data.is_boolean() {
                    return Err(format!("data type mismatch, required boolean").into());
                }
            }
        }

        Ok(())
    }

    pub fn validate(
        &self,
        root_pointer: &String,
        model: &Value,
        new_req: &Value,
        iter: Option<&Map<String, Value>>,
        mut iter_pointer: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let j_obj = match iter {
            Some(v) => v,
            None => match new_req.as_object() {
                Some(v) => {
                    iter_pointer = Some(root_pointer.clone());
                    v
                }
                None => {
                    if let Some(v) = self.map.get(&root_pointer.clone()) {
                        match self.validate_data_type(v, new_req) {
                            Ok(_) => return Ok(()),
                            Err(e) => return Err(e),
                        }
                    }
                    // Nothing to validate
                    return Ok(());
                }
            },
        };

        for (k, v) in j_obj {
            let p = format!("{}/{}", iter_pointer.as_ref().unwrap(), k);
            if v.is_object() {
                match self.validate(root_pointer, model, new_req, v.as_object(), Some(p)) {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            } else {
                // Validate type
                if let Some(v_map) = self.map.get(&p) {
                    match self.validate_data_type(v_map, v) {
                        Ok(_) => {}
                        Err(e) => return Err(format!("{}, path {}", e, p).into()),
                    }

                    //checkint required paths
                    if let Some(r_map) = &v_map.requires {
                        // from request
                        for r in r_map {
                            if r.path.starts_with(root_pointer) {
                                let mut _p = r.path.clone();
                                _p.replace_range(..root_pointer.len(), "");
                                match new_req.pointer(_p.as_str()) {
                                    Some(_) => continue,
                                    None => {}
                                }
                            }

                            // from model
                            match model.pointer(r.path.as_str()) {
                                Some(_) => continue,
                                None => {
                                    return Err(format!(
                                        "item {} requires {} which is missing",
                                        p, r.path
                                    )
                                    .into())
                                }
                            }
                        }
                    }
                }
            }
        }
        return Ok(());
    }
}
