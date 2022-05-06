#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::path::PathBuf;
use std::{env, fs};

use clap::StructOpt;
use color_eyre::eyre::{Context, Result};
use tegra_rcm::{Error, Payload, Rcm};

mod cli;
#[cfg(feature = "gui")]
mod gui;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    // check if we should start the gui
    #[cfg(feature = "gui")]
    check_gui_mode()?;

    color_eyre::install()?;
    let args = Cli::parse();

    match args.command {
        Commands::Execute { payload, wait } => execute(payload, wait)?,
        Commands::Device {} => device()?,
        #[cfg(feature = "gui")]
        Commands::Gui {} => gui::gui()?,
    }
    Ok(())
}

fn execute(payload: PathBuf, wait: bool) -> Result<()> {
    let payload_bytes = fs::read(&payload)
        .wrap_err_with(|| format!("Failed to read payload from: {}", &payload.display()))?;
    let payload = Payload::new(&payload_bytes)?;

    let mut switch = Rcm::new(wait)?;
    switch.init()?;
    println!("Smashing the stack!");

    // We need to read the device id first
    let _ = switch.read_device_id()?;
    switch.execute(&payload)?;

    println!("Done!");
    Ok(())
}

fn device() -> Result<()> {
    let switch = Rcm::new(false);

    let err = match switch {
        Ok(_) => {
            println!("[✓] Switch is RCM mode and connected");
            return Ok(());
        }
        Err(e) => e,
    };

    match err {
        Error::SwitchNotFound => println!("[x] Switch in RCM mode not found"),
        Error::AccessDenied => {
            switch.wrap_err_with(|| format!("USB permission error\nSee \"https://github.com/budde25/switcheroo#linux-permission-denied-error\" to troubleshoot"))?;
        }
        _ => return Err(err.into()),
    };

    Ok(())
}

#[cfg(feature = "gui")]
fn check_gui_mode() -> Result<()> {
    match env::var_os("SWITCHEROO_GUI_ONLY") {
        None => return Ok(()),
        Some(gui_only) => {
            if gui_only == "0" {
                gui::gui()?;
            }
        }
    };
    Ok(())
}
