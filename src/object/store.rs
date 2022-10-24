use std::collections::HashMap;

use rand::Rng;

use super::{ClientID, change::Change};

#[derive(Debug)]
pub struct Store{
    changes: HashMap<ClientID, Change>
}

impl Store{
    pub fn new(client_num: usize) -> Self{
        let mut changes = HashMap::new();
        for i in 0..client_num{
            let op_num = rand::thread_rng().gen_range(1..5);
            changes.insert(i as ClientID, Change::new(op_num, i as ClientID));
        }
        Self { changes }
    }
}

mod test{
    use super::*;

    #[test]
    fn test_store_new(){
        let store = Store::new(1);
        println!("{:?}", store);
    }
}