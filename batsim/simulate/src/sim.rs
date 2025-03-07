use crate::{battery::BatteryState, events::Event, record::AgentSimulationRecord};
use configuration::{charge_plan::ActivityChargingPlanner, config::Config, handler::AgentConfig};
use tracer::{Component, Trace};

/// Run a simulation for given trace, using given battery and viable charge events (as trace indices).
// todo: reduce args, perhaps pass config ref
pub fn simulate<'a>(
    pid: &'a str,
    trace: &'a Trace,
    charge_activities: Vec<usize>,
    agent_config: &AgentConfig,
    activity_charging_planner: ActivityChargingPlanner,
    config: &Config,
) -> AgentSimulationRecord<'a> {
    let close_precision = config.precision.unwrap();
    let max_days = config.patience.unwrap();
    let battery_spec = agent_config.battery.unwrap();
    let trigger_spec = agent_config.trigger.unwrap();
    let en_route_spec = agent_config.en_route.unwrap();
    let mut battery = BatteryState::new(battery_spec, trigger_spec);
    let mut simulation_record = AgentSimulationRecord::new(pid, close_precision);

    for day in 0..max_days {
        simulation_record.new_day(battery.state);

        for (i, component) in trace.plan.iter().enumerate() {
            match component {
                Component::ActivityType(activity) if charge_activities.contains(&i) => {
                    let charge_spec = activity_charging_planner.get(&activity.act).unwrap();
                    let (charge, charge_duration) =
                        battery.charge_for_duration(activity.duration(), charge_spec.charge_rate);
                    if charge > 0.0 {
                        simulation_record.add_event(Event::activity(
                            pid,
                            charge_spec.name,
                            charge,
                            day as u32 + 1,
                            (activity.start_time, activity.start_time + charge_duration),
                            &activity.act,
                            activity.node,
                        ))
                    }
                }
                Component::LinkType(link) => {
                    battery.apply_distance(link.distance);
                    if battery.must_charge() {
                        // check for en-route charge
                        let (charge, duration) = match charge_activities.len() {
                            0 => battery.charge_to_full(en_route_spec.charge_rate), // no valid activities for charging so just charge to full
                            _ => {
                                // plan ahead to minimise en-route charge
                                let charge = plan_ahead(
                                    trace,
                                    &charge_activities,
                                    i,
                                    battery.consumption_rate,
                                );
                                battery.charge_to_desired(charge, en_route_spec.charge_rate)
                            }
                        };
                        simulation_record.add_event(Event::en_route(
                            pid,
                            en_route_spec.name.clone(),
                            charge,
                            day as u32 + 1,
                            (link.start_time, link.start_time + duration),
                            &link.lid,
                            link.node,
                        ))
                    }
                }
                _ => (),
            }
        }
        if simulation_record.try_to_close(battery.state) {
            return simulation_record;
        }
    }
    simulation_record.new_day(battery.state);
    simulation_record.force_close();
    simulation_record
}

/// Plan ahead from index i looking for next available activity charge, return required additional charge to get there
fn plan_ahead(trace: &Trace, charge_activities: &[usize], start: usize, efficiency: f32) -> f32 {
    let mut required_charge = 0.0;
    for (i, component) in trace.plan.iter().enumerate().skip(start) {
        // this includes current link (again)
        match component {
            Component::ActivityType(_) if charge_activities.contains(&i) => return required_charge,
            Component::LinkType(ref link) => {
                required_charge += link.distance * efficiency;
            }
            Component::ActivityType(_) => (),
        }
    }
    for (i, component) in trace.plan.iter().enumerate().take(start) {
        match component {
            &Component::ActivityType(_) if charge_activities.contains(&i) => {
                return required_charge
            }
            Component::LinkType(link) => {
                required_charge += link.distance * efficiency;
            }
            Component::ActivityType(_) => (),
        };
    }
    required_charge
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::record::EventsRecord;
    use configuration::groups::{
        activity::ActivitySpec,
        battery::{BatterySpec, BatterySpecBuilder},
        en_route::EnRouteSpec,
        trigger::TriggerSpec,
    };
    use tracer::{Activity, Component, Link, Trace};

    macro_rules! quick_trace {
    ( [ $( $component:tt ),* ] ) => {
        Trace {
            plan: vec![ $( quick_trace!($component ) ),* ]
        }
    };
    ( ( A, $act:expr , $st:expr , $et:expr , $x:expr, $y:expr) )  => {
        Component::ActivityType(Activity {
            act: $act.to_string(),
            start_time: $st,
            end_time: $et,
            node: ($x as f32, $y as f32),
        })
    };
    ( ( L, $lid:expr , $st:expr , $et:expr , $d:expr , $x:expr, $y:expr ) ) => {
        Component::LinkType(Link {
            lid: $lid.to_string(),
            start_time: $st,
            end_time: $et,
            distance: $d as f32,
            node: ($x as f32, $y as f32),
        })
    };
    () => {};
}

    #[test]
    fn test_leg_sim() {
        let config = Config::default();
        let trace = quick_trace!([(L, "a", 1, 2, 1.0, 0, 0)]);
        let battery_spec = BatterySpec::unit();
        let trigger_spec = TriggerSpec::empty();
        let activity_spec = ActivitySpec::new(
            Some("home".to_string()),
            vec!["home".to_string()],
            1.0,
            None,
            None,
        );
        let enroute_spec = EnRouteSpec::new(Some("enroute".to_string()), 1.0, None, None);
        let agent_config: AgentConfig = AgentConfig {
            pid: "a",
            battery: Some(&battery_spec),
            trigger: Some(&trigger_spec),
            en_route: Some(&enroute_spec),
            activities: vec![&activity_spec],
        };

        let charge_activity_configs = ActivityChargingPlanner::new(vec![&activity_spec]);
        let charge_activities = vec![];
        let event = Event::en_route(
            "A",
            Some("enroute".to_string()),
            1.,
            1,
            (1, 2),
            "a",
            (0.0, 0.0),
        );
        let expected = vec![&event];
        let binding = simulate(
            "A",
            &trace,
            charge_activities,
            &agent_config,
            charge_activity_configs,
            &config,
        );
        let simulation_record = binding.days().flatten().collect::<Vec<&Event>>();
        assert_eq!(simulation_record, expected);
    }
    #[test]
    fn test_sim_full_charge_end_of_day() {
        let config = Config::default();
        let trace = quick_trace!([
            (L, "a", 1, 2, 1.0, 0, 0),
            (A, "work", 2, 3, 0, 0),
            (L, "b", 3, 4, 1.0, 1, 1),
            (A, "home", 4, 10, 0, 0)
        ]);
        let battery_spec = BatterySpecBuilder::new()
            .capacity(3.0 / 3600.0) // 3 kWs
            .full()
            .consumption_rate(1.0 / 3.6)
            .build();
        let trigger_spec = TriggerSpec::empty();
        let enroute_spec = EnRouteSpec::new(Some("enroute".to_string()), 1.0, None, None);
        let activity_spec = ActivitySpec::new(
            Some("home".to_string()),
            vec!["home".to_string()],
            1.0,
            None,
            None,
        );
        let agent_config: AgentConfig = AgentConfig {
            pid: "a",
            battery: Some(&battery_spec),
            trigger: Some(&trigger_spec),
            en_route: Some(&enroute_spec),
            activities: vec![&activity_spec],
        };
        let charge_activity_configs = ActivityChargingPlanner::new(vec![&activity_spec]);
        let charge_activities = vec![3];
        let event = Event::activity(
            "A",
            Some("home".to_string()),
            2.,
            1,
            (4, 6),
            "home",
            (0.0, 0.0),
        );
        let expected = vec![&event];
        assert_eq!(
            simulate(
                "A",
                &trace,
                charge_activities,
                &agent_config,
                charge_activity_configs,
                &config,
            )
            .days()
            .flatten()
            .collect::<Vec<&Event>>(),
            expected
        )
    }

    #[test]
    fn test_sim_no_activity_charge() {
        let config = Config::default();
        let trace = quick_trace!([
            (L, "a", 1, 2, 1., 0, 0),
            (A, "work", 2, 3, 1, 1),
            (L, "b", 3, 4, 1., 1, 1),
            (L, "c", 4, 5, 1., 2, 2),
            (A, "home", 5, 11, 0, 0)
        ]);
        let battery_spec = BatterySpecBuilder::new()
            .capacity(2.0 / 3600.0) // 3 kWs
            .full()
            .consumption_rate(1.0 / 3.6)
            .build();
        let trigger_spec = TriggerSpec::empty();
        let enroute_spec = EnRouteSpec::new(Some("enroute".to_string()), 1.0, None, None);
        let activity_spec = ActivitySpec::new(None, vec!["home".to_string()], 1.0, None, None);
        let agent_config: AgentConfig = AgentConfig {
            pid: "a",
            battery: Some(&battery_spec),
            trigger: Some(&trigger_spec),
            en_route: Some(&enroute_spec),
            activities: vec![&activity_spec],
        };
        let charge_activity_configs = ActivityChargingPlanner::new(vec![&activity_spec]);
        let charge_activities = vec![];
        let event_a = Event::en_route(
            "A",
            Some("enroute".to_string()),
            2.,
            1,
            (3, 5),
            "b",
            (1.0, 1.0),
        );
        let event_b = Event::en_route(
            "A",
            Some("enroute".to_string()),
            2.,
            2,
            (1, 3),
            "a",
            (0.0, 0.0),
        );
        let event_c = Event::en_route(
            "A",
            Some("enroute".to_string()),
            2.,
            2,
            (4, 6),
            "c",
            (2.0, 2.0),
        );
        let expected = vec![&event_a, &event_b, &event_c];
        assert_eq!(
            simulate(
                "A",
                &trace,
                charge_activities,
                &agent_config,
                charge_activity_configs,
                &config,
            )
            .days()
            .flatten()
            .collect::<Vec<&Event>>(),
            expected
        )
    }

    #[test]
    fn test_sim_look_ahead() {
        let config = Config::default();
        let trace = quick_trace!([
            (L, "a", 1, 2, 1., 0, 0),
            (L, "b", 2, 3, 1., 1, 1),
            (L, "c", 3, 4, 1., 2, 2),
            (A, "home", 4, 5, 0, 0)
        ]);
        let battery_spec = BatterySpecBuilder::new()
            .capacity(2.0 / 3600.)
            .full()
            .consumption_rate(1.0 / 3.6)
            .build();
        let trigger_spec = TriggerSpec::empty();
        let charge_act = ActivitySpec::new(
            Some("home".to_string()),
            vec!["home".to_string()],
            1.0,
            None,
            None,
        );
        let charge_activity_configs = ActivityChargingPlanner::new(vec![&charge_act]);
        let en_route_spec = EnRouteSpec::new(Some("enroute".to_string()), 1.0, None, None);
        let charge_activities = vec![3];
        let agent_config: AgentConfig = AgentConfig {
            pid: "a",
            battery: Some(&battery_spec),
            trigger: Some(&trigger_spec),
            en_route: Some(&en_route_spec),
            activities: vec![&charge_act],
        };
        let event_a = Event::en_route(
            "A",
            Some("enroute".to_string()),
            2.,
            1,
            (2, 4),
            "b",
            (1.0, 1.0),
        );
        let event_b = Event::activity(
            "A",
            Some("home".to_string()),
            1.,
            1,
            (4, 5),
            "home",
            (0.0, 0.0),
        );
        let expected = vec![&event_a, &event_b];
        assert_eq!(
            simulate(
                "A",
                &trace,
                charge_activities,
                &agent_config,
                charge_activity_configs,
                &config,
            )
            .days()
            .flatten()
            .collect::<Vec<&Event>>(),
            expected
        )
    }
}
