use std::ops::{Deref, DerefMut};

use crate::events::Event;

#[derive(PartialEq, Debug, Default)]
pub struct Day<'a> {
    pub events: Vec<Event<'a>>,
}

impl<'a> Day<'a> {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }
    pub fn iter_events(&'a self) -> std::slice::Iter<'a, Event<'a>> {
        self.events.iter()
    }
}

impl<'a> std::iter::IntoIterator for &'a Day<'a> {
    type Item = <std::slice::Iter<'a, Event<'a>> as Iterator>::Item;
    type IntoIter = std::slice::Iter<'a, Event<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.events.iter()
    }
}

impl<'a> Deref for Day<'a> {
    type Target = Vec<Event<'a>>;
    fn deref(&self) -> &Self::Target {
        &self.events
    }
}

impl<'a> DerefMut for Day<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day() {
        Day::new();
    }

    fn day<'a>() -> Day<'a> {
        Day {
            events: vec![
                Event::activity("a", None, 0., 0, (0, 1), "home", (0., 0.)),
                Event::en_route("a", None, 0., 0, (0, 1), "a", (0., 0.)),
            ],
        }
    }

    #[test]
    fn iter_events() {
        println!("{:?}", day().iter_events().collect::<Vec<&Event>>())
    }
}
