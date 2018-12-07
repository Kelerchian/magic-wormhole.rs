use super::event::{Events, Key, MySide, Phase};
use super::key;
// we process these
use super::event::SendEvent;
// we emit these
use super::event::MailboxEvent::AddMessage as M_AddMessage;

pub struct SendMachine {
    state: State,
    side: MySide,
    queue: Vec<(Phase, Vec<u8>)>,
}

#[derive(Debug, PartialEq)]
enum State {
    S0NoKey,
    S1HaveVerifiedKey(Key),
}

enum QueueStatus {
    Enqueue((Phase, Vec<u8>)),
    Drain,
    NoAction,
}

impl SendMachine {
    pub fn new(side: &MySide) -> SendMachine {
        SendMachine {
            state: State::S0NoKey,
            side: side.clone(),
            queue: Vec::new(),
        }
    }

    pub fn process(&mut self, event: SendEvent) -> Events {
        println!(
            "send: current state = {:?}, got event = {:?}",
            self.state, event
        );
        let (newstate, actions, queue_status) = match self.state {
            State::S0NoKey => self.do_s0(event),
            State::S1HaveVerifiedKey(ref key) => self.do_s1(&key, event),
        };

        // process the queue
        match queue_status {
            QueueStatus::Enqueue(tup) => self.queue.push(tup),
            QueueStatus::Drain => {
                self.queue = Vec::new();
            }
            QueueStatus::NoAction => (),
        };

        self.state = newstate;
        actions
    }

    fn drain(&self, key: &Key) -> Events {
        let mut es = Events::new();

        for &(ref phase, ref plaintext) in &self.queue {
            let data_key = key::derive_phase_key(&self.side, &key, phase);
            let (_nonce, encrypted) = key::encrypt_data(&data_key, plaintext);
            es.push(M_AddMessage(phase.clone(), encrypted));
        }

        es
    }

    fn deliver(&self, key: &Key, phase: Phase, plaintext: &[u8]) -> Events {
        let data_key = key::derive_phase_key(&self.side, &key, &phase);
        let (_nonce, encrypted) = key::encrypt_data(&data_key, plaintext);
        events![M_AddMessage(phase, encrypted)]
    }

    fn do_s0(&self, event: SendEvent) -> (State, Events, QueueStatus) {
        use super::event::SendEvent::*;
        match event {
            GotVerifiedKey(ref key) => (
                State::S1HaveVerifiedKey(key.clone()),
                self.drain(key),
                QueueStatus::Drain,
            ),
            // we don't have a verified key, yet we got messages to send, so queue it up.
            Send(phase, plaintext) => (
                State::S0NoKey,
                events![],
                QueueStatus::Enqueue((phase, plaintext)),
            ),
        }
    }

    fn do_s1(
        &self,
        key: &Key,
        event: SendEvent,
    ) -> (State, Events, QueueStatus) {
        use super::event::SendEvent::*;
        match event {
            GotVerifiedKey(_) => panic!(),
            Send(phase, plaintext) => {
                let deliver_events = self.deliver(&key, phase, &plaintext);
                (
                    State::S1HaveVerifiedKey(key.clone()),
                    deliver_events,
                    QueueStatus::NoAction,
                )
            }
        }
    }
}
