use super::*;
use crate::robot::robot_skill::robot_use_skill;
use log::warn;

#[derive(Default)]
pub struct UseSkillRobotAction {
    pub robot_id: u32,
    pub cter_id: u32,
    pub battle_data: Option<*const BattleData>,
    pub status: RobotStatus,
    pub sender: Option<Sender<RobotTask>>,
}

get_mut_ref!(UseSkillRobotAction);

impl UseSkillRobotAction {
    pub fn get_battle_data_ref(&self) -> Option<&BattleData> {
        unsafe {
            if self.battle_data.unwrap().is_null() {
                return None;
            }
            Some(self.battle_data.unwrap().as_ref().unwrap())
        }
    }

    pub fn new(battle_data: *const BattleData, sender: Sender<RobotTask>) -> Self {
        let mut use_skill_action = UseSkillRobotAction::default();
        use_skill_action.battle_data = Some(battle_data);
        use_skill_action.sender = Some(sender);
        use_skill_action
    }
}

impl RobotStatusAction for UseSkillRobotAction {
    fn set_sender(&mut self, sender: Sender<RobotTask>) {
        self.sender = Some(sender);
    }

    fn get_cter_id(&self) -> u32 {
        self.cter_id
    }

    fn enter(&self) {
        info!("robot:{} 进入使用技能状态！", self.robot_id);
        self.execute();
    }

    fn execute(&self) {
        let battle_data = self.get_battle_data_ref();
        if battle_data.is_none() {
            warn!("the point *const BattleData is null!");
            return;
        }
        let battle_data = battle_data.unwrap();
        let battle_player = battle_data.battle_player.get(&self.robot_id);
        if let None = battle_player {
            warn!("robot's cter is None!robot_id:{}", self.robot_id);
            return;
        }
        let battle_player = battle_player.unwrap();
        if battle_player.is_died() {
            return;
        }
        let robot = battle_player.robot_data.as_ref().unwrap();
        for skill in battle_player.cter.skills.values() {
            let _ = robot_use_skill(battle_data, skill, robot);
        }
    }

    fn exit(&self) {
        unimplemented!()
    }

    fn get_status(&self) -> RobotStatus {
        self.status
    }

    fn get_robot_id(&self) -> u32 {
        self.robot_id
    }

    fn get_sender(&self) -> &Sender<RobotTask> {
        self.sender.as_ref().unwrap()
    }
}
