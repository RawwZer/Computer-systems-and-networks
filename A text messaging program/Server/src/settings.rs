use clap::{self, arg, Parser};

// using macros for generating parser for command args
#[derive(Parser)] 
pub struct Args {
  #[arg(short, long, help = "Port that the server will serve")]
  pub port: u16,

  #[arg(short, long, help = "Maximum amount of chat users")]
  pub max_users: Option<u16>,
}

// using macros for generating code for right output ({:?}) and 
// rewrited method 'clone'
#[derive(Debug, Clone)]
pub struct Settings {
  pub port: u16,
  pub max_users: u16,
}

impl Settings {
  pub fn new() -> Settings {
    let args = Args::parse(); // getting args
    
    // creating new instance
    Settings { 
      port: args.port, 
      max_users: args.max_users.unwrap_or(10), 
    }
  }
}