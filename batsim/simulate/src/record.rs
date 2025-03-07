use serde::Serialize;

use crate::{
    days::Day,
    events::{ChargeType, Event},
};
use configuration::config::Config;

#[derive(Serialize)]
pub struct PlanRecord<'a> {
    pub pid: &'a str,
    pub days: usize,
    pub number_enroute: usize,
    pub number_activity: usize,
    pub number_charges: usize,
    #[serde(rename = "total_charge_(kWh)")]
    pub total_charge: f32,
    #[serde(rename = "total_enroute_(kWh)")]
    pub total_enroute: f32,
    #[serde(rename = "total_activity_(kWh)")]
    pub total_activity: f32,
    #[serde(rename = "leak_(kWh)")]
    pub leak: Option<f32>,
}

pub trait EventsRecord<'a> {
    // clean up record after simulation, for example to normalise to a single day and apply optional scale
    fn finalise(&mut self, config: &Config);

    // access events as an iterator
    fn days(&'a self) -> std::slice::Iter<'a, Day<'a>>;

    // create serialisable record of simulation
    fn to_record(&self) -> PlanRecord
    where
        PlanRecord<'a>: Serialize;
}

#[derive(PartialEq, Debug)]
pub struct AgentSimulationRecord<'a> {
    pid: &'a str,
    // vec of days, where each day hold vec of charge events
    days: Vec<Day<'a>>,
    // vec of charges at the start of each day
    history: Vec<f32>,
    // loop index
    slice_start: usize,
    slice_end: Option<usize>,
    close_precision: f32,
    pub error: Option<f32>,
}

impl<'a> EventsRecord<'a> for AgentSimulationRecord<'a> {
    // todo - find a better way to structure this post processing/normalising
    fn finalise(&mut self, config: &Config) {
        self.error = Some(self.error.unwrap() * config.scale.unwrap());
        let slice_length = self.slice().len();
        let start_day = self.slice_start;
        let end_day = self.slice_end.unwrap_or(self.len());
        for i in start_day..end_day {
            // only normalise within the slice
            for event in self.days[i].events.iter_mut() {
                if slice_length > 1 {
                    event.normalise(slice_length, start_day)
                };
                event.charge *= config.scale.unwrap();
            }
        }
    }

    fn days(&'a self) -> std::slice::Iter<'a, Day<'a>> {
        self.slice().iter()
    }

    fn to_record(&self) -> PlanRecord {
        PlanRecord {
            pid: self.pid,
            days: self.len(),
            number_charges: self.get_count(),
            number_enroute: self.get_count_en_route(),
            number_activity: self.get_count_activity(),
            total_charge: self.get_total_charge(),
            total_enroute: self.get_total_charge_en_route(),
            total_activity: self.get_total_charge_activity(),
            leak: self.get_error(),
        }
    }
}

impl<'a> AgentSimulationRecord<'a> {
    pub fn new(pid: &'a str, close_precision: f32) -> Self {
        AgentSimulationRecord {
            pid,
            days: Vec::new(),
            history: Vec::new(),
            slice_start: 0,
            slice_end: None,
            close_precision,
            error: None,
        }
    }
    pub fn empty(pid: &'a str) -> Self {
        AgentSimulationRecord {
            pid,
            days: Vec::new(),
            history: Vec::new(),
            slice_start: 0,
            slice_end: None,
            close_precision: 0.0,
            error: Some(0.0),
        }
    }

    pub fn new_day(&mut self, battery_state: f32) {
        self.history.push(battery_state);
        self.days.push(Day::new());
    }

    pub fn add_event(&mut self, event: Event<'a>) {
        self.days.last_mut().unwrap().push(event);
    }

    /// Check if state is in history
    /// Update state
    pub fn try_to_close(&mut self, state: f32) -> bool {
        for (k, v) in self.history.iter().enumerate() {
            if (state - v).abs() < self.close_precision {
                self.slice_start = k;
                self.error = Some(self.error(state));
                return true;
            }
        }
        false
    }

    /// Look for best closed loop, set slice start and end
    pub fn force_close(&mut self) {
        let mut best_score: (f32, usize) = (f32::MAX, usize::MAX);
        for i in 0..(self.history.len() - 1) {
            for j in (i + 1)..self.history.len() {
                let leak: f32 = self.history[i] - self.history[j];
                let score = (leak.abs(), j - i);
                if score < best_score {
                    best_score = score;
                    self.slice_start = i;
                    self.slice_end = Some(j);
                }
            }
        }
        self.error = Some(self.history[self.slice_end.unwrap()] - self.history[self.slice_start]);
    }

    /// Get error (gap between state and start of record slice)
    /// Positive is a surplus of charge provided by charging
    pub fn error(&self, state: f32) -> f32 {
        state - self.history[self.slice_start]
    }

    /// Crop record based on a loop check with given sensitivity
    pub fn slice(&'a self) -> &'a [Day<'a>] {
        match self.slice_end {
            None => &self.days[self.slice_start..],
            Some(end) => &self.days[self.slice_start..end],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.days.is_empty()
    }
    pub fn len(&self) -> usize {
        self.days.len()
    }

    /// Retrieve total charge from plan, convert from kWs to kWh
    pub fn get_total_charge(&self) -> f32 {
        self.slice()
            .iter()
            .flatten()
            .map(|event| event.charge)
            .sum::<f32>()
            / 3600.0
    }

    /// Retrieve total en_route charge from plan, convert from kWs to kWh
    fn get_total_charge_en_route(&self) -> f32 {
        self.slice()
            .iter()
            .flatten()
            .filter_map(|event| match event.charge_type {
                ChargeType::EnRoute => Some(event.charge),
                _ => None,
            })
            .sum::<f32>()
            / 3600.0
    }

    /// Retrieve total en_route charge from plan, convert from kWs to kWh
    fn get_total_charge_activity(&self) -> f32 {
        self.slice()
            .iter()
            .flatten()
            .filter_map(|event| match event.charge_type {
                ChargeType::Activity => Some(event.charge),
                _ => None,
            })
            .sum::<f32>()
            / 3600.0
    }
    fn get_count(&self) -> usize {
        self.slice().iter().flatten().count()
    }
    fn get_count_en_route(&self) -> usize {
        self.slice()
            .iter()
            .flatten()
            .filter(|event| event.charge_type == ChargeType::EnRoute)
            .count()
    }
    fn get_count_activity(&self) -> usize {
        self.slice()
            .iter()
            .flatten()
            .filter(|event| event.charge_type == ChargeType::Activity)
            .count()
    }
    /// Retrieve error (or "leak") from plan, convert from kWs to kWh
    fn get_error(&self) -> Option<f32> {
        self.error.map(|v| v / 3600.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_record() {
        let record = AgentSimulationRecord::new("a", 0.1);
        assert_eq!(record.close_precision, 0.1)
    }

    fn record() -> AgentSimulationRecord<'static> {
        let mut record = AgentSimulationRecord::new("a", 2. * 3600.);
        //day 0
        record.new_day(10. * 3600.);
        //day 1
        record.new_day(4. * 3600.);
        record.add_event(Event::activity(
            "a",
            None,
            2. * 3600.,
            1,
            (0, 1),
            "home",
            (0., 0.),
        ));
        record.add_event(Event::en_route(
            "a",
            None,
            2. * 3600.,
            1,
            (1, 2),
            "a",
            (0., 0.),
        ));
        //day 2
        record.new_day(8. * 3600.);
        record.add_event(Event::en_route(
            "a",
            None,
            2. * 3600.,
            2,
            (0, 1),
            "a",
            (0., 0.),
        ));
        record
    }

    #[test]
    fn test_not_close() {
        let mut record = record();
        let state = 6. * 3600.;
        let closed = record.try_to_close(state);
        assert!(!closed);
        assert_eq!(record.error(state), -4. * 3600.);
        assert_eq!(record.slice_start, 0);
        assert_eq!(record.slice_end, None);
        assert_eq!(record.slice().len(), 3)
    }

    #[test]
    fn test_close_on_first() {
        let mut record = record();
        let state = 10. * 3600.;
        let closed = record.try_to_close(state);
        assert!(closed);
        assert_eq!(record.error(state), 0. * 3600.);
        assert_eq!(record.slice_start, 0);
        assert_eq!(record.slice_end, None);
        assert_eq!(record.slice().len(), 3)
    }

    #[test]
    fn test_close() {
        let mut record = record();
        let state = 5. * 3600.;
        let closed = record.try_to_close(state);
        assert!(closed);
        assert_eq!(record.error(state), 1. * 3600.);
        assert_eq!(record.slice_start, 1);
        assert_eq!(record.slice_end, None);
        assert_eq!(record.slice().len(), 2)
    }

    #[test]
    fn test_normalise_closed_first() {
        let mut record = record();
        let state = 10. * 3600.;
        let closed = record.try_to_close(state);
        assert!(closed);
        assert_eq!(record.get_total_charge(), 6.);
        record.finalise(&Config::default());
        assert_eq!(record.slice().len(), 3);
        assert_eq!(record.get_total_charge(), 2.)
    }

    #[test]
    fn test_normalise_closed() {
        let mut record = record();
        let state = 4. * 3600.;
        let closed = record.try_to_close(state);
        assert!(closed);
        assert_eq!(record.slice_start, 1);
        assert_eq!(record.slice_end, None);
        assert_eq!(record.error, Some(0.));
        assert_eq!(record.get_total_charge(), 6.);
        record.finalise(&Config::default());
        assert_eq!(record.slice().len(), 2);
        assert_eq!(record.get_total_charge(), 3.)
    }

    #[test]
    fn test_force_close_a() {
        let mut record = record();
        record.force_close();
        assert_eq!(record.slice_start, 0);
        assert_eq!(record.slice_end, Some(2));
        assert_eq!(record.slice().len(), 2);
        assert_eq!(record.error, Some(-2. * 3600.));
        assert_eq!(record.get_total_charge(), 4.);
        record.finalise(&Config::default());
        assert_eq!(record.get_total_charge(), 2.)
    }

    #[test]
    fn test_force_close_b() {
        let mut record = AgentSimulationRecord::new("a", 0.1 * 3600.);
        //day 0
        record.new_day(10. * 3600.);
        //day 1
        record.new_day(4. * 3600.);
        record.add_event(Event::activity(
            "a",
            None,
            2. * 3600.,
            1,
            (0, 1),
            "home",
            (0., 0.),
        ));
        record.add_event(Event::en_route(
            "a",
            None,
            2. * 3600.,
            1,
            (1, 2),
            "a",
            (0., 0.),
        ));
        //day 2
        record.new_day(8. * 3600.);
        record.add_event(Event::en_route(
            "a",
            None,
            2. * 3600.,
            2,
            (0, 1),
            "a",
            (0., 0.),
        ));
        //day 3
        record.new_day(5. * 3600.);
        record.add_event(Event::en_route(
            "a",
            None,
            1. * 3600.,
            2,
            (0, 1),
            "a",
            (0., 0.),
        ));

        record.force_close();
        assert_eq!(record.slice_start, 1);
        assert_eq!(record.slice_end, Some(3));
        assert_eq!(record.slice().len(), 2);
        assert_eq!(record.error, Some(1. * 3600.));
        assert_eq!(record.get_total_charge(), 6.);
        record.finalise(&Config::default());
        assert_eq!(record.get_total_charge(), 3.)
    }

    #[test]
    fn test_force_close_short() {
        let mut record = AgentSimulationRecord::new("a", 0.1 * 3600.);
        //day 0
        record.new_day(10. * 3600.);
        //day 1
        record.new_day(8. * 3600.);

        record.force_close();
        assert_eq!(record.slice_start, 0);
        assert_eq!(record.slice_end, Some(1));
        assert_eq!(record.slice().len(), 1);
        assert_eq!(record.error, Some(-2. * 3600.));
        assert_eq!(record.get_total_charge(), 0.);
        record.finalise(&Config::default());
        assert_eq!(record.get_total_charge(), 0.)
    }

    #[test]
    fn test_totals() {
        let record = record();
        assert_eq!(record.len(), 3);
        assert_eq!(record.get_total_charge(), 6.);
        assert_eq!(record.get_total_charge_activity(), 2.);
        assert_eq!(record.get_total_charge_en_route(), 4.);
        assert_eq!(record.get_count(), 3);
        assert_eq!(record.get_count_activity(), 1);
        assert_eq!(record.get_count_en_route(), 2);
    }
}
