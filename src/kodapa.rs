use tokio::sync::mpsc;

use super::Result;

pub struct MeetingPoint {
    pub title: String,
    pub sender: String,
}

pub async fn handle(_agenda_receiver: mpsc::UnboundedReceiver<MeetingPoint>) -> Result<()> {
    Ok(())
}
