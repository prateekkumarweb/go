use crate::password::verify_password_hash;
use anyhow::{Context, Result};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize, Serializer};
use std::{collections::HashMap, path::PathBuf};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub short: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Store {
    username: Username,
    #[serde(serialize_with = "serialize_password")]
    password: Password,
    links: HashMap<String, String>,
}

pub struct Config {
    path: PathBuf,
    store: Store,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Username(String);

#[derive(Debug, Clone, Deserialize)]
pub struct Password(Secret<String>);

fn serialize_password<S>(secret: &Password, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(secret.0.expose_secret())
}

async fn read_db(path: &PathBuf) -> Result<Store> {
    let contents = fs::read_to_string(path)
        .await
        .context(format!("Failed to read config file: {:?}", path))?;
    Ok(serde_yaml::from_str(&contents)?)
}

async fn write_db(path: &PathBuf, db: &Store) -> Result<()> {
    let contents = serde_yaml::to_string(db).context("Failed to desearialize config")?;
    fs::write(path, contents)
        .await
        .context(format!("Failed to write config file: {:?}", path))
}

// #[derive(Debug, Clone)]
// pub struct Credentials {
//     username: Username,
//     password: Password,
// }

// impl Credentials {
//     pub fn auth(&self, username: &str, password: Secret<String>) -> Option<Username> {
//         if username == self.username.0
//             && verify_password_hash(self.password.0.clone(), password).is_ok()
//         {
//             Some(Username(username.into()))
//         } else {
//             None
//         }
//     }
// }

impl Config {
    pub async fn new(path: PathBuf) -> Result<Self> {
        let store = read_db(&path)
            .await
            .context(format!("Config file is missing or invalid: {:?}", path))?;
        Ok(Self { path, store })
    }

    pub async fn new_by_creating_file(
        path: PathBuf,
        username: String,
        password: Secret<String>,
    ) -> Result<Self> {
        let store = Store {
            links: HashMap::new(),
            username: Username(username),
            password: Password(password),
        };
        write_db(&path, &store).await?;
        Ok(Self { path, store })
    }

    // pub fn auth(&self) -> Credentials {
    //     Credentials {
    //         username: self.store.username.clone(),
    //         password: self.store.password.clone(),
    //     }
    // }

    pub fn auth_user(&self, username: &str, password: Secret<String>) -> Option<Username> {
        if username == self.store.username.0
            && verify_password_hash(self.store.password.0.clone(), password).is_ok()
        {
            Some(Username(username.into()))
        } else {
            None
        }
    }

    pub fn links_iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.store.links.iter()
    }

    pub fn get_link(&self, short: &str) -> Option<&String> {
        self.store.links.get(short)
    }

    pub async fn add_link(&mut self, link: Link) -> Result<()> {
        self.store
            .links
            .insert(link.short.clone(), link.url.clone());
        write_db(&self.path, &self.store)
            .await
            .context(format!("Failed to add short link {:?}", link))?;
        Ok(())
    }

    pub async fn remove_link(&mut self, short: &str) -> Result<String> {
        self.store
            .links
            .remove(short)
            .context("Failed to find short link to be deleted")
    }
}
