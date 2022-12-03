use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserData {
    pub username: String,
    pub password: String,
    pub name: Option<String>,
    pub one_time_password: Option<bool>,
}

#[derive(Clone)]
pub struct User {
    pub user: UserData,
    file_path: String,
}

impl User {
    pub fn new(user_file: &str) -> Result<User, Box<dyn std::error::Error>> {
        // Read user file
        let content =
            fs::read(user_file).expect(format!("Failed to read file: {}", user_file).as_str());

        // Desrialize data to Json Value
        Ok(User {
            user: serde_json::from_slice(&content)?,
            file_path: user_file.into(),
        })
    }

    fn hash(&self, user: &str, password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(user);
        hasher.update(":");
        hasher.update(password);
        format!("{:x}", hasher.finalize())
    }

    pub fn update(&mut self, mut user: UserData) -> Result<(), Box<dyn std::error::Error>> {
        user.password = self.hash(&user.username, &user.password);
	self.user = user.clone();
	fs::write(&self.file_path, serde_json::to_vec_pretty(&user)?)?;

	Ok(())
    }

    pub fn is_user_valid(&self, user: &str, password: &str) -> bool {
        if self.hash(user, password) == self.user.password {
            return true;
        }
        return false;
    }
}
