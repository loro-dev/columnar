extern crate rand;

use rand::{Rng, thread_rng};
use rand::distributions::{Alphanumeric, DistString};

#[derive(Debug)]
pub enum Action {
    Insert,
    Delete,
    Redo,
    Undo,
}

#[derive(Debug)]
pub struct Op{
    action: Action,
    content: String
}

impl Op{
    pub fn new() -> Self{
        let action = match thread_rng().gen_range(0..4){
            0 => Action::Insert,
            1 => Action::Delete,
            2 => Action::Redo,
            3 => Action::Undo,
            _ => unreachable!()
        };
        Self {
            action,
            content: Alphanumeric.sample_string(&mut thread_rng(), 32)
        }
    }
}