use std::sync::Arc;
use tokio::{join, sync::{Notify, broadcast, mpsc}};

use crate::calendar;

use super::Result;

#[derive(Debug, Clone)]
pub struct MeetingPoint {
    pub title: String,
    pub sender: String,
}

#[derive(Debug, Clone)]
pub enum Event {
    Reminder {
        agenda: Vec<MeetingPoint>,
    }
}

/// Entry point for the kodapa logic.
pub async fn handle(
    agenda_receiver: mpsc::UnboundedReceiver<MeetingPoint>,
    event_sender: broadcast::Sender<Event>,
) -> Result<()> {
    let reminder_notifier = Arc::new(Notify::new());
    let (e1, e2) = join!(
        handle_agenda(agenda_receiver),
        handle_reminders(reminder_notifier, event_sender.clone()),
    );
    println!("kodapa::handle: done");
    print_errors(&[e1, e2]);
    Ok(())
}

/// Receives meeting points from Discord and adds them to the agenda.
async fn handle_agenda(_receiver: mpsc::UnboundedReceiver<MeetingPoint>) -> Result<()> {
    todo!()
}

/// Receives notifications when a reminder should be sent and sends it.
async fn handle_reminders(notifier: Arc<Notify>, event_sender: broadcast::Sender<Event>) -> Result<()> {
    let (_e1, _e2) = join!(
        calendar::handle(Arc::clone(&notifier)),
        async {
            loop {
                notifier.notified().await;
                event_sender.send(get_reminder_event()).unwrap();
            }
        }
    );
    Ok(())
}

/// Read the agenda and get an Event that can be sent to Discord.
fn get_reminder_event() -> Event {
    todo!()
}

fn print_errors<T>(errs: &[Result<T>]) {
    for e in errs.iter().filter_map(|e| e.as_ref().err()) {
        println!("  error occured: {:?}", e);
    }
}
