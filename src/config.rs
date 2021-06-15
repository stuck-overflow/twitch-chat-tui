use anyhow::{Context, Result};
use figment::providers::{Env, Format, Serialized, Toml};
use figment::Figment;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "twitch", about = "Configuration flags for twitch-chat-tui")]
struct Flags {
    /// Config file
    #[structopt(
        parse(from_os_str),
        short,
        long,
        default_value = "twitch-chat-tui.toml"
    )]
    config: PathBuf,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub channel: String,
    pub mod_symbol: String,
    pub mod_symbol_width: usize,
    pub vip_symbol: String,
    pub vip_symbol_width: usize,
    pub subscriber_symbol: String,
    pub subscriber_symbol_width: usize,
    pub founder_symbol: String,
    pub founder_symbol_width: usize,
    pub invert_below_brightness: u8,
    pub messages_buffer_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            channel: "stuck_overflow".to_owned(),
            mod_symbol: "🗡 ".to_owned(),
            mod_symbol_width: 2,
            vip_symbol: "💎".to_owned(),
            vip_symbol_width: 2,
            subscriber_symbol: "🌟".to_owned(),
            subscriber_symbol_width: 2,
            founder_symbol: "🥇".to_owned(),
            founder_symbol_width: 2,
            invert_below_brightness: 30,
            messages_buffer_size: 50,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        Figment::from(Serialized::defaults(Self::default()))
            .merge(Toml::file(Flags::from_args().config))
            .merge(Env::prefixed("TWITCH_"))
            .extract()
            .context("failed to load config")
    }
}
