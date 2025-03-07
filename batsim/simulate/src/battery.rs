use configuration::groups::{battery::BatterySpec, trigger::TriggerSpec};

/// Battery state keeps track of the agent battery state during simulation.
/// We also convert battery specification units from hours to seconds and km to metres.
#[derive(Debug, Clone)]
pub struct BatteryState {
    pub state: f32,
    pub capacity: f32,
    pub initial: f32,
    pub trigger: f32,
    pub consumption_rate: f32,
}
impl BatteryState {
    pub fn new(battery_spec: &BatterySpec, trigger_spec: &TriggerSpec) -> BatteryState {
        let capacity = battery_spec.capacity * 3600.0; // convert kWh to kWs
        BatteryState {
            state: battery_spec.initial * 3600.0,     // convert kWh to kWs
            capacity,                                 // convert kWh to kWs
            initial: battery_spec.initial * 3600.0,   // convert kWh to kWs
            trigger: trigger_spec.trigger * capacity, // convert kWh to kWs
            consumption_rate: battery_spec.consumption_rate * 3.6, // convert kWh/km to kWs/m
        }
    }

    /// Reduce battery state for given distance
    pub fn apply_distance(&mut self, distance: f32) {
        self.state -= distance * self.consumption_rate;
    }

    /// Return difference between current battery state and capacity
    pub fn deficit(&self) -> f32 {
        self.capacity - self.state
    }

    /// Charge desired if state is at or below trigger level
    pub fn must_charge(&self) -> bool {
        self.state <= self.trigger
    }

    /// Charge battery to full at given rate, return size of charge and duration of charge
    pub fn charge_to_full(&mut self, charge_rate: f32) -> (f32, u32) {
        let desired = self.deficit();
        let duration = (desired / charge_rate) as u32;
        self.state = self.capacity;
        (desired, duration)
    }

    /// Attempt to charge battery for given duration and rate, return achieved charge and duration
    pub fn charge_for_duration(&mut self, duration: u32, charge_rate: f32) -> (f32, u32) {
        let mut charge = duration as f32 * charge_rate;
        if charge > self.deficit() {
            charge = self.deficit();
            let duration = charge / charge_rate;
            self.charge_to_full(charge_rate);
            return (charge, duration as u32);
        }
        self.state += charge;
        (charge, duration)
    }

    /// Attempt to apply desired charge at given rate, return achieved charge and duration
    pub fn charge_to_desired(&mut self, desired_charge: f32, charge_rate: f32) -> (f32, u32) {
        if desired_charge > self.deficit() {
            let charge = self.deficit();
            let duration = charge / charge_rate;
            self.charge_to_full(charge_rate);
            return (charge, duration as u32);
        }
        self.state += desired_charge;
        let duration = desired_charge / charge_rate;
        (desired_charge, duration as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battery_state_apply_distance() {
        let spec = BatterySpec::unit();
        let trigger_spec = TriggerSpec::empty();
        let mut battery = BatteryState::new(&spec, &trigger_spec);
        assert_eq!(battery.state, 1.0);

        battery.apply_distance(0.5);
        assert_eq!(battery.state, 0.5);
        assert_eq!(battery.deficit(), 0.5);
        assert!(!battery.must_charge());

        battery.apply_distance(0.5);
        assert_eq!(battery.state, 0.0);
        assert_eq!(battery.deficit(), 1.0);
        assert!(battery.must_charge());

        battery.apply_distance(0.5);
        assert_eq!(battery.state, -0.5);
        assert_eq!(battery.deficit(), 1.5);
        assert!(battery.must_charge());
    }

    #[test]
    fn test_charge_to_full_already_full() {
        let spec = BatterySpec::unit();
        let trigger_spec = TriggerSpec::empty();
        let mut battery = BatteryState::new(&spec, &trigger_spec);
        let (deficit, duration) = battery.charge_to_full(1.0);
        assert_eq!(deficit, 0.0);
        assert_eq!(duration, 0);
        assert_eq!(battery.deficit(), 0.0);
    }

    #[test]
    fn test_charge_to_full() {
        let spec = BatterySpec::unit();
        let trigger_spec = TriggerSpec::empty();
        let mut battery = BatteryState::new(&spec, &trigger_spec);
        battery.apply_distance(1.5);
        let (deficit, duration) = battery.charge_to_full(1.0);
        assert_eq!(deficit, 1.5);
        assert_eq!(duration, 1);
        assert_eq!(battery.deficit(), 0.0);
    }

    #[test]
    fn test_charge_for_duration_already_full() {
        let spec = BatterySpec::unit();
        let trigger_spec = TriggerSpec::empty();
        let mut battery = BatteryState::new(&spec, &trigger_spec);
        let (deficit, duration) = battery.charge_for_duration(1, 1.0);
        assert_eq!(deficit, 0.0);
        assert_eq!(duration, 0);
        assert_eq!(battery.deficit(), 0.0);
    }

    #[test]
    fn test_charge_for_duration_incomplete() {
        let spec = BatterySpec::unit();
        let trigger_spec = TriggerSpec::empty();
        let mut battery = BatteryState::new(&spec, &trigger_spec);
        battery.apply_distance(1.5);
        let (charge, duration) = battery.charge_for_duration(1, 1.0);
        assert_eq!(charge, 1.0);
        assert_eq!(duration, 1);
        assert_eq!(battery.deficit(), 0.5);
    }

    #[test]
    fn test_charge_for_duration() {
        let spec = BatterySpec::unit();
        let trigger_spec = TriggerSpec::empty();
        let mut battery = BatteryState::new(&spec, &trigger_spec);
        battery.apply_distance(0.5);
        let (charge, duration) = battery.charge_for_duration(1, 1.0);
        assert_eq!(charge, 0.5);
        assert_eq!(duration, 0); // rounds down from 0.5
        assert_eq!(battery.deficit(), 0.0);
    }

    #[test]
    fn test_charge_to_desired_already_full() {
        let spec = BatterySpec::unit();
        let trigger_spec = TriggerSpec::empty();
        let mut battery = BatteryState::new(&spec, &trigger_spec);
        let (deficit, duration) = battery.charge_to_desired(1.0, 1.0);
        assert_eq!(deficit, 0.0);
        assert_eq!(duration, 0);
        assert_eq!(battery.deficit(), 0.0);
    }

    #[test]
    fn test_charge_to_desired_incomplete() {
        let spec = BatterySpec::unit();
        let trigger_spec = TriggerSpec::empty();
        let mut battery = BatteryState::new(&spec, &trigger_spec);
        battery.apply_distance(1.5);
        let (charge, duration) = battery.charge_to_desired(1.0, 1.0);
        assert_eq!(charge, 1.0);
        assert_eq!(duration, 1);
        assert_eq!(battery.deficit(), 0.5);
    }

    #[test]
    fn test_charge_to_desired() {
        let spec = BatterySpec::unit();
        let trigger_spec = TriggerSpec::empty();
        let mut battery = BatteryState::new(&spec, &trigger_spec);
        battery.apply_distance(0.5);
        let (charge, duration) = battery.charge_to_desired(1.0, 1.0);
        assert_eq!(charge, 0.5);
        assert_eq!(duration, 0); // rounds down from 0.5
        assert_eq!(battery.deficit(), 0.0);
    }
}
