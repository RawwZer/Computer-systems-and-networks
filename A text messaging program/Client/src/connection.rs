use std::{
    net::TcpStream, 
    io::{
      self, 
      Write,
      Error, 
      ErrorKind, 
      BufRead, 
      BufReader
    },
  };
  use crate::types::{
    Signal, 
    SignalsHeader, 
    SignalsData,
    Authoritation
  };
  
  pub struct Connection {
    pub stream: TcpStream,
    reader: io::BufReader<TcpStream>
  }
  
  impl Connection {
    pub fn new(address: &str, username: &str) -> io::Result<Connection> {
      let signal = SignalsData::new(
        vec![
          SignalsHeader::signalType(Signal::Connection),
          SignalsHeader::username(username.to_owned())
        ],
        None
      );
      
      // try to connect to the address
      let mut connection = TcpStream::connect(address)?;
      // sending to the server
      connection.write_all(signal.to_string().as_bytes())?;
      let reader = BufReader::new(connection.try_clone()?);
  
      let mut instance = Connection {
        stream: connection,
        reader
      };
  
      let data_from_socket = instance.readSignal()?;
      if data_from_socket.contains(&Authoritation::Denied.to_string()) {
        return Err(Error::new(ErrorKind::ConnectionAborted, "Access denied"));
      }
    
      return Ok(instance)
    }
  
    pub fn readSignal(&mut self) -> io::Result<String> {
      let mut res_line = String::new();
      let mut isHeadersRead = false;
      loop {
        let mut buf_line = String::new();
        match self.reader.read_line(&mut buf_line) {
          Err(e) => panic!("Got an error: {}", e),
          Ok(0) => return Err(Error::new(ErrorKind::BrokenPipe, "Connection closed")),
          Ok(_) => (),
        };
        res_line.push_str(&buf_line);
    
        if res_line.ends_with("\r\n\r\n"){
          if !res_line.contains(&SignalsHeader::withMess.to_string()) || isHeadersRead {
            break;
          }
          isHeadersRead = true;
        }
      }
    
      Ok(res_line)
    }
  }
  
  impl Clone for Connection {
    fn clone(&self) -> Self {
      Connection { 
        stream: self.stream.try_clone().unwrap(), 
        reader: BufReader::new(self.stream.try_clone().unwrap())
      }
    }
  }