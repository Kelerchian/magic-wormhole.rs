use super::event::Events;
// we process these
use super::event::ListerEvent;
// we emit these
use super::event::InputEvent::GotNameplates as I_GotNameplates;
use super::event::ListerEvent::*;
use super::event::RendezvousEvent::TxList as RC_TxList;

pub struct ListerMachine {
    state: State,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum State {
    // All A's unconnected
    // All B's connected

    // not wanting list unconnected
    S0A,
    // Want list unconnected
    S1A,
    // not wanting list connected
    S0B,

    // want list connected
    S1B,
}

impl ListerMachine {
    pub fn new() -> ListerMachine {
        ListerMachine { state: State::S0A }
    }

    pub fn process(&mut self, event: ListerEvent) -> Events {
        use self::State::*;
        let (newstate, actions) = match self.state {
            S0A => self.do_s0a(&event),
            S0B => self.do_s0b(event),
            S1A => self.do_s1a(&event),
            S1B => self.do_s1b(event),
        };

        self.state = newstate;
        actions
    }

    fn do_s0a(&self, event: &ListerEvent) -> (State, Events) {
        match *event {
            Connected => (State::S0B, events![]),
            Refresh => (State::S1A, events![]),
            _ => (State::S0A, events![]),
        }
    }

    fn do_s0b(&self, event: ListerEvent) -> (State, Events) {
        match event {
            Refresh => (State::S1B, events![RC_TxList]),
            Lost => (State::S0A, events![]),
            RxNameplates(nids) => {
                (State::S0B, events![I_GotNameplates(nids.clone())])
            }
            Connected => (State::S0B, events![]),
        }
    }

    fn do_s1a(&self, event: &ListerEvent) -> (State, Events) {
        match event {
            Connected => (State::S1B, events![RC_TxList]),
            Refresh => (State::S1B, events![RC_TxList]),
            _ => (State::S1B, events![]),
        }
    }

    fn do_s1b(&self, event: ListerEvent) -> (State, Events) {
        match event {
            Lost => (State::S1A, events![]),
            Refresh => (State::S1B, events![RC_TxList]),
            RxNameplates(nids) => {
                (State::S0B, events![I_GotNameplates(nids.clone())])
            }
            Connected => (State::S1B, events![]),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{ListerMachine, State};
    use crate::core::event::{
        InputEvent::GotNameplates, ListerEvent::*, Nameplate,
        RendezvousEvent::TxList,
    };

    #[test]
    fn test_lister() {
        let mut lister = ListerMachine::new();

        assert_eq!(lister.state, State::S0A);

        assert_eq!(lister.process(Connected), events![]);
        assert_eq!(lister.state, State::S0B);

        assert_eq!(lister.process(Lost), events![]);
        assert_eq!(lister.state, State::S0A);

        lister.state = State::S0B;
        let nameplates: Vec<Nameplate> = vec!["3"]
            .into_iter()
            .map(|s| Nameplate(s.to_string()))
            .collect();
        assert_eq!(
            lister.process(RxNameplates(nameplates.clone())),
            events![GotNameplates(nameplates)]
        );
        assert_eq!(lister.state, State::S0B);

        assert_eq!(lister.process(Refresh), events![TxList]);
        assert_eq!(lister.state, State::S1B);

        assert_eq!(lister.process(Refresh), events![TxList]);
        assert_eq!(lister.state, State::S1B);
    }
}
