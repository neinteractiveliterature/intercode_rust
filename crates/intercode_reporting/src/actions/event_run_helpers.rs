use std::collections::HashSet;

use chrono::NaiveDateTime;
use intercode_entities::{events, rooms, runs};

pub trait EventRunHelpers {
  fn get_event_by_id(&self, event_id: &i64) -> Option<&events::Model>;
  fn get_run_by_id(&self, run_id: &i64) -> Option<&runs::Model>;
  fn get_rooms_for_run_id(&self, run_id: &i64) -> Option<&Vec<rooms::Model>>;

  fn run_starts_at(&self, run_id: &i64) -> Option<NaiveDateTime> {
    self.get_run_by_id(run_id).map(|run| run.starts_at)
  }

  fn room_names_for_run(&self, run_id: &i64) -> String {
    let names = self
      .get_rooms_for_run_id(run_id)
      .map(|rooms| {
        rooms
          .iter()
          .filter_map(|room| room.name.as_deref())
          .collect::<HashSet<_>>()
      })
      .unwrap_or_default();

    let mut names = Vec::from_iter(names);
    names.sort();
    names.join(", ")
  }

  fn title_for_run(&self, run_id: &i64) -> String {
    let Some(run) = self.get_run_by_id(run_id) else {
      return "".to_string();
    };

    let event_title = self
      .get_event_by_id(&run.event_id)
      .map(|event| event.title.as_str())
      .unwrap_or_default();

    match run.title_suffix.as_deref() {
      Some(suffix) => format!("{} ({})", event_title, suffix),
      None => event_title.to_string(),
    }
  }
}
