use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use std::str::FromStr;
use anyhow::Result;
use parking_lot::Mutex;
use uuid::Uuid;

use crate::messagesPool::{PoolMessage, MessagesPool};
use crate::state::UserData;
use crate::types::{
  Authoritation, 
  SignalsData, 
  SignalsHeader, 
  SignalError,
  Signal
};

use super::manager::Manager;
use super::streamManager::StreamManager;

pub trait DataManager {
  fn deny_auth(&mut self) -> Result<()>;
  fn auth(&mut self, signal: String) -> Result<()>;
  fn remove_user(&mut self, username: String) -> Result<()>;
  fn process_messages_pool(&mut self, receiver: Receiver<()>) -> Result<()>;
  fn process_incoming_message(messages_pool: Arc<Mutex<MessagesPool>>, signal: String) -> Result<()>;
}

impl DataManager for Manager {
  fn deny_auth(&mut self) -> Result<()> {
    let response = SignalsData::new(
      vec![SignalsHeader::auth(Authoritation::Denied)],
      None
    );

    self.send_data(&response.to_string())?;
    Ok(())
  }

  fn auth(&mut self, signal: String) -> Result<()> {
    let data = SignalsData::from_str(&signal)?;

    match data.signalType.unwrap() {
        Signal::Connection => {
          if let None = data.username {
            return Err(SignalError.into());
          }
          let mut state = self.state.get();
          if state.users.contains_key(&data.username.clone().unwrap()) {
            return Err(SignalError.into())
          }
          state.users.insert(data.username.clone().unwrap().to_owned(), UserData {
            address: self.stream.peer_addr()?.to_string(),
          });
          self.messages_pool.lock().push(PoolMessage {
            id: Uuid::new_v4().to_string(),
            username: String::new(),
            message: format!("{} joined the chat!", data.username.clone().unwrap()),
            from_server: true
          });
        }
        _ => return Err(SignalError.into()),
    }

    self.connected_user_username = Some(data.username.unwrap());

    let response = SignalsData::new(
      vec![SignalsHeader::auth(Authoritation::Accepted)],
      None
    );

    self.send_data(&response.to_string())?;
    Ok(())
  }

  fn remove_user(&mut self, username: String) -> Result<()> {
    let mut state = self.state.get();

    if state.users.contains_key(&username) {
      state.users.remove(&username);
      self.messages_pool.lock().push(PoolMessage {
        id: Uuid::new_v4().to_string(),
        username: String::new(),
        message: format!("{username} left the chat!"),
        from_server: true
      });
    }
    Ok(())
  }

  fn process_messages_pool(&mut self, receiver: Receiver<()>) -> Result<()> {
    loop {
      if let Ok(()) = receiver.try_recv() {
        break;
      };

      let lock_ref = self.messages_pool.clone();
      let pool_lock = lock_ref.lock();

      let messages = pool_lock.has_new(&self.last_read_message_id);
      if let Some(v) = messages {
        if let Some(last) = v.1 {
          self.last_read_message_id = last;
        }
        for message in v.0 {
          let mut syg_vec = vec![
            SignalsHeader::signalType(Signal::Message),
            SignalsHeader::username(message.username.clone()),
            SignalsHeader::withMess
          ];
          if message.from_server {
            syg_vec.push(SignalsHeader::serverMess);
          }
          let response = SignalsData::new(syg_vec, Some(&message.message));
          self.send_data(&response.to_string())?;
        }
      }
      thread::sleep(Duration::from_millis(10));
    }

    Ok(())
  }

  fn process_incoming_message(messages_pool: Arc<Mutex<MessagesPool>>, signal: String) -> Result<()> {
    let data = SignalsData::from_str(&signal)?;
  
    if !data.withMess || data.username.is_none() {
      return Err(SignalError.into())
    }
  
    messages_pool.lock().push(PoolMessage {
      id: Uuid::new_v4().to_string(),
      username: data.username.clone().unwrap(),
      message: data.message.clone().unwrap().trim().to_owned(),
      from_server: false
    });
  
    Ok(())
  }
}