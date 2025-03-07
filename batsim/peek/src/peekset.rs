use std::collections::BTreeSet;
use std::fmt::Display;
use std::hash::Hash;

#[derive(Debug)]
pub enum PeekerState {
    Peeking,
    Full,
}
#[derive(Debug)]
pub struct PeekSet<T> {
    max: usize,
    state: PeekerState,
    memory: BTreeSet<T>,
}
impl<T> PeekSet<T>
where
    T: Eq + Ord + Display,
{
    pub fn new(max: usize) -> Self {
        PeekSet {
            max,
            state: PeekerState::Peeking,
            memory: BTreeSet::<T>::new(),
        }
    }
    pub fn insert(&mut self, k: T) {
        self.state = match self.state {
            PeekerState::Full => PeekerState::Full,
            PeekerState::Peeking if self.memory.len() < self.max => {
                self.memory.insert(k);
                PeekerState::Peeking
            }
            _ => PeekerState::Full,
        }
    }
}
impl<T> std::fmt::Display for PeekSet<T>
where
    T: Display + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.state {
            PeekerState::Peeking => {
                let mut iter = self.memory.clone().into_iter();
                if let Some(s) = iter.next() {
                    write!(f, "{}", s)?
                }

                for v in iter {
                    write!(f, ", {}", v)?
                }
                Ok(())
            }
            PeekerState::Full => {
                for v in &self.memory {
                    write!(f, "{}, ", v)?;
                }
                write!(f, "...")?;
                Ok(())
            }
        }
    }
}
impl<K> FromIterator<K> for PeekSet<K>
where
    K: Eq + Ord + Display,
{
    fn from_iter<T: IntoIterator<Item = K>>(iter: T) -> PeekSet<K> {
        let mut set = PeekSet::new(10);
        set.memory.extend(iter);
        set
    }
}
impl<K> PartialEq for PeekSet<K>
where
    K: Eq + Hash + Display,
{
    fn eq(&self, other: &Self) -> bool {
        self.memory == other.memory
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peekset() {
        let mut set = PeekSet::new(3);
        set.insert("a");
        set.insert("a");
        set.insert("b");
        assert_eq!(
            set,
            PeekSet {
                max: 1,
                state: PeekerState::Full,
                memory: BTreeSet::from(["a", "b"])
            }
        );
    }

    #[test]
    fn test_peekset_full() {
        let mut set = PeekSet::new(1);
        set.insert("a");
        set.insert("b");
        assert_eq!(
            set,
            PeekSet {
                max: 1,
                state: PeekerState::Full,
                memory: BTreeSet::from([("a")])
            }
        );
    }

    #[test]
    fn test_peekset_full_zero() {
        let mut set = PeekSet::new(0);
        set.insert("a");
        set.insert("b");
        assert_eq!(
            set,
            PeekSet {
                max: 1,
                state: PeekerState::Full,
                memory: BTreeSet::from([])
            }
        );
    }
}
