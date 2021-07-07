use std::sync::Arc;
use tokio::{join, sync::{Notify, broadcast, mpsc}};

use crate::{agenda::{Agenda, AgendaPoint, read_agenda}, calendar};

#[derive(Debug, Clone)]
pub enum Event {
    Reminder {
        agenda: Agenda,
    }
}

/// Entry point for the kodapa logic.
pub async fn handle(
    agenda_receiver: mpsc::UnboundedReceiver<AgendaPoint>,
    event_sender: broadcast::Sender<Event>,
) {
    let reminder_notifier = Arc::new(Notify::new());
    let (_e1, _e2) = join!(
        handle_agenda(agenda_receiver),
        handle_reminders(reminder_notifier, event_sender.clone()),
    );
    println!("kodapa::handle: done");
}

/// Receives meeting points from Discord and adds them to the agenda.
async fn handle_agenda(mut receiver: mpsc::UnboundedReceiver<AgendaPoint>) {
    while let Some(point) = receiver.recv().await {
        let mut agenda = read_agenda();
        agenda.points.push(point);
        agenda.write();
    }
}

/// Receives notifications when a reminder should be sent and sends it.
async fn handle_reminders(notifier: Arc<Notify>, event_sender: broadcast::Sender<Event>) {
    let (_e1, _e2) = join!(
        calendar::handle(Arc::clone(&notifier)),
        async {
            loop {
                notifier.notified().await;
                event_sender.send(get_reminder_event()).unwrap();
            }
        }
    );
}

/// Read the agenda and get an Event that can be sent to Discord.
fn get_reminder_event() -> Event {
    Event::Reminder {
        agenda: read_agenda(),
    }
}

fn _print_errors<T, U: std::fmt::Debug>(errs: &[Result<T, U>]) {
    for e in errs.iter().filter_map(|e| e.as_ref().err()) {
        println!("  error occured: {:?}", e);
    }
}
