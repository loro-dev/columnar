use smallvec::SmallVec;

use super::{op::Op, ID, ClientID};

#[derive(Debug)]
pub struct Change{
    ops: Vec<Op>,
    id: ID,
    deps: SmallVec<[ID; 2]>
}

impl Change {
    pub fn new(op_num: usize, client_id: ClientID) -> Self {
        let mut ops = Vec::with_capacity(op_num);
        for _ in 0..op_num{
            ops.push(Op::new());
        }
        let mut deps = SmallVec::new();
        deps.push(ID::new(client_id, 0));
        Self {
            ops,
            id: ID::new(client_id, 0),
            deps
        }
    }
}