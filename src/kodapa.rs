use tokio::{join, sync::{broadcast, mpsc}};

use super::Result;

#[derive(Debug)]
pub struct MeetingPoint {
    pub title: String,
    pub sender: String,
}

#[derive(Debug, Clone)]
pub enum Event {
    SendMessage {
        msg: String,
    }
}

pub async fn handle(
    agenda_receiver: mpsc::UnboundedReceiver<MeetingPoint>,
    event_sender: broadcast::Sender<Event>,
) -> Result<()> {
    let (e1, e2) = join!(
        handle_agenda(agenda_receiver),
        handle_reminders(event_sender),
    );
    println!("kodapa::handle: done");
    for e in [e1, e2].iter().filter_map(|e| e.as_ref().err()) {
        println!("  error occured: {:?}", e);
    }
    Ok(())
}

async fn handle_agenda(_receiver: mpsc::UnboundedReceiver<MeetingPoint>) -> Result<()> {
    todo!()
}

async fn handle_reminders(_event_sender: broadcast::Sender<Event>) -> Result<()> {
    todo!()
}
