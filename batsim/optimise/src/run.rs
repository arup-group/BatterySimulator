use configuration::{charge_plan::ActivityChargingPlanner, config::Config, handler::AgentConfig};
use simulate::{record::AgentSimulationRecord, scoring::score_events, sim::simulate};
use tracer::Person;

/// For given person and battery simulate activity charging permitations and return best
pub fn run_simulations<'a>(
    pid: &'a str,
    person: &'a Person,
    agent_config: &AgentConfig,
    activity_charging_planner: ActivityChargingPlanner<'a>,
    config: &Config,
) -> Option<AgentSimulationRecord<'a>> {
    let mut best_result: Option<AgentSimulationRecord> = None;
    let mut best_score: (f32, f32, f32) = (f32::MAX, f32::MAX, f32::MAX);

    let charge_options = person.viable_combinations(activity_charging_planner.activities());

    for options in charge_options.into_iter() {
        for charge_activities in options.into_iter() {
            let charging_planner = activity_charging_planner.clone();
            let simulation_record = simulate(
                pid,
                &person.trace,
                charge_activities,
                agent_config,
                charging_planner,
                config,
            );
            let score = score_events(&simulation_record);
            if score < best_score {
                best_score = score;
                best_result = Some(simulation_record);
            }
        }
        if best_score.0 == 0. {
            // there are 0 en-route charge events - we do not need to look further
            return best_result;
        }
    }
    best_result
}

#[cfg(test)]
mod tests {
    use super::*;
    use configuration::{
        config::Config,
        groups::{
            activity::ActivitySpec, battery::BatterySpecBuilder, en_route::EnRouteSpec,
            trigger::TriggerSpec,
        },
    };
    use simulate::{events::Event, record::EventsRecord};
    use std::collections::HashMap;
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
    fn test_run_simulation_find_best_act_for_charging() {
        let config = Config::default();
        // agent has 3 charging activities, but should choose to charge at last one as this activity charges
        let person = Person {
            attributes: HashMap::<String, String>::default(),
            trace: quick_trace!([
                (L, "a", 1, 2, 1., 0, 0),
                (A, "home", 2, 4, 0, 0),
                (L, "b", 4, 5, 1., 1, 1),
                (A, "home", 5, 7, 0, 0),
                (L, "c", 7, 8, 1., 2, 2),
                (A, "home", 8, 12, 0, 0)
            ]),
        };
        let battery_spec = BatterySpecBuilder::new()
            .capacity(10.0) // kWh -> 36000 kWs
            .full()
            .consumption_rate(1000. / 3.6) // 1000 kWs/m
            .build();
        let trigger_spec = TriggerSpec::empty();
        let en_route_spec = EnRouteSpec::new(Some("enroute".to_string()), 1000.0, None, None);
        let charge_act = ActivitySpec::new(
            Some("home".to_string()),
            vec!["home".to_string()],
            1000.0,
            None,
            None,
        );
        let agent_config: AgentConfig = AgentConfig {
            pid: "A",
            battery: Some(&battery_spec),
            trigger: Some(&trigger_spec),
            en_route: Some(&en_route_spec),
            activities: vec![&charge_act],
        };
        let charge_activity_configs = ActivityChargingPlanner::new(vec![&charge_act]);
        let solution = run_simulations(
            "A",
            &person,
            &agent_config,
            charge_activity_configs,
            &config,
        );
        let mut expected_record = AgentSimulationRecord::new("A", 1.0);
        expected_record.new_day(36000.);
        expected_record.add_event(Event::activity(
            "A",
            Some("home".to_string()),
            3000.,
            1,
            (8, 11),
            "home",
            (0.0, 0.0),
        ));
        expected_record.try_to_close(36000.0);
        assert_eq!(solution, Some(expected_record))
    }

    #[test]
    fn test_run_simulation_requires_en_route() {
        let config = Config::default();
        // agent has 2 charging activities, but will also require a route charge of 1 unit
        // this pattern takes a while to resolve but we can check the total charge is 3 units
        // the exact amount of en-route versus activity depends on the plan sequence
        let person = Person {
            attributes: HashMap::<String, String>::default(),
            trace: quick_trace!([
                (L, "a", 1, 2, 1., 0, 0),
                (A, "home", 2, 3, 0, 0),
                (L, "b", 3, 4, 1., 1, 1),
                (A, "home", 4, 5, 0, 0),
                (L, "c", 5, 6, 1., 2, 2)
            ]),
        };
        let battery_spec = BatterySpecBuilder::new()
            .capacity(3.0 / 3600.) // kWh -> 3 kWs
            .full()
            .consumption_rate(1. / 3.6) // 1 kWs/m
            .build();
        let trigger_spec = TriggerSpec::empty();
        let en_route_spec = EnRouteSpec::new(None, 1.0, None, None);
        let charge_act = ActivitySpec::new(None, vec!["home".to_string()], 1.0, None, None);
        let agent_config: AgentConfig = AgentConfig {
            pid: "A",
            battery: Some(&battery_spec),
            trigger: Some(&trigger_spec),
            en_route: Some(&en_route_spec),
            activities: vec![&charge_act],
        };
        let charge_activity_configs = ActivityChargingPlanner::new(vec![&charge_act]);
        let mut simulation_record = run_simulations(
            "A",
            &person,
            &agent_config,
            charge_activity_configs,
            &config,
        )
        .unwrap();
        simulation_record.finalise(&Config::default());
        assert_eq!(simulation_record.get_total_charge(), 3.0 / 3600.)
    }

    #[test]
    fn test_run_simulation_with_activity_indifference() {
        let config = Config::default();
        // agent has 2 charging activities, either of which is sufficient for plan,
        // agent should choose later activity as this is generally assumed to be at home
        let person = Person {
            attributes: HashMap::<String, String>::default(),
            trace: quick_trace!([
                (L, "a", 1, 2, 1., 0, 0),
                (A, "work", 2, 4, 0, 0),
                (L, "b", 4, 5, 1., 1, 1),
                (A, "home", 5, 7, 0, 0)
            ]),
        };
        let battery_spec = BatterySpecBuilder::new()
            .capacity(3.0 / 3600.) // kWh -> 3 kWs
            .full()
            .consumption_rate(1. / 3.6) // 1 kWs/m
            .build();
        let trigger_spec = TriggerSpec::empty();
        let charge_spec_home = ActivitySpec::new(None, vec!["home".to_string()], 1.0, None, None);
        let charge_spec_work = ActivitySpec::new(None, vec!["work".to_string()], 1.0, None, None);
        let charge_activity_configs =
            ActivityChargingPlanner::new(vec![&charge_spec_home, &charge_spec_work]);
        let en_route_spec = EnRouteSpec::new(None, 1.0, None, None);
        let agent_config: AgentConfig = AgentConfig {
            pid: "A",
            battery: Some(&battery_spec),
            trigger: Some(&trigger_spec),
            en_route: Some(&en_route_spec),
            activities: vec![&charge_spec_home, &charge_spec_work],
        };
        let mut simulation_record = run_simulations(
            "A",
            &person,
            &agent_config,
            charge_activity_configs,
            &config,
        )
        .unwrap();
        simulation_record.finalise(&Config::default());
        let charge_event = &simulation_record.slice()[0].events[0];
        assert_eq!(charge_event.activity, Some("home"));
        assert_eq!(charge_event.charge, 2.0);
        assert_eq!(simulation_record.error, Some(0.0));
    }
}
