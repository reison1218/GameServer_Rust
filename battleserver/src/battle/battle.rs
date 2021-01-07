use crate::battle::battle_enum::skill_type::{
    ADD_BUFF, AUTO_PAIR_MAP_CELL, CHANGE_MAP_CELL_INDEX, MOVE_USER, NEAR_SKILL_DAMAGE_AND_CURE,
    RED_SKILL_CD, SCOPE_CURE, SHOW_INDEX, SHOW_MAP_CELL, SKILL_AOE, SKILL_DAMAGE,
    SKILL_DAMAGE_OPENED_ELEMENT, SKILL_OPEN_MAP_CELL, TRANSFORM,
};

use crate::battle::battle_enum::LIMIT_TOTAL_TURN_TIMES;
use crate::battle::battle_skill::{
    add_buff, auto_pair_map_cell, change_map_cell_index, move_user, scope_cure, show_index,
    show_map_cell, single_skill_damage, skill_aoe_damage, skill_damage_and_cure,
    skill_damage_opened_element, skill_open_map_cell, sub_cd, transform,
};
use crate::room::character::BattleCharacter;
use crate::room::map_data::TileMap;
use crate::room::MEMBER_MAX;
use crate::task_timer::{Task, TaskCmd};
use crossbeam::channel::Sender;
use log::error;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use tools::protos::base::{ActionUnitPt, SummaryDataPt};
use tools::templates::skill_temp::SkillTemp;

///物品结构体
#[derive(Clone, Debug)]
pub struct Item {
    pub id: u32,                        //物品id
    pub skill_temp: &'static SkillTemp, //物品带的技能
}

///方向结构体,用于aoe技能范围计算
#[derive(Debug, Clone)]
pub struct Direction {
    pub direction: &'static Vec<isize>,
}

type SkillFn = HashMap<
    &'static [u32],
    unsafe fn(
        &mut BattleData,
        user_id: u32,
        skill_id: u32,
        target_array: Vec<u32>,
        au: &mut ActionUnitPt,
    ) -> Option<Vec<ActionUnitPt>>,
>;

#[derive(Clone, Default)]
pub struct SummaryUser {
    pub user_id: u32,         //玩家id
    pub cter_id: u32,         //角色id
    pub grade: u8,            //玩家等级
    pub rank: u8,             //玩家当局排名
    pub league_score: i32,    //段位总积分
    pub league_id: u8,        //段位
    pub reward_score: i32,    //当局奖励积分
    pub push_to_server: bool, //是否推送过给游戏服务器
}

impl From<&BattleCharacter> for SummaryUser {
    fn from(cter: &BattleCharacter) -> Self {
        let mut sp = SummaryUser::default();
        sp.user_id = cter.get_user_id();
        sp.cter_id = cter.get_cter_id();
        sp.grade = cter.base_attr.grade;
        sp.league_score = cter.league.score;
        sp
    }
}
impl Into<SummaryDataPt> for SummaryUser {
    fn into(self) -> SummaryDataPt {
        let mut smp = SummaryDataPt::new();
        smp.user_id = self.user_id;
        smp.cter_id = self.cter_id;
        smp.rank = self.rank as u32;
        smp.grade = self.grade as u32;
        smp.reward_score = self.reward_score;
        smp.league_score = self.league_score as u32;
        smp.league_id = self.league_id as u32;
        smp
    }
}

///房间战斗数据封装
#[derive(Clone)]
pub struct BattleData {
    pub tile_map: TileMap,                          //地图数据
    pub next_turn_index: usize,                     //下个turn的下标
    pub turn_orders: [u32; MEMBER_MAX as usize],    //turn行动队列，里面放玩家id
    pub reflash_map_turn: Option<usize>,            //刷新地图时的turn下标
    pub battle_cter: HashMap<u32, BattleCharacter>, //角色战斗数据
    pub summary_vec: Vec<Vec<SummaryUser>>,         //排名  user_id
    pub summary_vec_temp: Vec<SummaryUser>,         //同一批挂掉的人
    pub leave_user: (u32, bool),                    //离开玩家id,是否惩罚
    pub leave_map: HashMap<u32, u8>,                //段位快照
    pub turn_limit_time: u64,                       //战斗turn时间限制
    pub skill_cmd_map: SkillFn,                     //技能函数指针map
    pub total_turn_times: u16,                      //总的turn次数
    pub last_map_id: u32,                           //上次地图id
    pub task_sender: Sender<Task>,                  //任务sender
    pub tcp_sender: Sender<Vec<u8>>,                //sender
}

tools::get_mut_ref!(BattleData);

unsafe impl Send for BattleData {}
unsafe impl Sync for BattleData {}

impl BattleData {
    ///添加总turn的次数
    pub fn add_total_turn_times(&mut self) {
        self.total_turn_times += 1;
        if self.total_turn_times < LIMIT_TOTAL_TURN_TIMES {
            return;
        }
        let mut task = Task::default();
        task.cmd = TaskCmd::MaxBattleTurnTimes.into();
        let mut map = serde_json::Map::new();
        let mut user_id = 0;
        for cter in self.battle_cter.values() {
            user_id = cter.get_user_id();
            break;
        }
        map.insert("user_id".to_owned(), serde_json::Value::from(user_id));
        task.data = serde_json::Value::from(map);
        let res = self.task_sender.send(task);
        if let Err(e) = res {
            error!("{:?}", e);
        }
    }

    ///初始化战斗数据
    pub fn new(task_sender: Sender<Task>, tcp_sender: Sender<Vec<u8>>) -> Self {
        let mut v = Vec::new();
        for _ in 0..MEMBER_MAX {
            v.push(Vec::new());
        }
        let mut bd = BattleData {
            tile_map: TileMap::default(),
            next_turn_index: 0,
            turn_orders: [0; MEMBER_MAX as usize],
            reflash_map_turn: None,
            battle_cter: HashMap::new(),
            summary_vec: v,
            summary_vec_temp: Vec::new(),
            leave_user: (0, false),
            leave_map: HashMap::new(),
            turn_limit_time: 60000, //默认一分钟
            skill_cmd_map: HashMap::new(),
            total_turn_times: 0,
            last_map_id: 0,
            task_sender,
            tcp_sender,
        };

        //初始化函数指针，封装到map里
        bd.skill_cmd_map
            .insert(&AUTO_PAIR_MAP_CELL[..], auto_pair_map_cell);
        bd.skill_cmd_map.insert(&ADD_BUFF[..], add_buff);
        bd.skill_cmd_map
            .insert(&CHANGE_MAP_CELL_INDEX[..], change_map_cell_index);
        bd.skill_cmd_map.insert(&SHOW_MAP_CELL[..], show_map_cell);
        bd.skill_cmd_map.insert(&SHOW_INDEX[..], show_index);

        bd.skill_cmd_map.insert(&MOVE_USER[..], move_user);
        bd.skill_cmd_map
            .insert(&NEAR_SKILL_DAMAGE_AND_CURE[..], skill_damage_and_cure);
        bd.skill_cmd_map
            .insert(&SKILL_DAMAGE[..], single_skill_damage);
        bd.skill_cmd_map.insert(&SKILL_AOE[..], skill_aoe_damage);
        bd.skill_cmd_map.insert(&RED_SKILL_CD[..], sub_cd);
        bd.skill_cmd_map
            .insert(&SKILL_OPEN_MAP_CELL[..], skill_open_map_cell);
        bd.skill_cmd_map.insert(
            &SKILL_DAMAGE_OPENED_ELEMENT[..],
            skill_damage_opened_element,
        );
        bd.skill_cmd_map.insert(&SCOPE_CURE[..], scope_cure);
        bd.skill_cmd_map.insert(&TRANSFORM[..], transform);
        bd
    }

    ///得到当前turn玩家的id
    /// 当找不到的时候就返回错误信息
    /// 找的到到时候范围Ok(user_id)
    pub fn get_turn_user(&self, _index: Option<usize>) -> anyhow::Result<u32> {
        let index;
        if let Some(_index) = _index {
            index = _index;
        } else {
            index = self.next_turn_index;
        }
        let res = self.turn_orders.get(index);
        if res.is_none() {
            anyhow::bail!("get_next_turn_user is none for index:{} ", index)
        }
        let user_id = *res.unwrap();
        Ok(user_id)
    }

    pub fn get_sender_mut(&mut self) -> &mut Sender<Vec<u8>> {
        self.tcp_sender.borrow_mut()
    }

    ///获得战斗角色借用指针
    pub fn get_battle_cter(
        &self,
        user_id: Option<u32>,
        is_alive: bool,
    ) -> anyhow::Result<&BattleCharacter> {
        let _user_id;
        if let Some(id) = user_id {
            _user_id = id;
        } else {
            let res = self.get_turn_user(None);
            if let Err(e) = res {
                anyhow::bail!("{:?}", e)
            }
            _user_id = res.unwrap();
        }
        let cter = self.battle_cter.get(&_user_id);
        if let None = cter {
            anyhow::bail!("there is no battle_cter!user_id:{}", _user_id)
        }
        let cter = cter.unwrap();
        if is_alive && cter.is_died() {
            anyhow::bail!(
                "this battle_cter is already died!user_id:{},cter_id:{}",
                _user_id,
                cter.get_cter_id()
            )
        }
        Ok(cter)
    }

    pub fn get_battle_cter_by_map_cell_index(
        &self,
        index: usize,
    ) -> anyhow::Result<&BattleCharacter> {
        let res = self.tile_map.map_cells.get(index);
        if res.is_none() {
            anyhow::bail!("there is no map_cell!index:{}", index)
        }
        let map_cell = res.unwrap();
        let user_id = map_cell.user_id;
        if user_id <= 0 {
            anyhow::bail!("this map_cell's user_id is 0!map_cell_index:{}", index)
        }
        let cter = self.battle_cter.get(&user_id);
        if cter.is_none() {
            anyhow::bail!("cter not find!user_id:{}", user_id)
        }
        let cter = cter.unwrap();
        if cter.is_died() {
            anyhow::bail!(
                "this battle_cter is already died!user_id:{},cter_id:{}",
                user_id,
                cter.get_cter_id()
            )
        }
        Ok(cter)
    }

    ///根据地图下标获得上面的战斗角色
    ///如果找不到该下标的地图块或者该地图块上面的玩家id为0，则返回错误信息
    pub fn get_battle_cter_mut_by_map_cell_index(
        &mut self,
        index: usize,
    ) -> anyhow::Result<&mut BattleCharacter> {
        let res = self.tile_map.map_cells.get(index);
        if res.is_none() {
            anyhow::bail!("there is no map_cell!index:{}", index)
        }
        let map_cell = res.unwrap();
        let user_id = map_cell.user_id;
        if user_id <= 0 {
            anyhow::bail!("this map_cell's user_id is 0!map_cell_index:{}", index)
        }
        let cter = self.battle_cter.get_mut(&user_id);
        if cter.is_none() {
            anyhow::bail!("cter not find!user_id:{}", user_id)
        }
        let cter = cter.unwrap();
        if cter.is_died() {
            anyhow::bail!(
                "this battle_cter is already died!user_id:{},cter_id:{}",
                user_id,
                cter.get_cter_id()
            )
        }
        Ok(cter)
    }

    pub fn get_battle_cters_vec(&self) -> Vec<u32> {
        let mut v = Vec::new();
        for id in self.battle_cter.keys() {
            v.push(*id);
        }
        v
    }
}
