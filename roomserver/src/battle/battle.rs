use crate::battle::battle_enum::skill_type::{
    ADD_BUFF, AUTO_PAIR_CELL, CHANGE_INDEX, MOVE_USER, NEAR_SKILL_DAMAGE_AND_CURE, RED_SKILL_CD,
    SHOW_INDEX, SKILL_AOE, SKILL_DAMAGE,
};

use crate::battle::battle_skill::{
    add_buff, auto_pair_cell, change_index, move_user, show_index, single_skill_damage,
    skill_aoe_damage, skill_damage_and_cure, sub_cd,
};
use crate::room::character::BattleCharacter;
use crate::room::map_data::TileMap;
use crate::task_timer::Task;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use tools::protos::base::ActionUnitPt;
use tools::tcp::TcpSender;
use tools::templates::skill_temp::SkillTemp;

#[derive(Clone, Debug)]
pub struct Item {
    pub id: u32,                        //物品id
    pub skill_temp: &'static SkillTemp, //物品带的技能
}

#[derive(Debug, Clone)]
pub struct Direction {
    pub direction: &'static Vec<isize>,
}

///房间战斗数据封装
#[derive(Clone)]
pub struct BattleData {
    pub tile_map: TileMap,                          //地图数据
    pub choice_orders: [u32; 4],                    //选择顺序里面放玩家id
    pub next_choice_index: usize,                   //下一个选择的下标
    pub next_turn_index: usize,                     //下个turn的下标
    pub turn_orders: [u32; 4],                      //turn行动队列，里面放玩家id
    pub reflash_map_turn: Option<usize>,            //刷新地图时的turn下标
    pub battle_cter: HashMap<u32, BattleCharacter>, //角色战斗数据
    pub rank_vec: Vec<Vec<u32>>,                    //排名  user_id
    pub turn_limit_time: u64,                       //战斗turn时间限制
    pub skill_cmd_map: HashMap<
        &'static [u32],
        unsafe fn(
            &mut BattleData,
            user_id: u32,
            skill_id: u32,
            target_array: Vec<u32>,
            au: &mut ActionUnitPt,
        ) -> Option<Vec<ActionUnitPt>>,
    >, //技能函数指针map
    pub task_sender: crossbeam::Sender<Task>,       //任务sender
    pub sender: TcpSender,                          //sender
}

impl BattleData {
    ///初始化战斗数据
    pub fn new(task_sender: crossbeam::Sender<Task>, sender: TcpSender) -> Self {
        let mut bd = BattleData {
            tile_map: TileMap::default(),
            choice_orders: [0; 4],
            next_choice_index: 0,
            next_turn_index: 0,
            turn_orders: [0; 4],
            reflash_map_turn: None,
            battle_cter: HashMap::new(),
            rank_vec: Vec::new(),
            turn_limit_time: 60000, //默认一分钟
            skill_cmd_map: HashMap::new(),
            task_sender,
            sender,
        };

        //初始化函数指针，封装到map里
        bd.skill_cmd_map.insert(&AUTO_PAIR_CELL[..], auto_pair_cell);
        bd.skill_cmd_map.insert(&ADD_BUFF[..], add_buff);
        bd.skill_cmd_map.insert(&CHANGE_INDEX[..], change_index);
        bd.skill_cmd_map.insert(&SHOW_INDEX[..], show_index);
        bd.skill_cmd_map.insert(&MOVE_USER[..], move_user);
        bd.skill_cmd_map
            .insert(&NEAR_SKILL_DAMAGE_AND_CURE[..], skill_damage_and_cure);
        bd.skill_cmd_map
            .insert(&SKILL_DAMAGE[..], single_skill_damage);
        bd.skill_cmd_map.insert(&SKILL_AOE[..], skill_aoe_damage);
        bd.skill_cmd_map.insert(&RED_SKILL_CD[..], sub_cd);

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
            let str = format!("get_next_turn_user is none for index:{} ", index);
            anyhow::bail!(str)
        }
        let user_id = *res.unwrap();
        Ok(user_id)
    }

    ///获得当前turn玩家的下标
    pub fn get_turn_index(&self, user_id: u32) -> isize {
        let mut index = 0_isize;
        for member_id in self.turn_orders.iter() {
            if member_id == &user_id {
                return index;
            }
            index += 1;
        }
        return -1;
    }

    pub fn get_sender_mut(&mut self) -> &mut TcpSender {
        self.sender.borrow_mut()
    }

    ///获得战斗角色借用指针
    pub fn get_battle_cter(&self, user_id: Option<u32>) -> anyhow::Result<&BattleCharacter> {
        let _user_id;
        if let Some(user_id) = user_id {
            _user_id = user_id;
        } else {
            let res = self.get_turn_user(None);
            if let Err(e) = res {
                anyhow::bail!("{:?}", e)
            }
            _user_id = res.unwrap();
        }
        let cter = self.battle_cter.get(&_user_id);
        if let None = cter {
            let str = format!("there is no battle_cter!user_id:{}", _user_id);
            anyhow::bail!("{:?}", str)
        }
        Ok(cter.unwrap())
    }

    pub fn get_battle_cter_by_cell_index(&self, index: usize) -> anyhow::Result<&BattleCharacter> {
        let res = self.tile_map.map.get(index);
        if res.is_none() {
            anyhow::bail!("there is no cell!index:{}", index)
        }
        let cell = res.unwrap();
        let user_id = cell.user_id;
        if user_id <= 0 {
            anyhow::bail!("this cell's user_id is 0!cell_index:{}", index)
        }
        let cter = self.battle_cter.get(&user_id);
        if cter.is_none() {
            anyhow::bail!("cter not find!user_id:{}", user_id)
        }
        Ok(cter.unwrap())
    }

    ///根据地图下标获得上面的战斗角色
    ///如果找不到该下标的地图块或者该地图块上面的玩家id为0，则返回错误信息
    pub fn get_battle_cter_mut_by_cell_index(
        &mut self,
        index: usize,
    ) -> anyhow::Result<&mut BattleCharacter> {
        let res = self.tile_map.map.get(index);
        if res.is_none() {
            anyhow::bail!("there is no cell!index:{}", index)
        }
        let cell = res.unwrap();
        let user_id = cell.user_id;
        if user_id <= 0 {
            anyhow::bail!("this cell's user_id is 0!cell_index:{}", index)
        }
        let cter = self.battle_cter.get_mut(&user_id);
        if cter.is_none() {
            anyhow::bail!("cter not find!user_id:{}", user_id)
        }
        Ok(cter.unwrap())
    }

    pub fn get_battle_cters_vec(&self) -> Vec<u32> {
        let mut v = Vec::new();
        for id in self.battle_cter.keys() {
            v.push(*id);
        }
        v
    }
}
