use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct User{
    pub nickname: String,
    pub password: String,
    pub folder: String,
}

pub struct DataBase{
    pub users: HashMap<String, User>,
}

impl DataBase{
    pub fn new() -> Self{
        DataBase{
            users: HashMap::new(),
        }
    }
    
    pub fn add_user(&mut self, nickname: String, password: String, folder: String) {
        self.users.insert(nickname.clone(), User { nickname, password, folder });
    }

    pub fn auth_user(&self, username: &str, password: &str) -> bool{
        if let Some(user) = self.users.get(username) {return user.password == password;} 
        else{return false;}
    }
}