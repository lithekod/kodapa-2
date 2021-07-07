use tokio::{join, sync::{broadcast, mpsc}};

use crate::{agenda::{Agenda, AgendaPoint}, calendar};

#[derive(Debug, Clone)]
pub enum Event {
    Reminder {
        event: calendar::model::events::Event,
    },
}

/// Entry point for the kodapa logic.
pub async fn handle(
    agenda_receiver: mpsc::UnboundedReceiver<AgendaPoint>,
    event_sender: broadcast::Sender<Event>,
) {
    let (_e1, _e2) = join!(
        handle_agenda(agenda_receiver),
        handle_reminders(event_sender.clone()),
    );
    println!("kodapa::handle: done");
}

/// Receives meeting points from Discord and adds them to the agenda.
async fn handle_agenda(mut receiver: mpsc::UnboundedReceiver<AgendaPoint>) {
    while let Some(point) = receiver.recv().await {
        let mut agenda = Agenda::read();
        agenda.points.push(point);
        agenda.write();
    }
}

/// Receives notifications when a reminder should be sent and sends it.
async fn handle_reminders(event_sender: broadcast::Sender<Event>) {
    let (calendar_tx, mut calendar_rx) = mpsc::unbounded_channel();
    let (_e1, _e2) = join!(
        calendar::handle(calendar_tx),
        async {
            while let Some(event) = calendar_rx.recv().await {
                event_sender.send(Event::Reminder { event }).unwrap();
            }
        }
    );
}

fn _print_errors<T, U: std::fmt::Debug>(errs: &[Result<T, U>]) {
    for e in errs.iter().filter_map(|e| e.as_ref().err()) {
        println!("  error occured: {:?}", e);
    }
}
