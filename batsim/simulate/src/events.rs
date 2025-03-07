use serde::Serialize;

#[derive(Serialize, PartialEq, Debug, Default)]
pub enum ChargeType {
    EnRoute,
    #[default]
    Activity,
}

/// Charge event
/// Charge size is assumed to be kWs
/// Start_time and end times are in seconds
#[derive(Serialize, Default, Debug, PartialEq)]
pub struct Event<'a> {
    pub charge_type: ChargeType,
    pid: &'a str,
    pub spec: Option<String>,
    #[serde(rename = "charge_(kWs)")]
    pub charge: f32,
    pub day: u32,
    #[serde(rename = "start_time_(s)")]
    start_time: u32,
    #[serde(rename = "end_time_(s)")]
    end_time: u32,
    pub activity: Option<&'a str>,
    link_id: Option<&'a str>,
    x: f32,
    y: f32,
}

impl<'a> Event<'a> {
    pub fn en_route(
        pid: &'a str,
        spec: Option<String>,
        charge: f32,
        day: u32,
        time: (u32, u32),
        link_id: &'a str,
        loc: (f32, f32),
    ) -> Self {
        Event {
            charge_type: ChargeType::EnRoute,
            pid,
            spec,
            charge,
            day,
            start_time: time.0,
            end_time: time.1,
            activity: None,
            link_id: Some(link_id),
            x: loc.0,
            y: loc.1,
        }
    }
    pub fn activity(
        pid: &'a str,
        spec: Option<String>,
        charge: f32,
        day: u32,
        time: (u32, u32),
        activity: &'a str,
        loc: (f32, f32),
    ) -> Self {
        Event {
            charge_type: ChargeType::Activity,
            pid,
            spec,
            charge,
            day,
            start_time: time.0,
            end_time: time.1,
            activity: Some(activity),
            link_id: None,
            x: loc.0,
            y: loc.1,
        }
    }
    pub fn normalise(&mut self, days: usize, start_day: usize) {
        self.charge /= days as f32;
        self.day -= start_day as u32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_activity() {
        let event = Event::activity("a", None, 0., 0, (0, 1), "home", (0., 0.));
        assert_eq!(event.charge_type, ChargeType::Activity)
    }

    #[test]
    fn test_init_enroute() {
        let event = Event::en_route("a", None, 0., 0, (0, 1), "a", (0., 0.));
        assert_eq!(event.charge_type, ChargeType::EnRoute)
    }

    #[test]
    fn test_init_default() {
        let event = Event::default();
        assert_eq!(event.charge_type, ChargeType::Activity)
    }

    #[test]
    fn test_normalise() {
        let mut event = Event::en_route("a", None, 2., 2, (0, 1), "a", (0., 0.));
        event.normalise(2, 1);
        assert_eq!(event.charge, 1.);
        assert_eq!(event.day, 1);
    }
}
