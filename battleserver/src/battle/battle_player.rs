use crate::battle::battle_enum::{
    AttackState, BattleCterState, BattlePlayerState, TURN_DEFAULT_MOVEMENT_POINTS,
};
use crate::battle::mission::MissionData;
use crate::battle::mission::MissionResetType;
use crate::battle::{battle::BattleData, mission::MissionCompleteType};
use crate::mgr::League;
use crate::robot::robot_action::RobotStatusAction;
use crate::robot::robot_task_mgr::RobotTask;
use crate::robot::RobotData;
use crate::room::member::Member;
use crate::TEMPLATES;
use crossbeam::channel::Sender;
use log::{error, info, warn};
use std::collections::HashMap;
use std::collections::HashSet;
use std::str::FromStr;
use tools::macros::GetMutRef;
use tools::protos::base::BattlePlayerPt;

use super::battle_cter::BattleCharacter;
use super::battle_enum::buff_type::SUB_MOVE_POINT;

///角色战斗基础属性
#[derive(Clone, Debug, Default)]
pub struct BattleStatus {
    pub is_pair: bool,                       //最近一次翻块是否匹配  玩家
    pub is_attacked: bool,                   //一轮有没有受到攻击伤害
    is_can_end_turn: bool,                   //是否可以结束turn
    pub locked_oper: u32,                    //锁住的操作，如果有值，玩家什么都做不了
    attack_state: AttackState,               //是否可以攻击
    pub attack_reward_movement_points: bool, //配对攻击后奖励翻地图块次数,表示是否奖励过翻拍次数
    pub battle_state: BattlePlayerState,     //玩家战斗状态
}

///角色战斗流程相关数据
#[derive(Clone, Debug, Default)]
pub struct TurnFlowData {
    pub residue_movement_points: u8,           //剩余移动点数
    pub open_map_cell_vec: Vec<usize>,         //最近一次turn翻过的地图块
    pub open_map_cell_vec_history: Vec<usize>, //这个turn所有翻过的地图块
    pub turn_limit_skills: Vec<u32>,           //turn限制技能
    pub round_limit_skills: Vec<u32>,          //round限制技能
    pub pair_usable_skills: HashSet<u32>,      //配对可用技能
}

///角色战斗流程相关数据
#[derive(Clone, Debug, Default)]
pub struct IndexData {
    pub map_cell_index: Option<usize>,      //角色所在位置
    pub last_map_cell_index: Option<usize>, //上一次所在地图块位置
}

///商品数据
#[derive(Clone, Debug, Default)]
pub struct MerchandiseData {
    buy_map: HashMap<u32, u8>,      //购买商品次数
    buy_history: HashMap<u32, u16>, //购买记录
}

impl MerchandiseData {
    pub fn get_turn_buy_times(&self, merchandise_id: u32) -> u8 {
        let times = self.buy_map.get(&merchandise_id);
        let res = match times {
            Some(times) => *times,
            None => 0,
        };
        res
    }

    pub fn add_buy_times(&mut self, merchandise_id: u32) {
        let times = self.buy_map.get(&merchandise_id);
        let mut res = match times {
            Some(times) => *times,
            None => 0,
        };
        res += 1;
        self.buy_map.insert(merchandise_id, res);

        let times = self.buy_history.get(&merchandise_id);
        let mut res = match times {
            Some(times) => *times,
            None => 0,
        };
        res += 1;
        self.buy_history.insert(merchandise_id, res);
    }

    pub fn clear_turn_buy_times(&mut self) {
        self.buy_map.clear();
    }
}

///战斗玩家数据
#[derive(Clone, Default)]
pub struct BattlePlayer {
    pub user_id: u32,                         //玩家ID
    pub name: String,                         //名称
    pub gold: i32,                            //金币
    pub grade: u8,                            //玩家grade
    pub league: League,                       //段位数据
    pub mission_data: MissionData,            //任务数据
    pub merchandise_data: MerchandiseData,    //商品数据
    pub cters: HashMap<u32, BattleCharacter>, //玩家的战斗角色
    pub major_cter: (u32, u32),               //主角色(动态id,配置id)
    pub current_cter: (u32, u32),             //当前角色(动态id,配置id)
    pub flow_data: TurnFlowData,              //战斗流程相关数据
    pub status: BattleStatus,                 //战斗状态
    pub robot_data: Option<RobotData>,        //机器人数据;如果有值，则是机器人，没有则是玩家
    pub team_id: u8,                          //队伍id
}

tools::get_mut_ref!(BattlePlayer);

impl BattlePlayer {
    ///初始化战斗角色数据
    pub fn init(
        member: &Member,
        battle_data: &mut BattleData,
        remember_size: u32,
        robot_sender: Sender<RobotTask>,
    ) -> anyhow::Result<Self> {
        let cter_id = battle_data.generate_cter_id();
        let mut battle_player = BattlePlayer::default();
        battle_player.user_id = member.user_id;
        battle_player.name = member.nick_name.clone();
        battle_player.league = member.league.clone();
        battle_player.grade = member.grade;
        let cter = BattleCharacter::init(member, cter_id)?;
        battle_player.major_cter = (cter.base_attr.cter_id, cter.base_attr.cter_temp_id);
        battle_player.current_cter = (cter.base_attr.cter_id, cter.base_attr.cter_temp_id);
        battle_player.cters.insert(cter.base_attr.cter_id, cter);
        //处理机器人部分
        if member.robot_temp_id > 0 {
            let robot_data = RobotData::new(
                battle_player.user_id,
                member.robot_temp_id,
                battle_data as *mut BattleData,
                remember_size,
                robot_sender,
            );
            battle_player.robot_data = Some(robot_data);
        }
        battle_player.reset_residue_movement_points();
        battle_player.team_id = member.team_id;
        Ok(battle_player)
    }

    pub fn convert_to_battle_player_pt(&self) -> BattlePlayerPt {
        let mut battle_player_pt = BattlePlayerPt::new();
        battle_player_pt.set_user_id(self.get_user_id());
        battle_player_pt.set_name(self.name.clone());
        battle_player_pt.set_league(self.league.into_pt());
        battle_player_pt.set_gold(self.gold as u32);
        battle_player_pt.set_grade(self.grade as u32);
        battle_player_pt.major_cter = self.major_cter.0;
        battle_player_pt.current_cter = self.current_cter.0;
        battle_player_pt.set_mission(self.mission_data.into_mission_pt());
        battle_player_pt.set_is_robot(self.is_robot());
        battle_player_pt.set_is_died(self.is_died());
        for cter in self.cters.values() {
            battle_player_pt
                .cters
                .push(cter.convert_to_battle_cter_pt());
        }
        battle_player_pt
    }

    pub fn clear_residue_movement_points(&mut self) {
        self.flow_data.residue_movement_points = 0;
    }

    pub fn get_major_cter_mut(&mut self) -> &mut BattleCharacter {
        self.cters.get_mut(&self.major_cter.0).unwrap()
    }

    pub fn get_major_cter(&self) -> &BattleCharacter {
        self.cters.get(&self.major_cter.0).unwrap()
    }

    pub fn get_current_cter_mut(&mut self) -> &mut BattleCharacter {
        self.cters.get_mut(&self.current_cter.0).unwrap()
    }

    pub fn get_current_cter(&self) -> &BattleCharacter {
        self.cters.get(&self.current_cter.0).unwrap()
    }

    pub fn player_die(&mut self, str: Option<String>) {
        self.get_major_cter_mut().base_attr.hp = 0;
        self.get_major_cter_mut().state = BattleCterState::Died;
        self.status.battle_state = BattlePlayerState::Died;
        if let Some(str) = str {
            info!("{:?}", str);
        }
    }

    pub fn add_open_map_cell(&mut self, index: usize) {
        self.flow_data.open_map_cell_vec_history.push(index);
    }

    pub fn clear_turn_open_map_cell(&mut self) {
        self.flow_data.open_map_cell_vec.clear();
        self.flow_data.open_map_cell_vec_history.clear();
    }

    pub fn change_robot_status(&self, robot_action: Box<dyn RobotStatusAction>) {
        let res = self.get_robot_action();
        if let Some(res) = res {
            res.exit();
        }
        self.set_robot_action(robot_action);
        let res = self.get_robot_action();
        if let Some(res) = res {
            res.enter();
        }
    }

    pub fn get_cter_temp_id(&self) -> u32 {
        self.get_current_cter().base_attr.cter_temp_id
    }

    ///是否行为被锁住了
    pub fn is_locked(&self) -> bool {
        self.status.locked_oper > 0
    }

    pub fn add_mission_progress(
        &mut self,
        value: u16,
        mission_type: MissionCompleteType,
        mission_parm: (u32, u32),
    ) -> bool {
        let res = self
            .mission_data
            .add_progress(value, mission_type, mission_parm);
        if res.0 {
            self.add_gold(res.1 as i32);
            return true;
        }
        false
    }

    pub fn add_gold(&mut self, value: i32) -> i32 {
        self.gold += value;
        if self.gold < 0 {
            self.gold = 0;
        }
        self.gold
    }

    pub fn get_robot_data_ref(&self) -> anyhow::Result<&RobotData> {
        if self.robot_data.is_none() {
            anyhow::bail!(
                "this battle_cter is not robot!user_id:{},cter_id:{}",
                self.user_id,
                self.get_cter_temp_id()
            )
        }
        Ok(self.robot_data.as_ref().unwrap())
    }

    pub fn robot_start_action(&mut self, battle_data: *mut BattleData) {
        if self.robot_data.is_none() {
            return;
        }
        let res = self.robot_data.as_mut().unwrap();
        res.battle_data = battle_data;
        //开始仲裁
        res.thinking_do_something();
    }

    pub fn get_robot_action(&self) -> Option<&mut Box<dyn RobotStatusAction>> {
        let self_mut_ref = self.get_mut_ref();
        self_mut_ref
            .robot_data
            .as_mut()
            .unwrap()
            .robot_status
            .as_mut()
    }

    pub fn set_robot_action(&self, action: Box<dyn RobotStatusAction>) {
        let self_mut_ref = self.get_mut_ref();
        self_mut_ref.robot_data.as_mut().unwrap().robot_status = Some(action);
    }

    ///奖励移动点数
    pub fn attack_reward_movement_points(&mut self) {
        if self.status.attack_reward_movement_points {
            return;
        }
        self.status.attack_reward_movement_points = true;
        self.reset_residue_movement_points();
    }

    pub fn get_current_cter_index(&self) -> usize {
        self.get_current_cter().get_map_cell_index()
    }

    pub fn is_died(&self) -> bool {
        self.status.battle_state != BattlePlayerState::Normal
    }

    pub fn get_user_id(&self) -> u32 {
        self.user_id
    }

    ///重制翻块次数
    pub fn reset_residue_movement_points(&mut self) {
        let temp = TEMPLATES.constant_temp_mgr();
        let res = temp.temps.get("turn_default_movement_points");
        let mut reward_count;
        match res {
            Some(res) => {
                let res = u8::from_str(res.value.as_str());
                match res {
                    Ok(res) => {
                        reward_count = res;
                    }
                    Err(e) => {
                        error!("{:?}", e);
                        reward_count = TURN_DEFAULT_MOVEMENT_POINTS;
                    }
                }
            }
            None => {
                warn!("ConstantTemp could not find!the key is 'turn_default_movement_points'");
                reward_count = TURN_DEFAULT_MOVEMENT_POINTS;
            }
        }
        //判断是否有扣行动点数上限的buff
        for cter in self.cters.values() {
            let res = cter
                .battle_buffs
                .buffs()
                .values()
                .find(|buff| buff.function_id == SUB_MOVE_POINT);
            if let Some(buff) = res {
                reward_count = reward_count - buff.buff_temp.par1 as u8;
                break;
            }
        }
        self.flow_data.residue_movement_points = reward_count;
    }

    pub fn set_is_can_end_turn(&mut self, value: bool) {
        self.status.is_can_end_turn = value;
    }

    pub fn clear_turn_buy_times(&mut self) {
        self.merchandise_data.clear_turn_buy_times();
    }

    pub fn reset_mission(&mut self, reset_type: MissionResetType) {
        self.mission_data.reset(reset_type);
    }

    pub fn get_is_can_end_turn(&self) -> bool {
        self.status.is_can_end_turn
    }

    pub fn is_can_attack(&self) -> bool {
        self.status.attack_state == AttackState::Able
    }

    pub fn change_attack_able(&mut self) {
        self.status.attack_state = AttackState::Able;
    }

    pub fn change_attack_locked(&mut self) {
        self.status.attack_state = AttackState::Locked;
    }

    pub fn change_attack_none(&mut self) {
        self.status.attack_state = AttackState::None;
    }

    pub fn get_attack_state(&self) -> AttackState {
        self.status.attack_state
    }

    pub fn is_robot(&self) -> bool {
        self.robot_data.is_some()
    }

    ///重制角色数据
    pub fn round_reset(&mut self) {
        self.status.is_attacked = false;
        self.change_attack_none();
        self.status.attack_reward_movement_points = false;
        self.get_current_cter_mut().index_data.map_cell_index = None;
        self.clear_turn_open_map_cell();
        self.get_current_cter_mut().index_data.last_map_cell_index = None;
        self.flow_data.round_limit_skills.clear();
        self.robot_reset();
        self.minon_clear();
    }

    pub fn robot_reset(&mut self) {
        if self.is_robot() {
            self.robot_data.as_mut().unwrap().remember_map_cell.clear();
        }
    }

    pub fn minon_clear(&mut self) {
        let mut rm_v = vec![];

        for cter in self.cters.values_mut() {
            for &minon in cter.minons.iter() {
                rm_v.push(minon);
            }
            cter.minons.clear();
        }

        for cter_id in rm_v {
            self.cters.remove(&cter_id);
        }
        self.current_cter = self.major_cter;
    }

    ///回合结算
    pub fn turn_start_reset(&mut self) {
        //重制剩余移动点数
        self.reset_residue_movement_points();
        //回合开始触发buff
        for cter in self.cters.values_mut() {
            cter.trigger_turn_start();
        }
    }

    pub fn turn_end_reset(&mut self) {
        //重制配对攻击奖励翻地图块次数
        self.status.attack_reward_movement_points = false;
        //重制是否翻过地图块
        self.clear_turn_open_map_cell();
        //清空turn限制
        self.flow_data.turn_limit_skills.clear();
        //重制是否可以攻击
        self.change_attack_none();
        //重制匹配状态
        self.status.is_pair = false;
        //重制商品购买次数
        self.clear_turn_buy_times();
        //重制任务
        self.reset_mission(MissionResetType::Trun);
        //重制可结束turn状态
        self.set_is_can_end_turn(false);
    }
}

pub enum TransformInheritValue {
    None,
    Int(usize),
}
impl TransformInheritValue {
    pub fn as_usize(&self) -> Option<usize> {
        match self {
            TransformInheritValue::Int(ref n) => Some(*n),
            _ => None,
        }
    }
}
