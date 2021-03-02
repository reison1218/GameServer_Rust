use rand::Rng;
use tools::templates::mission_temp::MissionTemp;

use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

use crate::room::character::BattleCharacter;

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
#[derive(Clone)]
pub struct Mission {
    pub user_id: u32,                       //玩家id
    pub progress: u16,                      //任务进度
    pub is_complete: bool,                  //是否完成
    pub mission_temp: &'static MissionTemp, //任务id
}

impl Mission {
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
        if self.is_complete {
            return (res, 0);
        }
        if mission_type == MissionCompleteType::OpenCellElement
            || mission_type == MissionCompleteType::PairCellElement
            || mission_type == MissionCompleteType::AttackLastTurnUser
        {
            let element = misson_parm.0;
            if element == self.mission_temp.complete_par1 as u32 {
                self.progress += value;
            }
            if self.progress >= self.mission_temp.appear_par1 {
                res = true;
            }
        } else {
            self.progress += value;
            if self.progress >= self.mission_temp.appear_par1 {
                res = true;
            }
        }

        if res {
            self.is_complete = res;
            return (res, self.mission_temp.complete_reward);
        }
        (res, 0)
    }
}

///随机任务
pub fn random_mission(cter: &mut BattleCharacter) {
    let mission_temp_mgr = crate::TEMPLATES.mission_temp_mgr();
    let mut random = rand::thread_rng();
    let no_condition_missions = mission_temp_mgr.no_condition_mission();
    let index = random.gen_range(0, no_condition_missions.len());
    let temp = no_condition_missions.get(index).unwrap();

    let res = Mission {
        user_id: cter.get_user_id(),
        progress: 0,
        is_complete: false,
        mission_temp: temp,
    };
    cter.mission = Some(res);
}
