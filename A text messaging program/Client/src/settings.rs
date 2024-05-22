use clap::Parser;

// using macros for generating parser for command args
#[derive(Parser)]
pub struct Args {
  #[arg(short, long, help = "Server address")]
  pub address: String,
}

// using macros for generating code for right output ({:?}) and 
// rewrited method 'clone'
#[derive(Debug, Clone)]
pub struct Settings {
  pub server_address: String,
}

impl Settings {
  pub fn new() -> Settings {
    let args = Args::parse();
    
    Settings { 
      server_address: args.address,
    }
  }
}