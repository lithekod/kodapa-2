use std::{env, error::Error};
use tokio::{join, sync::{broadcast, mpsc}};

mod calendar;
mod discord;
mod kodapa;

type Result<T> = ::std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn main() {
    // let discord_token = env::var("DISCORD_BOT_TOKEN").expect("set the Discord bot token via env-variable DISCORD_BOT_TOKEN");

    // let (agenda_sender, agenda_receiver) = mpsc::unbounded_channel::<kodapa::MeetingPoint>();
    // let (event_sender, event_receiver) = broadcast::channel::<kodapa::Event>(10);

    let rt = tokio::runtime::Runtime::new().expect("unable to create async runtime");
    let _ = rt.block_on(async {
        join!(
            calendar::handle(),
            // discord::handle(&discord_token, agenda_sender, event_receiver),
            // kodapa::handle(agenda_receiver, event_sender),
        )
    });
}
