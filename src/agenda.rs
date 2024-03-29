use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::{fmt, fs, ops::RangeBounds};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgendaPoint {
    pub title: String,
    pub adder: String,
    pub timestamp: DateTime<Local>,
}

impl fmt::Display for AgendaPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} ({})",
            self.adder,
            self.title,
            self.timestamp.format("%B %d -- w%V-%u")
        )
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Agenda {
    pub points: Vec<AgendaPoint>,
}

impl Agenda {
    pub fn read() -> Self {
        match fs::read_to_string("agenda.json") {
            Ok(s) => serde_json::from_str(&s).expect("Error parsing agenda.json"),
            Err(_) => Self { points: Vec::new() },
        }
    }
    pub fn write(&self) {
        fs::write(
            std::path::Path::new("agenda.json"),
            serde_json::to_string_pretty(&self).expect("Can't serialize agenda"),
        )
        .expect("Can't write agenda.json");
    }

    pub fn push_write(point: AgendaPoint) {
        let mut agenda = Self::read();
        agenda.points.push(point);
        agenda.write();
    }

    pub fn remove_one(idx: usize) -> Result<(), String> {
        let mut agenda = Self::read();
        if idx >= agenda.points.len() {
            return Err("out of bounds".to_string());
        }
        agenda.points.remove(idx);
        agenda.write();
        Ok(())
    }

    pub fn remove_many(range: impl RangeBounds<usize>) -> Result<(), String> {
        let mut agenda = Self::read();
        if agenda
            .points
            .get((range.start_bound().cloned(), range.end_bound().cloned()))
            .is_none()
        {
            return Err("invalid range".to_string());
        }
        agenda.points.drain(range);
        agenda.write();
        Ok(())
    }
}

impl fmt::Display for Agenda {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self
            .points
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        write!(
            f,
            "{}",
            match s.as_str() {
                "" => "Empty agenda",
                _ => &s,
            }
        )
    }
}
