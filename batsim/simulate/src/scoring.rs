use crate::{events::ChargeType, record::AgentSimulationRecord};

/// Score charge events
pub fn score_events(record: &AgentSimulationRecord) -> (f32, f32, f32) {
    let days = record.slice().len() as f32;
    let mut en_route: u32 = 0;
    let mut cost: f32 = 0.;
    let mut activity: u32 = 0;
    for daily_events in record.slice() {
        for event in daily_events {
            match event.charge_type {
                ChargeType::Activity => {
                    activity += 1;
                }
                ChargeType::EnRoute => {
                    en_route += 1;
                    cost += event.charge;
                }
            }
        }
    }
    (
        en_route as f32 / days, // number of en-route charge events per day
        cost / days,            // average en-route charge total per day
        activity as f32 / days, // number of activity charge events per day
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::Event;

    #[test]
    fn test_scoring() {
        let mut record = AgentSimulationRecord::new("A", 0.1);
        record.new_day(10.0);
        record.add_event(Event::activity(
            "A",
            None,
            2.,
            1,
            (4, 7),
            "home",
            (0.0, 0.0),
        ));
        record.new_day(9.0);
        record.add_event(Event::en_route("A", None, 1., 2, (4, 7), "a", (0.0, 0.0)));
        assert_eq!(score_events(&record), (0.5, 0.5, 0.5));
        record.new_day(8.0);
        record.new_day(7.0);
        record.add_event(Event::en_route("A", None, 3., 4, (4, 7), "a", (0.0, 0.0)));
        record.add_event(Event::activity(
            "A",
            None,
            1.,
            4,
            (4, 7),
            "home",
            (0.0, 0.0),
        ));
        assert_eq!(score_events(&record), (0.5, 1., 0.5));
    }
}
