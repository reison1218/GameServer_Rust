use std::convert::TryFrom;

use log::error;
use protobuf::Message;
use rand::Rng;
use tools::cmd_code::ClientCode;
use tools::protos::battle::S_MISSION_NOTICE;
use tools::templates::mission_temp::MissionTemp;

use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

use super::battle::BattleData;

use tools::protos::base::MissionPt;

///任务通知类型
#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum MissionNoticeType {
    New = 1,
    Complete = 2,
}

impl MissionNoticeType {
    pub fn into_u32(self) -> u32 {
        let res: u32 = self.into();
        res
    }
}

///任务重制类型
#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum MissionResetType {
    Trun = 1,
    Round = 2,
}

///任务完成条件枚举
#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum MissionCompleteType {
    ///翻地图块次数
    OpenCellTimes = 1,
    ///配对次数
    PairTimes = 2,
    ///翻开指定元素地图块次数
    OpenCellElement = 3,
    ///使用技能次数
    UseSkillTimes = 4,
    ///攻击上一个turn攻击过你的玩家      
    AttackLastTurnUser = 5,
    ///一个turn内配对次数
    TurnPairTimes = 6,
    ///收集金币数量  
    GoldCount = 7,
    ///配对指定元素地图块次数   
    PairCellElement = 8,
}

///任务结构体
#[derive(Clone, Default)]
pub struct MissionData {
    pub mission: Option<Mission>, //当前任务
    pub complete_list: Vec<u32>,  //完成的任务列表
    pub history_list: Vec<u32>,   //接过的任务列表
}

#[derive(Clone)]
pub struct Mission {
    pub progress: u16,                      //任务进度
    pub is_complete: bool,                  //是否完成
    pub mission_temp: &'static MissionTemp, //任务id
}

impl MissionData {
    pub fn into_mission_pt(&self) -> MissionPt {
        let mut pt = MissionPt::new();
        let mission = self.mission.as_ref();
        if let Some(mission) = mission {
            pt.set_complete(mission.is_complete);
            pt.set_mission_id(mission.mission_temp.id);
            pt.set_progress(mission.progress as u32);
        }
        pt
    }

    pub fn get_last_mission(&self) -> u32 {
        let mission = self.mission.as_ref();
        match mission {
            Some(mission) => mission.mission_temp.id,
            None => 0,
        }
    }

    ///新任务
    pub fn new_mission(&mut self, mission_temp: &'static MissionTemp) {
        let mission = Mission {
            progress: 0,
            is_complete: false,
            mission_temp,
        };
        self.mission = Some(mission);
        self.history_list.push(mission_temp.id);
    }

    ///是否完成
    pub fn is_complete(&self) -> bool {
        let mission = self.mission.as_ref();
        match mission {
            Some(mission) => mission.is_complete,
            None => false,
        }
    }

    ///重制任务
    pub fn reset(&mut self, reset_type: MissionResetType) {
        if self.mission.is_none() {
            return;
        }
        let mission = self.mission.as_mut().unwrap();
        let complete_type =
            MissionCompleteType::try_from(mission.mission_temp.complete_condition).unwrap();
        match reset_type {
            MissionResetType::Trun => match complete_type {
                MissionCompleteType::TurnPairTimes => {
                    mission.progress = 0;
                }
                _ => {}
            },
            MissionResetType::Round => {}
        }
    }

    ///任务加进度，并会判断任务是否完成
    ///
    ///value:需要加的进度值
    ///
    ///misson_parm:元组数据，任务的参数
    pub fn add_progress(
        &mut self,
        value: u16,
        mission_type: MissionCompleteType,
        misson_parm: (u32, u32),
    ) -> (bool, u16) {
        let mut res = false;
        //如果任务已经完成，或者任务是空的，直接返回
        if self.is_complete() || self.mission.is_none() {
            return (res, 0);
        }
        let mission = self.mission.as_mut().unwrap();
        let mission_id = mission.mission_temp.id;
        let complete_condition = mission.mission_temp.complete_condition;
        let complete_par1 = mission.mission_temp.complete_par1;
        let _ = mission.mission_temp.complete_par2;
        let _ = mission.mission_temp.complete_par3;
        let complete_value = mission.mission_temp.complete_value;
        let complete_reward = mission.mission_temp.complete_reward;

        let temp_type = MissionCompleteType::try_from(complete_condition).unwrap();

        //如果类型不匹配,则返回
        if temp_type != mission_type {
            return (res, 0);
        }

        //校验需要带参数的任务
        match mission_type {
            MissionCompleteType::OpenCellElement | MissionCompleteType::PairCellElement => {
                let element = misson_parm.0;
                if element == complete_par1 as u32 {
                    mission.progress += value;
                }
            }
            _ => {
                mission.progress += value;
            }
        }
        //判断任务是否完成
        if mission.progress >= complete_value {
            res = true;
            mission.is_complete = res;
            self.complete_list.push(mission_id);
            return (res, complete_reward);
        }
        (res, 0)
    }
}

///随机任务
pub fn random_mission(battle_data: &mut BattleData, user_id: u32) {
    let cter = battle_data.battle_player.get_mut(&user_id).unwrap();
    let mission_temp_mgr = crate::TEMPLATES.mission_temp_mgr();
    let mut random = rand::thread_rng();
    let no_condition_missions = mission_temp_mgr.no_condition_mission();
    let mut mission_list = vec![];
    let history_list = &cter.mission_data.history_list;
    let last_mission_id = cter.mission_data.get_last_mission();
    //先添加无需条件都任务
    for mission_temp in no_condition_missions.iter() {
        if history_list.contains(&mission_temp.id) {
            continue;
        }
        mission_list.push(mission_temp.id);
    }

    //todo 再添加需要条件的

    //如果任务都接过了,只过滤上一次的任务就行了
    if mission_list.is_empty() {
        let mut temp_id;
        for temp in no_condition_missions.iter() {
            temp_id = temp.id;
            if temp_id == last_mission_id {
                continue;
            }
            mission_list.push(temp_id);
        }
    }

    //随机一个出来
    let index = random.gen_range(0..mission_list.len());
    let &temp_id = mission_list.get(index).unwrap();
    let temp = mission_temp_mgr.get_temp(&temp_id).unwrap();
    cter.mission_data.new_mission(temp);
    let missoin_id = cter.mission_data.get_last_mission();
    //封装proto，通知客户端
    let mut proto = S_MISSION_NOTICE::new();
    proto.set_user_id(cter.get_user_id());
    proto.set_mission_id(missoin_id);
    proto.set_notice_type(MissionNoticeType::New.into_u32());
    let bytes = proto.write_to_bytes();
    match bytes {
        Ok(bytes) => battle_data.send_2_client(ClientCode::MissionNoice, user_id, bytes),
        Err(e) => {
            error!("{:?}", e);
        }
    }
}

///任务触发类型
pub enum MissionTriggerType {
    ///翻开地图块触发
    OpenCell,
    ///配对触发
    Pair,
    ///攻击触发
    Attack,
    ///使用技能触发
    UseSkill,
    ///获得金币触发
    GetGold,
}

///触发任务
pub fn trigger_mission(
    battle_data: &mut BattleData,
    user_id: u32,
    trigger_types: Vec<(MissionTriggerType, u16)>,
    mission_parm: (u32, u32),
) {
    let battle_player = battle_data.battle_player.get_mut(&user_id).unwrap();
    //如果任务是空的，或者任务完成了则直接返回
    if battle_player.mission_data.mission.is_none()
        || battle_player
            .mission_data
            .mission
            .as_ref()
            .unwrap()
            .is_complete
    {
        return;
    }
    let missoin_id = battle_player
        .mission_data
        .mission
        .as_ref()
        .unwrap()
        .mission_temp
        .id;

    //匹配任务
    for (trigger_type, value) in trigger_types {
        let res = match trigger_type {
            MissionTriggerType::OpenCell => {
                open_cell_trigger_mission(battle_data, user_id, value, mission_parm)
            }
            MissionTriggerType::Pair => {
                pair_cell_trigger_mission(battle_data, user_id, value, mission_parm)
            }
            MissionTriggerType::Attack => {
                attack_trigger_mission(battle_data, user_id, value, mission_parm)
            }
            MissionTriggerType::UseSkill => {
                skill_times_trigger_mission(battle_data, user_id, value, mission_parm)
            }
            MissionTriggerType::GetGold => {
                get_gold_trigger_mission(battle_data, user_id, value, mission_parm)
            }
        };
        //没有完成就continue
        if !res {
            continue;
        }

        //任务完成了，通知客户端
        let mut proto = S_MISSION_NOTICE::new();
        proto.set_user_id(user_id);
        proto.set_mission_id(missoin_id);
        proto.set_notice_type(MissionNoticeType::Complete.into_u32());
        let bytes = proto.write_to_bytes();
        match bytes {
            Ok(bytes) => battle_data.send_2_all_client(ClientCode::MissionNoice, bytes),
            Err(e) => {
                error!("{:?}", e);
            }
        }
        break;
    }
}

///翻地图块触发任务
fn open_cell_trigger_mission(
    battle_data: &mut BattleData,
    user_id: u32,
    value: u16,
    mission_parm: (u32, u32),
) -> bool {
    let battle_player = battle_data.get_battle_player_mut(Some(user_id), true);
    if let Err(e) = battle_player {
        error!("{:?}", e);
        return false;
    }
    let battle_player = battle_player.unwrap();

    //翻地图块次数;翻开指定元素的地图块
    let mission_type_list = vec![
        MissionCompleteType::OpenCellTimes,
        MissionCompleteType::OpenCellElement,
    ];
    let mut res = false;
    for &mission_type in mission_type_list.iter() {
        res = battle_player.add_mission_progress(value, mission_type, mission_parm);
        if res {
            break;
        }
    }
    res
}

///配对地图块触发任务
fn pair_cell_trigger_mission(
    battle_data: &mut BattleData,
    user_id: u32,
    value: u16,
    mission_parm: (u32, u32),
) -> bool {
    let cter = battle_data.get_battle_player_mut(Some(user_id), true);
    if let Err(e) = cter {
        error!("{:?}", e);
        return false;
    }
    let cter = cter.unwrap();
    //配对地图块次数;配对地图块次数;配对指定元素地图块
    let mission_type_list = vec![
        MissionCompleteType::PairTimes,
        MissionCompleteType::TurnPairTimes,
        MissionCompleteType::PairCellElement,
    ];

    let mut res = false;
    for &mission_type in mission_type_list.iter() {
        res = cter.add_mission_progress(value, mission_type, mission_parm);
        if res {
            break;
        }
    }
    res
}

///技能触发任务
fn skill_times_trigger_mission(
    battle_data: &mut BattleData,
    user_id: u32,
    value: u16,
    mission_parm: (u32, u32),
) -> bool {
    let cter = battle_data.get_battle_player_mut(Some(user_id), true);
    if let Err(e) = cter {
        error!("{:?}", e);
        return false;
    }
    let cter = cter.unwrap();
    //使用技能触发任务
    cter.add_mission_progress(value, MissionCompleteType::UseSkillTimes, mission_parm)
}

///攻击行为触发任务
fn attack_trigger_mission(
    battle_data: &mut BattleData,
    user_id: u32,
    value: u16,
    mission_parm: (u32, u32),
) -> bool {
    let battle_player = battle_data.get_battle_player_mut(Some(user_id), true);
    if let Err(e) = battle_player {
        error!("{:?}", e);
        return false;
    }
    let battle_player = battle_player.unwrap();
    if battle_player.cter.revenge_user_id == mission_parm.0 {
        //复仇
        return battle_player.add_mission_progress(
            value,
            MissionCompleteType::AttackLastTurnUser,
            mission_parm,
        );
    }
    false
}

///获得金币触发任务
fn get_gold_trigger_mission(
    battle_data: &mut BattleData,
    user_id: u32,
    value: u16,
    mission_parm: (u32, u32),
) -> bool {
    let cter = battle_data.get_battle_player_mut(Some(user_id), true);
    if let Err(e) = cter {
        error!("{:?}", e);
        return false;
    }
    let cter = cter.unwrap();
    //加任务进度
    cter.add_mission_progress(value, MissionCompleteType::GoldCount, mission_parm)
}
