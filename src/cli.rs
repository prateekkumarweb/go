use crate::{password::compute_password_hash, store::Config};
use inquire::{Confirm, Password, Text};
use secrecy::Secret;

pub async fn init() -> anyhow::Result<()> {
    let config_file = Text::new("Enter the path to the config file:")
        .with_default("data/config.yaml")
        .prompt()?;
    let username = Text::new("Enter username:").prompt()?;
    let password = Password::new("Enter password:")
        .with_custom_confirmation_message("Re-enter password:")
        .prompt()?;
    let confirm =
        Confirm::new("This may overwrite the existing config file if it is present").prompt()?;
    if confirm {
        Config::new_by_creating_file(
            config_file.into(),
            username,
            compute_password_hash(Secret::new(password))?,
        )
        .await?;
        println!("Config file created successfully!");
    }
    Ok(())
}
