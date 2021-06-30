use super::*;
use crate::robot::robot_skill::{robot_use_skill, skill_condition, skill_target};
use log::warn;

#[derive(Default)]
pub struct UseSkillRobotAction {
    pub robot_id: u32,
    pub cter_id: u32,
    pub temp_id: u32,
    pub battle_data: Option<*mut BattleData>,
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

    pub fn get_battle_data_mut_ref(&self) -> Option<&mut BattleData> {
        unsafe {
            if self.battle_data.unwrap().is_null() {
                return None;
            }

            Some(self.battle_data.unwrap().as_mut().unwrap())
        }
    }

    pub fn new(battle_data: *mut BattleData, sender: Sender<RobotTask>) -> Self {
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
        let battle_data = self.get_battle_data_mut_ref();
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
        let mut v = vec![];
        for skill in battle_player.cter.skills.values() {
            //先判断技能释放条件
            let res = skill_condition(battle_data, skill, robot);
            //可以释放就往下走
            if !res {
                continue;
            }
            //获取技能释放目标
            let targets = skill_target(battle_data, skill, robot);
            if let Err(_) = targets {
                continue;
            }
            v.push(skill);
        }
        if v.is_empty() {
            return;
        }
        let mut rand = rand::thread_rng();
        let index = rand.gen_range(0..v.len());
        let skill = v.get(index).unwrap();
        let battle_data = self.get_battle_data_mut_ref().unwrap();
        let res = robot_use_skill(battle_data, skill, robot);
        if res {
            info!("机器人释放技能成功！skill_id:{}", skill.id);
        }
    }

    fn exit(&self) {
        // info!("robot:{} 退出使用技能状态！", self.robot_id);
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
