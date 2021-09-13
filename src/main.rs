use std::error::Error;
use tokio::{
    join,
    sync::{broadcast, mpsc},
};

use self::agenda::AgendaPoint;

mod agenda;
mod calendar;
mod discord;
mod error;
mod kodapa;

#[allow(dead_code)]
type Result<T> = ::std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn main() {
    let discord_token = std::env::var("DISCORD_BOT_TOKEN").expect("missing DISCORD_BOT_TOKEN");

    let (agenda_sender, agenda_receiver) = mpsc::unbounded_channel::<AgendaPoint>();
    let (event_sender, event_receiver) = broadcast::channel::<kodapa::Event>(10);

    let rt = tokio::runtime::Runtime::new().expect("unable to create async runtime");
    let _ = rt.block_on(async {
        join!(
            discord::handle(discord_token, agenda_sender, event_receiver),
            kodapa::handle(agenda_receiver, event_sender),
        )
    });
}
