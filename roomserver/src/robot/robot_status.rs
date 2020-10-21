use crate::robot::robot_action::RobotStatusAction;
use crate::room::character::BattleCharacter;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

///pos操作类型
#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RobotStatus {
    None = 0,
}

impl RobotStatus {
    pub fn into_u8(self) -> u8 {
        let value: u8 = self.into();
        value
    }
}

impl Default for RobotStatus {
    fn default() -> Self {
        RobotStatus::None
    }
}

#[derive(Default)]
pub struct Attack {
    pub status: RobotStatus,
}

impl RobotStatusAction for Attack {
    fn enter(&self, cter: &mut BattleCharacter) {
        unimplemented!()
    }

    fn execute(&self, cter: &mut BattleCharacter) {
        unimplemented!()
    }

    fn exit(&self, cter: &mut BattleCharacter) {
        unimplemented!()
    }

    fn get_status(&self) -> RobotStatus {
        self.status
    }
}

#[derive(Default)]
pub struct OpenCell {
    pub status: RobotStatus,
}

impl RobotStatusAction for OpenCell {
    fn enter(&self, cter: &mut BattleCharacter) {
        self.execute(cter);
    }

    fn execute(&self, cter: &mut BattleCharacter) {}

    fn exit(&self, cter: &mut BattleCharacter) {
        unimplemented!()
    }

    fn get_status(&self) -> RobotStatus {
        self.status
    }
}
