#![feature(box_patterns)]

mod discord;

use std::{env, error::Error};

type Result<T> = ::std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn main() {
    let discord_token = env::var("DISCORD_BOT_TOKEN").expect("set the Discord bot token via env-variable DISCORD_BOT_TOKEN");

    let rt = tokio::runtime::Runtime::new().expect("unable to create async runtime");
    rt.block_on(discord::handle(&discord_token)).expect("something went wrong in the Discord handler");
}
