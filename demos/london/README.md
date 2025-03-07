# Simulation London

Very small network and 100 synthetic agents with various attributes, and sequences of activites and modes. We supply the traces in `demos/london/data` only.

## Scenario - Low EV

We first use filters to give 50% of high income agents an EV. All agents are able to charge at home. We assume the input is a 10% sample so scale all outputs by 10. We add these probabilities we add a `p` key to our `high_income` battery group component:

```
name: low_ev
scale: 10
patience: 100
seed: 1234

battery_group:
- name: high_income
  capacity: 100  # kWh
  initial: 100  # kWh
  trigger: 20  # kWh
  consumption_rate: 0.15  # kWh/km
  filters:
    - {key: subpopulation, values: [high income]}
  p: 0.5  # 50% probability

enroute_group:
- name: default
  charge_rate: 10  # kW

activity_group:
- name: at_home
  activities: [home]
  charge_rate: 3  # kW
```

`batsim optimise -t data/traces.json -c scenario_low_ev.yaml -o output_low_ev --json`

> All paths are assuming you are at the root of this readme (`batsim/demos/london`).

> We are using the `optimise` command (rather than `run`) because the traces are already supplied in the data directory.

> We use the `--json` flag because the traces are serialised in human-readable json format.

You should some level of charging happen in simulation. Note that we are using sampling probabilities in the config so results may not be reproducible:

```
Total Charge: 61 kWh
Total Events: 70
Total Energy Leak: 0 kWs

[En Route Charging]
Total En-route Charge: 0 kWs
Total En-route Charge Events: 0

[Activity Charging]
Total Activity Charge: 61 kWh
Total Activity Charge Events: 70

[Charging by activity]
home: 61 kWh from 70 charge events
```

You can find detailed event and agent level outputs in the output directory: `./output_low_ev`.

> If your projects requires a more deterministic/reproducible/explainable analysis, we suggest explicitly modelling and assigning vehicle ownership (for example) in the input synthetic population.

> To see which agents were assigned batteries we can check `specs.csv` in the outputs directories. Specs are also available without running the sim using the `batsim config` command - `batsim config -t data/traces.json -c scenario_low_ev.yaml -o output_low_ev/config.csv --json`.

## Scenario - High EV

We now use the filters to give 90% of high income agents an EV and 80% of medium income agents an EV, and 50% of low income agents an EV. All agents are able to charge at home. We assume the input is a 10% sample so scale all outputs by 10:

```
name: high_ev
...

battery_group:
- name: high_income
  capacity: 100  # kWh
  initial: 100  # kWh
  trigger: 20  # kWh
  consumption_rate: 0.15  # kWh/km
  filters:
    - {key: subpopulation, values: [high income]}
  p: 0.9

- name: med_income
  capacity: 100  # kWh
  initial: 100  # kWh
  trigger: 20  # kWh
  consumption_rate: 0.15  # kWh/km
  filters:
    - {key: subpopulation, values: [medium income]}
  p: 0.8

- name: low_income
  capacity: 100  # kWh
  initial: 100  # kWh
  trigger: 20  # kWh
  consumption_rate: 0.15  # kWh/km
  filters:
    - {key: subpopulation, values: [low income]}
  p: 0.5

enroute_group:
...

activity_group:
...
```

`batsim optimise -t data/traces.json -c scenario_high_ev.yaml -o output_high_ev --json`

We expect to see a significant increase in total demand for energy as agents are more likely to own an EV.

```
Total Charge: 294 kWh
Total Events: 310
Total Energy Leak: 0 kWs

[En Route Charging]
Total En-route Charge: 0 kWs
Total En-route Charge Events: 0

[Activity Charging]
Total Activity Charge: 294 kWh
Total Activity Charge Events: 310

[Charging by activity]
home: 294 kWh from 310 charge events
```

Note that lower income agents tend to travel less far (although not necessarily in this toy example data). So already we might find some useful insights looking through the outputs.

## Scenario - High EV High Inequality

Finally we start to consider the impact of increased heterogeneity for different agents:

```
name: high_ev_high_inequality
...

battery_group:
...

enroute_group:
- name: default
  charge_rate: 10  # kW

activity_group:
- name: default_home
  activities: [home]
  charge_rate: 3  # kW
  p: 0.5

- name: off_street
  activities: [home]
  charge_rate: 3  # kW
  filters:
    - {key: subpopulation, values: [high income, medium income]}
    - {key: household_LAD, values: [Kensington and Chelsea, Westminster]}

- name: work
  activities: [work]
  charge_rate: 11  # kW
  filters:
    - {key: subpopulation, values: [high income, medium income]}
```

- high income agents have batteries with higher capacities.
- high and medium income agents living in chosen boroughs are given guaranteed access to charging at home, perhaps as a consequence of improved on street installation.
- high and medium income agents are also given access to fast charging at work activities.

`batsim optimise -t data/traces.json -c scenario_high_ev_high_inequality.yaml -o output_high_inequality --json`

```
Total Charge: 270 kWh
Total Events: 270
Total Energy Leak: 0 kWs

[En Route Charging]
Total En-route Charge: 123 kWh
Total En-route Charge Events: 90

[Activity Charging]
Total Activity Charge: 147 kWh
Total Activity Charge Events: 180

[Charging by activity]
home: 121 kWh from 150 charge events
work: 26 kWh from 30 charge events
```

The above are for demonstrative purposes only. We leave it as an exercise to the reader to interogate the resulting outputs (in `./output_high_inequality/`) to fully inspect the consequences of these inequalities.
