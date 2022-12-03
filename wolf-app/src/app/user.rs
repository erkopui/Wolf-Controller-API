use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
struct User {
    username: String,
    password: String,
}

#[derive(Clone)]
pub struct Users {
    user: Vec<User>,
}

impl Users {
    pub fn new(user_file: &str) -> Result<Users, Box<dyn std::error::Error>> {
        // Read user file
        let content = fs::read(user_file).expect(format!("Failed to read file: {}", user_file).as_str());

        // Desrialize data to Json Value
        let data: Vec<User> = serde_json::from_slice(&content)?;
        Ok(Users { user: data })
    }

    pub fn is_user_valid(&self, user: &str, password: &str) -> bool {
	for u in &self.user {
		if user == u.username && password == u.password {
			return true
		}
	}
        return false;
    }
}
