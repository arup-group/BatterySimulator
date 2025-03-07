# Configuration

Batsim uses a yaml formatted configuration to parameterise each `batsim optimise' run. The config is broken into four groups:

- **General**
- **Battery Group** - *which* agents have access to *what* vehicle battery technology
- **En Route Group** - *which* agents have access to *what* en-route/network charge speed
- **Activity Group** - *which* agents have access to *what* charging at different activities in their plans

## Dry Run Command

Batsim configurations can get arbitrarily complex. Knowing how a config file will work with your population can be checked using the `batsim dryrun` command:

```{.sh}
❯ batsim dryrun --help
```

```{.sh}
Dry run agent configurations

Usage: batsim dryrun [OPTIONS]

Options:
  -c, --config <CONFIG>          Config path
  -t, --trace-path <TRACE_PATH>  Path to traces file [default: traces.json]
  -o, --output <OUTPUT>          Output file path [default: config.csv]
  -j, --json                     Read traces from human readable json format
  -h, --help                     Print help information
  -V, --version                  Print version information
```

This outputs a csv log of which specifications have been assigned to each agent using the specification `name` fields. This is a good reason to use the optional `name` field for your specifications.

## General

```{.yaml}
name: demo_config
scale: 1.0
precision: 1.0  # kWs
patience: 100
seed: 1234
```

- **name**: optional field for naming your config file
- **scale**: optional field for scaling all outputs (charge sizes and charge counts), defaults to 1
- **precision**: optional field for setting simulation 'sequence closing' precision, larger numbers improve the likelihood of agents finding "closed" or "cyclical" charging plans, but allow more energy 'leaking', defaults to 1.0 kWs
- **patience**: optional field for setting simulation patience, larger numbers improve the likelihood of agents finding "closed" or "cyclical" charging plans, defaults to 100
- **seed**: optional field for using a random seed, can be used for reproducibility if applying probabilities (`p`) to specifications

## Battery Group Configuration

```{.yaml}
battery_group:
- name: default
  capacity: 100  // maximum battery charge, kWh
  initial: 100  // initial state of charge, kWh
  consumption_rate: 0.15 // rate at which agent will run down charge, kWh per km
```

Simulating agent charging behaviour requires those agents to have access to batteries, the above is the default battery specification. Which is applied to all agents in the population. If you would like to only give some agents access to an EV, and or give different agents different batteries, you will need to use more [advanced configuration](#advanced-configuration).

- **name**: optional field for naming a battery specification within the battery group
- **capacity**: maximum battery charge, defaults to 100kWh
- **initial**: initial battery state, defaults to 100kWh (full)
- **consumption_rate**: rate at which agent will run down charge, kWs per m, defaults to 0.15 kWh per km (~7 km per kWh)

## Trigger Group Specification

```{.yaml}
trigger_group:
- name: default
  trigger: 0.2  // proportion of capacity at which agent will enroute charge
```

Agents with batteries will also be given some risk aversion that causes them to 'seek' a charge when their battery runs low. We define this as a trigger value, which is the proportion of battery capacity at which an agent will make an enroute charge.

- **name**: optional field for naming a trigger specification within the group
- **trigger**: battery level at which agent triggers an *EnRouteChargeEvent*, agents will seek to minimise these, defaults to 0.2 (20% full)

We keep trigger specification in a separate group from batteries to allow more flexibility in configuration.

## En-Route Group Configuration

```{.yaml}
enroute_group:
- name: default
  charge_rate: 10  // en-route station charging rate, kW
```

Simulations require all agents with a battery (an EV) to have access to enroute (or "network") charging, the above is the default enroute specification.

- **name**: optional field for naming an enroute specification within the en-route group
- **charge_rate**: en-route station charging rate, kW, default 10kW

## Activity Group Configuration

```{.yaml}
activity_group:
- name: default
  activities: [home]
  charge_rate: 3.0  // charge rate available at home activity, kW
```

Simulations do not require agents to have access to any activity charging facilities, the default group is therefore empty. In such a case, all agents will only have access to en-route charging.

The above example specification gives all agents access to a 3 kW charger at home.

- **name**: optional field for naming an activity specification within the activity group
- **activities**: list of activity types for which charging is available, default is "home"
- **charge_rate**: rate of charge, kW

## Advanced Configuration

Specifications within each group can be given **filters**. A specification with a filter is only applied to agents with an attribute key value pair that matches the filter. Specifications without filters will match all agents and can be thought of as "defaults". Agent attributes are extracted from the original MATSim input population. For example, a filter can be used to only give batteries to agents who own an EV:

```{.yaml}
battery_group:
- name: regular
  capacity: 100.0
  initial: 100.0
  consumption_rate: 0.15
  filters:
  - {key: owns_ev, values: [true]}
```

Specifications can be stacked within each group, for example, some agents can be given larger capacity batteries:

```{.yaml}
battery_group:
- name: regular
  capacity: 100.0
  initial: 100.0
  consumption_rate: 0.15
  filters:
  - {key: owns_ev, values: [true}

- name: large
  capacity: 150.0
  initial: 150.0
  consumption_rate: 0.2
  filters:
  - {key: owns_ev, values: [true}
  - {key: veh_size, values: [medium, large]}
```

The `battery_group` specifications are read in order, such that firstly, agents who own an EV are assigned the above `regular` battery, then secondly, agents with matching attribute key and values to the "large battery" filter are **re-assigned** with the `large` battery. Specifications and filters can be further stacked to provide highly heterogenous configuration based on the available agent attributes.
_____

It is also possible to stack multiple filters (**all** filters must be true for an agent to be assigned a specification - you can think of this as **AND** logic). The `enroute_group` works the same way:

```{.yaml}
enroute_group:
- name: default
  charge_rate: 3

- name: rapid
  charge_rate: 10
  filters:
  - {key: model, values: [rapid]}
  - {key: income, values: [high, medium]}
```

The above en-route group gives access to default 3kW en-route chargers for all agents (because this specification has no filters, it is applied to all agents). Then assigns an improved 10kW en-route charger, but only to agents with a "rapid" charge "model" **and** who are in the "high" or "medium" "income" groups.
_____

The `activity_group` is slightly different, rather than overwriting, agents are assigned **all** matching configurations. However, the order is still important, because where a charging specification for an activity is duplicated for an agent, this will be re-assigned:

```{.yaml}
...

activity_group:
- name: home_charger
  activities: [home]
  charge_rate: 3.0  // re-charge rate available at home activity, kW
  filters:
  - {key: home_type, values: [detached, demi-detached]}

- name: rapid_home_charger
  activities: [home]
  charge_rate: 10.0  // re-charge rate available at home activity, kW
  filters:
  - {key: home_type, values: [detached, demi-detached]}
  - {key: income, values: [high, medium]}

- name: work_charger
  activities: [work, business]
  charge_rate: 3.0  // re-charge rate available at home activity, kW
  p: 0.5
  filters:
  - {key: occupation, values: [type_a, type_b]}
```

Note that it is also possible to set a probability (using `p`) that a specification is available (the sampling is applied after the filters).

## Attributes Command

BATSim also provides a convenience command to quickly check what person attribute key-values are available in an input MATSim population:

```{.sh}
batsim attributes --help
```

```{.sh}
Peek attributes in a plans file

Usage: batsim attributes [OPTIONS]

Options:
  -p, --plans <PLANS>  Path to MATSim xml plans to peek [default: output_plans.xml]
  -m, --max <MAX>      Max number of attribute values to show [default: 10]
  -h, --help           Print help information
  -V, --version        Print version information
```
