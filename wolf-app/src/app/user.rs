use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, collections::HashMap};

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserData {
    pub password: String,
    pub name: Option<String>,
    pub one_time_password: Option<bool>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
	#[serde(flatten)]
	pub user: HashMap<String, UserData>,
}

#[derive(Clone)]
pub struct Users {
    pub users: User,
    file: String,
}

impl Users{
    pub fn new(user_file: &str) -> Result<Users, Box<dyn std::error::Error>> {
        // Read user file
        let content =
            fs::read(user_file).expect(format!("Failed to read file: {}", user_file).as_str());

        // Desrialize data to Json Value
        Ok(Users {
            users: serde_json::from_slice(&content)?,
            file: user_file.into(),
        })
    }

    fn hash(&self, user: &str, password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(user);
        hasher.update(":");
        hasher.update(password);
        format!("{:x}", hasher.finalize())
    }

    pub fn update(&mut self, user: User) -> Result<(), Box<dyn std::error::Error>> {

	let mut u: HashMap<String, UserData> = HashMap::new(); 
	for (username ,data) in &user.user {
		let mut d = data.clone();
		d.password = self.hash(username, &d.password);
		u.insert(username.clone(), d);
	}
	
	self.users.user = u.clone();
	fs::write(&self.file, serde_json::to_vec_pretty(&u)?)?;
	Ok(())
    }

    pub fn is_user_valid(&self, user: &str, password: &str) -> bool {
	if let Some(d) =  self.users.user.get(user) {
		if self.hash(user, password) == d.password {
			return true;
		}
	}
 
	return false;
    }
}
