pub mod reason;
pub mod error;
pub use reason::DeadReason;
pub use error::Error;

use crate::app::{interface, network};

use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;


pub static STATE: Lazy<Arc<RwLock<Operator>>> = Lazy::new(
    || Arc::new(RwLock::new(Operator::default()))
);



#[derive(Debug, Clone)]
pub enum State {
    Dead(DeadReason),
    Alive
}

pub struct Operator {
    state: State,
}
impl Operator {
    pub fn get(&self) -> &State {
        &self.state
    }

    fn set(&mut self, state: State) {
        self.state = state
    }

    pub fn get_dead_reason(&self) -> Option<&DeadReason> {
        match &self.state {
            State::Alive => None,
            State::Dead(reason) => Some(reason)
        }
    }

    pub fn can_die(&self) -> bool {
        match &self.state {
            State::Alive => true,
            State::Dead(reason) => match reason {
                DeadReason::Uninitialized => true,
                _ => false,
            }
        }
    }

    fn print_message(&self) {
        match &self.state {
            State::Alive => {},
            State::Dead(reason) => {
                match reason {
                    DeadReason::Uninitialized => {
                        println!("! DEAD: uninitialized");
                        println!("? FIX: check that the app's variables are initialized correctly");
                    },
                    DeadReason::NoInterface => {
                        println!("! DEAD: no available wireless interfaces");
                        println!("? FIX: connect at least one wireless interface (USB, PCIe, virtual, etc.)");
                    },
                    DeadReason::NoNetwork => {
                        println!("! DEAD: could not connect to any available network");
                        println!("? FIX: check that at least one network defined in config is reachable");
                    },
                }
            }
        }
    }

    pub async fn choose(&mut self) -> &State {
        if interface::LIST.read().await.is_empty() {
            println!("app state operator calls dead");
            self.dead(DeadReason::NoInterface).unwrap();
        } else if !network::LIST.read().await.cfg_networks_available() {
            println!("app state operator calls dead");
            self.dead(DeadReason::NoNetwork).unwrap();
        } else if !self.is_alive() {
            self.alive().unwrap();
        }

        self.get()
    }

    pub fn dead(&mut self, reason: DeadReason) -> Result<(), Error> {
        if !self.can_die() {
            return Err(Error::AlreadyDead)
        }

        self.set(State::Dead(reason.clone()));

        self.print_message();

        Ok(())
    }

    pub fn alive(&mut self) -> Result<(), Error> {
        if self.is_alive() {
            return Err(Error::AlreadyAlive)
        }

        self.set(State::Alive);
        println!("o APP ALIVE");

        Ok(())
    }

    pub fn is_dead(&self) -> bool {
        match self.state {
            State::Dead(_) => true,
            State::Alive => false,
        }
    }

    pub fn is_alive(&self) -> bool {
        !self.is_dead()
    }
}
impl Default for Operator {
    fn default() -> Self {
        Self { state: super::State::Dead(DeadReason::Uninitialized) }
    }
}