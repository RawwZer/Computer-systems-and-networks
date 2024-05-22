use std::{error::Error, fmt, str::FromStr};

// ----- Error type -----
#[derive(Debug, Clone)]
pub struct SignalError;
impl Error for SignalError{}
impl fmt::Display for SignalError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid signal data")
    }
}

// ----- Signals type -----
#[derive(Debug, Copy, Clone)]
pub enum Signal{
    Connection,
    Message,
}

impl FromStr for Signal{
    type Err = SignalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CONNECTION" => Ok(Signal::Connection),
            "MESSAGE" => Ok(Signal::Message),
            _ => Err(SignalError)
        }
    }
}

impl ToString for Signal{
    fn to_string(&self) -> String{
        match self {
            Signal::Connection => "CONNECTION".to_owned(),
            Signal::Message => "MESSAGE".to_owned(),
        }
    }
}

// ----- Authoritation type -----
#[derive(Debug, Copy, Clone)]
pub enum Authoritation{
    Accepted,
    Denied,
}

impl FromStr for Authoritation{
    type Err = SignalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ACCEPTED" => Ok(Authoritation::Accepted),
            "DENIED" => Ok(Authoritation::Denied),
            _ => Err(SignalError)
        }
    }
}

impl ToString for Authoritation{
    fn to_string(&self) -> String{
        match self {
            Authoritation::Accepted => "ACCEPTED".to_owned(),
            Authoritation::Denied => "DENIED".to_owned(),
        }
    }
}

// ----- Signal's header type -----
pub enum SignalsHeader{
    username(String), 
    key(String),
    auth(Authoritation),
    signalType(Signal),
    withMess,
    serverMess,
}

impl FromStr for SignalsHeader {
    type Err = SignalError;
  
    fn from_str(s: &str) -> Result<Self, Self::Err> {
      let (header, value) = s.split_once(':').unwrap_or((s, s));
  
      match header {
        "USERNAME" => Ok(SignalsHeader::username(value.trim().to_owned())),
        "KEY" => Ok(SignalsHeader::key(value.trim().to_owned())),
        "AUTH_STATUS" => {
          match Authoritation::from_str(value.trim()) {
            Ok(v) => return Ok(SignalsHeader::auth(v)),
            Err(_) => Err(SignalError)
          }
        },
        "SIGNAL_TYPE" => {
          match Signal::from_str(value.trim()) {
            Ok(v) => return Ok(SignalsHeader::signalType(v)),
            Err(_) => Err(SignalError)
          }
        }
        "WITH_MESSAGE" => Ok(SignalsHeader::withMess),
        "SERVER_MESSAGE" => Ok(SignalsHeader::serverMess),
        _ => Err(SignalError)
      }
    }
  }
  
impl ToString for SignalsHeader {
    fn to_string(&self) -> String {
      match self {
        SignalsHeader::username(v) => format!("USERNAME: {v}\r\n"),
        SignalsHeader::key(v) => format!("KEY: {v}\r\n"),
        SignalsHeader::auth(v) => format!("AUTH_STATUS: {}\r\n", v.to_string()),
        SignalsHeader::signalType(v) => format!("SIGNAL_TYPE: {}\r\n", v.to_string()),
        SignalsHeader::withMess => "WITH_MESSAGE\r\n".to_owned(),
        SignalsHeader::serverMess => "SERVER_MESSAGE\r\n".to_owned()
      }
    }
}
  
// ----- Signal's data type -----
#[derive(Debug, Clone)]
pub struct SignalsData {
    pub username: Option<String>,
    pub key: Option<String>,
    pub auth: Option<Authoritation>,
    pub signalType: Option<Signal>,
    pub withMess: bool,
    pub message: Option<String>,
    pub serverMess: bool
}

impl SignalsData {
    pub fn new(headers: Vec<SignalsHeader>, message: Option<&str>) -> SignalsData {
      let mut data = SignalsData {
        username: None,
        key: None,
        auth: None,
        signalType: None,
        withMess: false,
        message: None,
        serverMess: false
      };
  
      for header in headers {
        match header {
          SignalsHeader::username(v) => {
            data.username = Some(v);
          },
          SignalsHeader::key(v) => {
            data.key = Some(v);
          },
          SignalsHeader::auth(v) => {
            data.auth = Some(v);
          },
          SignalsHeader::signalType(v) => {
            data.signalType = Some(v);
          },
          SignalsHeader::withMess => {
            data.withMess = true;
            data.message = Some(message.unwrap_or("").to_owned());
          },
          SignalsHeader::serverMess => {
            data.serverMess = true;
          }
        }
      }
  
      data
    }
  }
  
  impl FromStr for SignalsData {
    type Err = SignalError;
  
    fn from_str(s: &str) -> Result<Self, Self::Err> {
      let mut data = SignalsData { 
        username: None, 
        key: None, 
        auth: None, 
        signalType: None,
        withMess: false,
        message: None,
        serverMess: false,
      };
      let splitted = s.split("\r\n");
      for string in splitted {
        let header = match SignalsHeader::from_str(string) {
          Ok(v) => v,
          Err(_) => continue
        };
  
        match header {
          SignalsHeader::username(v) => {
            data.username = Some(v);
          },
          SignalsHeader::key(v) => {
            data.key = Some(v);
          },
          SignalsHeader::auth(v) => {
            data.auth = Some(v);
          },
          SignalsHeader::signalType(v) => {
            data.signalType = Some(v);
          }
          SignalsHeader::withMess => {
            data.withMess = true;
          },
          SignalsHeader::serverMess => {
            data.serverMess = true;
          }
        }
      }
  
      if data.withMess {
        let splitted = s.split_once("\r\n\r\n");
        if let Some(v) = splitted {
          if v.1.ends_with("\r\n\r\n") {
            let string = v.1.to_owned();
            data.message = Some(string[..string.len() - 4].to_owned());
          }
          else {
            data.message = Some(v.1.to_owned());
          }
        }
        else {
          return Err(SignalError);
        }
      }
  
      if let None = data.signalType {
        return Err(SignalError)
      }
  
      Ok(data)
    }
  }
  
  impl ToString for SignalsData {
    fn to_string(&self) -> String {
      let mut res_str = String::new();
  
      if let Some(v) = &self.username {
        res_str.push_str(&SignalsHeader::username(v.to_owned()).to_string());
      }
      if let Some(v) = &self.key {
        res_str.push_str(&SignalsHeader::key(v.to_owned()).to_string());
      }
      if let Some(v) = &self.auth {
        res_str.push_str(&SignalsHeader::auth(v.clone()).to_string());
      }
      if let Some(v) = &self.signalType {
        res_str.push_str(&SignalsHeader::signalType(v.clone()).to_string());
      }
      if self.serverMess {
        res_str.push_str(&SignalsHeader::serverMess.to_string());
      }
      if self.withMess {
        if let Some(v) = &self.message {
          res_str.push_str(&SignalsHeader::withMess.to_string());
          res_str.push_str("\r\n");
          res_str.push_str(&v);
        }
      }
      res_str.push_str("\r\n\r\n");
  
      res_str
    }
  }