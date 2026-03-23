#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = LcolachatWindow::new()?;

    // grahh

    ui.run()?;

    Ok(())
}
