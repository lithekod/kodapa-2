#![feature(box_patterns)]

use std::{env, error::Error};
use tokio::{join, sync::mpsc};

mod kodapa;
mod discord;

type Result<T> = ::std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn main() {
    let discord_token = env::var("DISCORD_BOT_TOKEN").expect("set the Discord bot token via env-variable DISCORD_BOT_TOKEN");

    let (agenda_sender, agenda_receiver) = mpsc::unbounded_channel::<kodapa::MeetingPoint>();

    let rt = tokio::runtime::Runtime::new().expect("unable to create async runtime");
    let _ = rt.block_on(async {
        join!(
            discord::handle(&discord_token, agenda_sender.clone()),
            kodapa::handle(agenda_receiver),
        )
    });
}
