use crate::robot::robot_status::RobotStatus;
use crate::room::character::BattleCharacter;

pub trait RobotStatusAction: Send + 'static {
    fn enter(&self, cter: &mut BattleCharacter);
    fn execute(&self, cter: &mut BattleCharacter);
    fn exit(&self, cter: &mut BattleCharacter);
    fn get_status(&self) -> RobotStatus;
}
