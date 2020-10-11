use crate::robot::{BattleCharacter, Robot};
use crate::status::{EnterMineAndDigForNugget, Status};

pub mod robot;

pub mod status;

fn main() {
    let mut bc = BattleCharacter::default();
    let e = EnterMineAndDigForNugget {
        status: Status::EnterMineAndDigForNugget,
    };
    bc.change_status(Box::new(e));
}
