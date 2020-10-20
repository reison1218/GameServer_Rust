use crate::battle::enums::{AttackState, BattleCterState, RobotState};
use crate::battle::map_data::TileMap;
use crossbeam::atomic::AtomicCell;
use std::collections::HashMap;
use tools::templates::buff_temp::BuffTemp;
use tools::templates::skill_temp::SkillTemp;

#[derive(Clone, Debug)]
pub struct Skill {
    pub id: u32,
    pub skill_temp: &'static SkillTemp,
    pub cd_times: i8,    //剩余cd,如果是消耗能量则无视这个值
    pub is_active: bool, //是否激活
}

impl From<&'static SkillTemp> for Skill {
    fn from(skill_temp: &'static SkillTemp) -> Self {
        Skill {
            id: skill_temp.id,
            cd_times: 0,
            skill_temp: skill_temp,
            is_active: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Item {
    pub id: u32,                        //物品id
    pub skill_temp: &'static SkillTemp, //物品带的技能
}

///角色战斗buff
#[derive(Clone, Debug, Default)]
pub struct BattleBuff {
    pub buffs: HashMap<u32, Buff>,          //角色身上的buff
    pub passive_buffs: HashMap<u32, Buff>,  //被动技能id
    pub add_damage_buffs: HashMap<u32, u8>, //伤害加深buff key:buffid value:叠加次数
    pub sub_damage_buffs: HashMap<u32, u8>, //减伤buff  key:buffid value:叠加次数
}

#[derive(Clone, Debug)]
pub struct Buff {
    pub id: u32,
    pub buff_temp: &'static BuffTemp,
    pub trigger_timesed: i8,       //已经触发过的次数
    pub keep_times: i8,            //剩余持续轮数
    pub scope: Vec<Direction>,     //buff的作用范围
    pub permanent: bool,           //是否永久
    pub from_user: Option<u32>,    //来源的玩家id
    pub from_skill: Option<u32>,   //来源的技能id
    pub turn_index: Option<usize>, //生效于turn_index
}

#[derive(Debug, Clone)]
pub struct Direction {
    pub direction: &'static Vec<isize>,
}

///角色战斗基础属性
#[derive(Debug, Default)]
pub struct BaseAttr {
    pub room_id: AtomicCell<u64>,
    pub cter_id: AtomicCell<u32>,
    pub robot_id: AtomicCell<u32>,
    pub name: String, //机器人名字
    pub grade: u8,    //等级
    pub atk: u8,      //攻击力
    pub hp: i16,      //角色血量
    pub defence: u8,  //角色防御
    pub energy: u8,   //角色能量
    pub element: u8,  //角色元素
}

///角色战斗基础属性
#[derive(Clone, Debug, Default)]
pub struct BattleStatus {
    pub is_pair: bool,             //最近一次翻块是否匹配
    pub is_attacked: bool,         //一轮有没有受到攻击伤害
    is_can_end_turn: bool,         //是否可以结束turn
    pub locked_oper: u32,          //锁住的操作，如果有值，玩家什么都做不了
    pub state: BattleCterState,    //角色状态
    pub attack_state: AttackState, //是否可以攻击
}

///角色战斗流程相关数据
#[derive(Clone, Debug, Default)]
pub struct TurnFlowData {
    pub residue_open_times: u8,        //剩余翻地图块次数
    pub open_map_cell_vec: Vec<usize>, //最近一次turn翻过的地图块
    pub turn_limit_skills: Vec<u32>,   //turn限制技能
    pub round_limit_skills: Vec<u32>,  //round限制技能
}

///角色战斗流程相关数据
#[derive(Clone, Debug, Default)]
pub struct IndexData {
    map_cell_index: Option<usize>,          //角色所在位置
    pub last_map_cell_index: Option<usize>, //上一次所在地图块位置
}

///角色战斗数据
#[derive(Debug, Default)]
pub struct RobotCter {
    pub tail_map: TileMap,                           //地图数据
    pub base_attr: BaseAttr,                         //基础属性
    pub battle_status: BattleStatus,                 //战斗状态
    pub robot_status: RobotState,                    //机器人状态
    pub battle_buffs: BattleBuff,                    //战斗buff
    pub flow_data: TurnFlowData,                     //战斗流程相关数据
    pub index_data: IndexData,                       //角色位置数据
    pub skills: HashMap<u32, Skill>,                 //玩家选择的主动技能id
    pub items: HashMap<u32, Item>,                   //角色身上的道具
    pub self_transform_cter: Option<Box<RobotCter>>, //自己变身的角色
    pub self_cter: Option<Box<RobotCter>>,           //原本的角色
}
