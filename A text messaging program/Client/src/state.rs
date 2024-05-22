use std::{
    sync::{
        mpsc::{Sender, Receiver, self},
        Arc
    },
    io::{self, Write}
};
use crossterm::terminal::{self, Clear, ClearType};
use parking_lot::Mutex;

pub struct State{
    pub username: String,
    pub chatReloadRX: Option<Receiver<()>>,
    pub chatReloadTX: Sender<()>,
    pub userInp: Arc<Mutex<String>>,
    pub messagesThr: Arc<Mutex<Vec<String>>>
}

impl State{
    pub fn new() -> io::Result<State> {
        let (tx, rx) = mpsc::channel::<()>();

        let mut instance = State{
            username: String::new(),
            chatReloadRX: Some(rx),
            chatReloadTX: tx,
            userInp: Arc::new(Mutex::new(String::new())),
            messagesThr: Arc::new(Mutex::new(Vec::<String>::new())),
        };

        instance.readUserName()?;
        Ok(instance)
    }

    fn readUserName(&mut self) -> io::Result<()>{
        Clear(ClearType::All);
        print!("Ur name is: ");
        io::stdout().flush();

        let mut username = String::new();
        io::stdin().read_line(&mut username)?;
        self.username = username.trim().to_owned();
        Clear(ClearType::All);

        Ok(())
    }   
}