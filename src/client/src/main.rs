use std::io::{Read, Write};
use std::process;
use clap::Parser;
use crate::cli::Cli;
use crate::mappers::ClientRequest;

mod cli;
mod mappers;

fn print_banner() {
    println!(r"
          __            __
    |  | /__` \ / |\ | /  `
    |/\| .__/  |  | \| \__,
   ");
}

fn main() {
    print_banner();

    let cli: Cli = Cli::parse();
    let _request = ClientRequest::get_client_request(cli).unwrap_or_else(|e| {
        eprintln!("[ERROR] {e}");
        process::exit(1)
    });

}
