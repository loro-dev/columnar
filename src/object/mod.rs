mod change;
mod store;
mod op;

pub type ClientID = u64;
pub type Counter = i32;

#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy)]
pub struct ID {
    pub client_id: ClientID,
    pub counter: Counter,
}

impl ID {
    pub fn new(client_id: ClientID, counter: Counter) -> Self {
        Self {
            client_id,
            counter,
        }
    }
}