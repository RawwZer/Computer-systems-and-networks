use std::{
    thread, 
    io::{self, Write},
    str::FromStr
  };
use crossterm::{event::{self, Event, KeyCode}, execute, style::{Attribute, Color, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor}, terminal::{self, enable_raw_mode, ClearType}};

use crate::{
    settings::Settings, 
    state::State, 
    connection::Connection, 
    types::{
      Signal, 
      SignalsData, 
      SignalsHeader
    }
  };
  
pub struct Service {
    pub connection: Connection,
    pub settings: Settings,
    pub state: State,
  }
  
impl Service {
    pub fn run(settings: Settings, state: State) -> io::Result<()> {
      let connection = Connection::new(
        &settings.server_address.to_owned(), 
        &state.username
      )?;
  
      let mut instance = Service {
        connection,
        settings,
        state
      }.enable_print();
  
      instance.proccess_incoming_messages();
      instance.read_inputs();
  
      Ok(())
    }

    pub fn proccess_incoming_messages(&self) {
      let messages = self.state.messagesThr.clone();
      let tx = self.state.chatReloadTX.clone();
      let mut connection = self.connection.clone();
      thread::spawn(move || -> io::Result<()> {
        loop {
          let data_from_socket = match connection.readSignal() {
            Ok(v) => v,
            Err(_) => break
          };
          let signal = SignalsData::from_str(&data_from_socket);
          let mut messages = messages.lock();
          if let Ok(s) = signal {
            if let Some(Signal::Message) = s.signalType {
              if s.serverMess {
                messages.push(
                  format!(
                    "{}{}{}{}",
                    // termion::style::Faint,
                    // termion::style::Bold,
                    SetAttribute(Attribute::Dim),
                    SetAttribute(Attribute::Bold),
                    s.message.unwrap(),
                    // termion::style::Reset,
                    ResetColor,
                  )
                );
              }
              else {
                messages.push(
                  format!(
                    "<{}> {}", 
                    s.username.unwrap(), 
                    s.message.unwrap()
                  )
                );
              }
            }
          }
          match tx.send(()) {
            Ok(_) => {},
            Err(_) => break
          };
        }
    
        Ok(())
      });
    }
  
    pub fn enable_print(self) -> Service {
      let rx = self.state.chatReloadRX.unwrap();
      let messages = self.state.messagesThr.clone();
      let user_input = self.state.userInp.clone();
      let username = self.state.username.clone();
  
      thread::spawn(move || -> io::Result<()> {
        loop {
          match rx.recv() {
            Ok(()) => {},
            Err(_) => break
          };

          execute!(
            io::stdout(),
            terminal::Clear(ClearType::All),
          )?;

          for (index, m) in messages.lock().iter().enumerate() {
            if index == 0 {
              print!("\r\n{m}\r\n");
            }
            else {
              print!("{m}\r\n");
            }
          }
          let input = user_input.lock().clone();
          print!(
            "{}{}{} >{} {}", 
            SetBackgroundColor(Color::White),
            SetForegroundColor(Color::Black),
            username, 
            ResetColor,
            input
          );
  
          std::io::stdout().flush()?;
        }
        Ok(())
      });
  
      Service { 
        connection: self.connection,
        settings: self.settings, 
        state: State {
          username: self.state.username.clone(),
          chatReloadRX: None,
          chatReloadTX: self.state.chatReloadTX.clone(),
          userInp: self.state.userInp.clone(),
          messagesThr: self.state.messagesThr.clone(),
        }
      }
    }
  
    pub fn read_inputs(&mut self) {
      enable_raw_mode().unwrap();  
      loop {
        if let Event::Key(key_event) = event::read().unwrap(){
          if key_event.kind == event::KeyEventKind::Press {
            match key_event.code {
              KeyCode::Char('c') if key_event.modifiers.contains(event::KeyModifiers::CONTROL) => break,
              KeyCode::Enter => {
                let ms = self.state.userInp.lock().clone().trim().to_owned();
                if ms == "" {
                  match self.state.chatReloadTX.send(()) {
                    Ok(_) => {},
                    Err(_) => break, 
                  };
                  continue;
                }
                self.state.userInp.lock().clear();
                let signal = SignalsData::new(
                  vec![
                    SignalsHeader::signalType(Signal::Message),
                    SignalsHeader::withMess,
                    SignalsHeader::username(self.state.username.to_owned())
                  ],
                  Some(&ms)
                );
      
                self.connection.stream.write_all(signal.to_string().as_bytes()).unwrap();
              },
              KeyCode::Backspace => {
                self.state.userInp.lock().pop();
                match self.state.chatReloadTX.send(()) {
                  Ok(_) => {},
                  Err(_) => break, 
                };
              }
              KeyCode::Char(k) => {
                println!("{k}");
                self.state.userInp.lock().push_str(&k.to_string());
                match self.state.chatReloadTX.send(()) {
                  Ok(_) => {},
                  Err(_) => break, 
                };
              },
              _ => {
                continue;
              }
            }
        }
        }
      }
    }
  }