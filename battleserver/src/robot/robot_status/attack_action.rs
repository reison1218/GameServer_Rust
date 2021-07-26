use super::*;
use crate::{
    battle::battle_enum::{TargetType, TRIGGER_SCOPE_CENTER_NEAR_TEMP_ID},
    robot::{robot_skill::get_hp_max_cter, RobotActionType},
};
use log::warn;
use tools::cmd_code::BattleCode;

#[derive(Default)]
pub struct AttackRobotAction {
    pub robot_id: u32,
    pub cter_id: u32,
    pub temp_id: u32,
    pub battle_data: Option<*mut BattleData>,
    pub status: RobotStatus,
    pub sender: Option<Sender<RobotTask>>,
}

get_mut_ref!(AttackRobotAction);

impl AttackRobotAction {
    pub fn get_battle_data_mut_ref(&self) -> Option<&mut BattleData> {
        unsafe {
            if self.battle_data.unwrap().is_null() {
                return None;
            }

            Some(self.battle_data.unwrap().as_mut().unwrap())
        }
    }

    pub fn new(battle_data: *mut BattleData, sender: Sender<RobotTask>) -> Self {
        let mut attack_action = AttackRobotAction::default();
        attack_action.battle_data = Some(battle_data);
        attack_action.sender = Some(sender);
        attack_action
    }
}

impl RobotStatusAction for AttackRobotAction {
    fn set_sender(&mut self, sender: Sender<RobotTask>) {
        self.sender = Some(sender);
    }

    fn get_cter_temp_id(&self) -> u32 {
        self.cter_id
    }

    fn enter(&self) {
        info!("robot:{} 进入攻击状态！", self.robot_id);
        self.execute();
    }

    fn execute(&self) {
        let res = self.get_battle_data_mut_ref();
        if res.is_none() {
            warn!("the *const BattleData is null!");
            return;
        }
        let battle_data = res.unwrap();
        let target_index: usize;
        let robot_id = self.robot_id;
        let robot = battle_data.battle_player.get(&robot_id).unwrap();
        let skill_scope_temp = crate::TEMPLATES
            .skill_scope_temp_mgr()
            .get_temp(&TRIGGER_SCOPE_CENTER_NEAR_TEMP_ID)
            .unwrap();
        //如果有这个buff，就找人最多的
        if robot.get_current_cter().is_has_add_attack_and_aoe() {
            let mut player_index;
            let mut user_id;
            let mut player_count = (0, 0);
            for player in battle_data.battle_player.values() {
                player_index = player.get_current_cter_index();
                user_id = player.get_user_id();
                if user_id == robot_id {
                    continue;
                }
                let (_, count_v) = battle_data.cal_scope(
                    robot_id,
                    player_index as isize,
                    TargetType::SelfScopeOthers,
                    None,
                    Some(skill_scope_temp),
                );
                if count_v.len() > player_count.1 {
                    player_count.0 = player_index;
                    player_count.1 = count_v.len();
                }
            }

            if player_count.1 > 1 {
                target_index = player_count.0;
            } else {
                //如果没有就找血最多的
                let res = get_hp_max_cter(battle_data, robot_id);
                if let None = res {
                    warn!("attack counld not find target!robot_id:{}", robot_id);
                    return;
                }
                target_index = res.unwrap();
            }
        } else {
            //如果没有就找血最多的
            let res = get_hp_max_cter(battle_data, robot_id);
            if let None = res {
                warn!("attack counld not find target!robot_id:{}", robot_id);
                return;
            }
            target_index = res.unwrap();
        }
        self.send_2_battle(target_index, RobotActionType::Attack, BattleCode::Action);
    }

    fn exit(&self) {
        // info!("robot:{} 退出攻击状态！", self.robot_id);
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
