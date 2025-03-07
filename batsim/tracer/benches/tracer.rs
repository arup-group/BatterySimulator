use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;

use tracer::{network::Network, population::Population, MATSimEventsReader, TraceHandler};

pub fn build_network(c: &mut Criterion) {
    c.bench_function("tracer command", |b| {
        b.iter(|| {
            let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            path.push("tests/data/output_network.xml");
            let mut reader = xml::reader(&path).unwrap();
            let _ = Network::from_xml(black_box(&mut reader));
        })
    });
}

pub fn build_population(c: &mut Criterion) {
    c.bench_function("tracer command", |b| {
        b.iter(|| {
            let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            path.push("tests/data/output_plans.xml");
            let mut reader = xml::reader(&path).unwrap();
            let _ = Population::from_xml(black_box(&mut reader));
        })
    });
}

pub fn build_traces(c: &mut Criterion) {
    c.bench_function("tracer command", |b| {
        b.iter(|| {
            let mut network_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            network_path.push("tests/data/output_network.xml");
            let mut network_reader = xml::reader(&network_path).unwrap();
            let network = Network::from_xml(black_box(&mut network_reader)).unwrap();

            let mut population_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            population_path.push("tests/data/output_plans.xml");
            let mut population_reader = xml::reader(&population_path).unwrap();
            let mut population = Population::from_xml(black_box(&mut population_reader)).unwrap();

            let mut events_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            events_path.push("tests/data/output_events.xml");
            let events_reader = xml::reader(&events_path).unwrap();

            let mut tracer = TraceHandler::new();
            let mut events = MATSimEventsReader::from_xml(events_reader);
            tracer.add_network(&network);
            let _ = tracer.add_traces(&mut population, &mut events);
        })
    });
}

criterion_group!(
    tracer_benchmarks,
    build_network,
    build_population,
    build_traces
);
criterion_main!(tracer_benchmarks);
