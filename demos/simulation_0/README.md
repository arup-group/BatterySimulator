# Simulation 0

Very simple network and single agent with home -> work -> home plan. Outbound and return trip distances are both 10km.

## Scenario A

Agent may charge at home, but their battery trigger level is set such that they will en-route charge on their return trip each day.

We run this simulation using the `scenario_a.yaml` config;

```
name: scenario_a
scale: 1
patience: 10

battery_group:
- name: default
  capacity: 20  # kWh
  initial: 20  # kWh
  trigger: 5  # kWh
  consumption_rate: 1.0  # kWh/km
  filters:
  - {key: vehicle_type, values: [ev]}

enroute_group:
- name: default
  charge_rate: 10  # kW

activity_group:
- name: at_home
  activities: [home]
  charge_rate: 3  # kW
```

`batsim run -d data -c scenario_a.yaml -o output_scenario_a`

> All paths are assuming you are at the root of this readme (`batsim/demos/simulation_0`).

The expected output is 2 charges totalling 20KWs, 10kWh from an en-route charge on the return trip and 10kWh once at home after:

```
Total Charge: 20 kWh
Total Events: 2
Total Energy Leak: 0 kWs

[En Route Charging]
Total En-route Charge: 10 kWh
Total En-route Charge Events: 1

[Activity Charging]
Total Activity Charge: 10 kWh
Total Activity Charge Events: 1

[Charging by activity]
home: 10 kWh from 1 charge events
```

## Scenario B

We imagine that the agent has no access to charging at home. So may only charge en-route. We remove access to charging at home by adding a filter, which our agent does not match:

```
name: scenario_b
scale: 1
patience: 10

battery_group:
- name: default
  capacity: 20  # kWh
  initial: 20  # kWh
  trigger: 5  # kWh
  consumption_rate: 1  # kWh/km
  filters:
  - {key: vehicle_type, values: [ev]}

enroute_group:
- name: default
  charge_rate: 10  # kW

activity_group:
- name: at_home
  activities: [home]
  charge_rate: 3  # kW
  filters:
  - {key: house_type, values: [detached, demi-detached]}
```

`batsim run -d data -c scenario_b.yaml -o output_scenario_b`

We expect a single (daily) en-route charge of 20kWh from an en-route charge on the return trip. Note that we could could use a lower initial battery state which would trigger a charge on the out-bound trips instead:

```
Total Charge: 20 kWh
Total Events: 1
Total Energy Leak: 0 kWs

[En Route Charging]
Total En-route Charge: 20 kWh
Total En-route Charge Events: 1

[Activity Charging]
Total Activity Charge: 0 kWs
Total Activity Charge Events: 0
```

## Scenario C

We imagine that the agent has access to charging at home and at work, we know that a full charge is not enough to support their daily plan, so we expect them to charge both at home and at work so that they can avoid an en-route charge.

We do this by adding a new component to the activity group named `at_work`:

```
name: scenario_c
scale: 1
patience: 10

battery_group:
- name: default
  capacity: 20  # kWh
  initial: 20  # kWh
  trigger: 5  # kWh
  consumption_rate: 1  # kWh/km
  filters:
  - {key: vehicle_type, values: [ev]}

enroute_group:
- name: default
  charge_rate: 10  # kW

activity_group:
- name: at_home
  activities: [home]
  charge_rate: 3  # kW

- name: at_work
  activities: [work, business]
  charge_rate: 10  # kW
```

`batsim run -d data -c scenario_c.yaml -o output_scenario_c`

Agent now charges both at home and at work:

```
Total Charge: 20 kWh
Total Events: 2
Total Energy Leak: 0 kWs

[En Route Charging]
Total En-route Charge: 0 kWs
Total En-route Charge Events: 0

[Activity Charging]
Total Activity Charge: 20 kWh
Total Activity Charge Events: 2

[Charging by activity]
work: 10 kWh from 1 charge events
home: 10 kWh from 1 charge events
```