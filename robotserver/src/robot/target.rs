use crate::robot::miner::Miner;

pub trait TargetAction: Send + 'static {
    fn process(&mut self, miner: &mut Miner);

    fn activate(&self, miner: &mut Miner) -> bool;

    fn terminate(&mut self, miner: &mut Miner);
}

#[derive(Default)]
pub struct Cell {
    index: u32,
    user_id: u32,
}

impl TargetAction for Cell {
    fn process(&mut self, miner: &mut Miner) {
        if self.user_id > 0 {
            return;
        }
        self.user_id = miner.id;
    }

    fn activate(&self, miner: &mut Miner) -> bool {
        unimplemented!()
    }

    fn terminate(&mut self, miner: &mut Miner) {
        unimplemented!()
    }
}
