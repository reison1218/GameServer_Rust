use crate::battle::battle::BattleData;
use crate::battle::battle_enum::{ActionType, TargetType, TRIGGER_SCOPE_NEAR};
use crate::room::character::BattleCharacter;
use crate::room::map_data::{Cell, CellType};
use crate::task_timer::{Task, TaskCmd};
use log::{error, warn};
use protobuf::{Message, RepeatedField};
use std::borrow::BorrowMut;
use tools::cmd_code::ClientCode;
use tools::protos::base::ActionUnitPt;
use tools::protos::battle::S_ACTION_NOTICE;
use tools::tcp::TcpSender;
use tools::templates::skill_scope_temp::SkillScopeTemp;
use tools::util::packet::Packet;

impl BattleData {
    ///校验是否翻过块
    pub fn check_user_is_opened_cell(&self) -> bool {
        if self.turn_action.actions.is_empty() {
            return false;
        }
        for action in self.turn_action.actions.iter() {
            let action_type = ActionType::from(action.action_type as u32);
            if action_type.eq(&ActionType::Open) {
                continue;
            }
            return true;
        }
        return false;
    }
    ///获得战斗角色可变借用指针
    pub fn get_battle_cter_mut(
        &mut self,
        user_id: Option<u32>,
    ) -> anyhow::Result<&mut BattleCharacter> {
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
        let cter = self.battle_cter.get_mut(&_user_id);
        if let None = cter {
            let str = format!("there is no battle_cter!user_id:{}", _user_id);
            anyhow::bail!("{:?}", str)
        }
        Ok(cter.unwrap())
    }

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

    ///获得玩家回合下标
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

    ///计算攻击力
    pub fn calc_damage(&self, user_id: u32) -> i32 {
        let battle_cter = self.battle_cter.get(&user_id).unwrap();
        let mut damage = battle_cter.atk;
        for buff in battle_cter.buff_array.iter() {
            //加攻击力的buff
            if buff.id != 4 {
                continue;
            }
            damage += buff.buff_temp.par1;
        }
        damage as i32
    }

    ///计算减伤
    pub fn calc_reduce_damage(&self, user_id: u32) -> i32 {
        let battle_cter = self.battle_cter.get(&user_id).unwrap();
        let value = battle_cter.defence;
        //todo 此处应该加上角色身上的减伤buff
        value as i32
    }

    pub fn send_2_client(&mut self, cmd: ClientCode, user_id: u32, bytes: Vec<u8>) {
        let bytes = Packet::build_packet_bytes(cmd as u32, user_id, bytes, true, true);
        self.get_sender_mut().write(bytes);
    }

    ///推送action通知
    pub fn push_action_notice(&mut self, aus: Vec<ActionUnitPt>) {
        let mut san = S_ACTION_NOTICE::new();
        san.set_action_uints(RepeatedField::from(aus));
        let bytes = san.write_to_bytes().unwrap();
        for member_id in self.battle_cter.clone().keys() {
            self.send_2_client(ClientCode::ActionNotice, *member_id, bytes.clone());
        }
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

    pub fn check_open_cell(&self, cell: &Cell) -> anyhow::Result<()> {
        if cell.id < CellType::Valid as u32 {
            let str = format!(
                "auto_pair_cell, this is cell can not be choice!index:{}",
                cell.index
            );
            anyhow::bail!(str)
        }
        if cell.is_world {
            let str = format!(
                "auto_pair_cell, world_cell can not be choice!index:{}",
                cell.index
            );
            anyhow::bail!(str)
        }
        if cell.pair_index.is_some() {
            let str = format!(
                "auto_pair_cell, this cell already pair!index:{}",
                cell.index
            );
            anyhow::bail!(str)
        }
        if cell.check_is_locked() {
            let str = format!("auto_pair_cell, this cell is locked!index:{}", cell.index);
            anyhow::bail!(str)
        }
        Ok(())
    }

    ///检查目标数组
    pub fn check_target_array(
        &self,
        user_id: u32,
        target_type: TargetType,
        target_array: &Vec<u32>,
    ) -> bool {
        match target_type {
            TargetType::None => return false, //无效目标
            TargetType::Cell => {
                //校验地图块下标有效性

                for index in target_array {
                    let index = *index;
                    let res = self.tile_map.map.get(index as usize);
                    if res.is_none() {
                        return false;
                    }
                    let cell = res.unwrap();
                    if cell.id <= CellType::Valid as u32 {
                        return false;
                    }
                }
                return true;
            } //任意玩家
            TargetType::AnyPlayer => {
                //校验玩家id
                let target_id = target_array.get(0);
                if target_id.is_none() {
                    return false;
                }
                let target_id = target_id.unwrap();
                if !self.battle_cter.contains_key(target_id) {
                    return false;
                }
                return true;
            } //玩家自己
            TargetType::PlayerSelf => {}      //玩家自己
            //全图
            TargetType::AllPlayer => {
                for member_id in target_array {
                    if !self.battle_cter.contains_key(&member_id) {
                        return false;
                    }
                }
                return true;
            } //不包括自己的其他玩家
            TargetType::OtherAllPlayer => {
                for member_id in target_array {
                    if member_id != &user_id && !self.battle_cter.contains_key(&user_id) {
                        return false;
                    }
                }
                return true;
            } //除自己外任意玩家
            TargetType::OtherAnyPlayer => {
                let member_id = target_array.get(0);
                if member_id.is_none() {
                    return false;
                }
                let member_id = member_id.unwrap();
                if member_id == &user_id {
                    return false;
                }
                if !self.battle_cter.contains_key(&member_id) {
                    return false;
                }
                return true;
            } //未翻开的地图块
            TargetType::UnOpenCell => {
                for index in target_array {
                    let index = *index;
                    let cell = self.tile_map.map.get(index as usize);
                    if cell.is_none() {
                        return false;
                    }
                    let cell = cell.unwrap();
                    if cell.id <= CellType::Valid as u32 {
                        return false;
                    }
                    if cell.pair_index.is_some() {
                        return false;
                    }
                }
                return true;
            } //未配对的地图块
            TargetType::UnPairCell => {
                for index in target_array {
                    let index = *index;
                    let cell = self.tile_map.map.get(index as usize);
                    if cell.is_none() {
                        return false;
                    }
                    let cell = cell.unwrap();
                    if cell.id <= CellType::Valid as u32 {
                        return false;
                    }
                    if cell.pair_index.is_some() {
                        return false;
                    }
                }
                return true;
            } //空的地图块
            TargetType::NullCell => {
                for index in target_array {
                    let index = *index;
                    let cell = self.tile_map.map.get(index as usize);
                    if cell.is_none() {
                        return false;
                    }
                    let cell = cell.unwrap();
                    if cell.id <= CellType::Valid as u32 {
                        return false;
                    }
                    if cell.user_id != 0 {
                        return false;
                    }
                }
                return true;
            } //空的地图块，上面没人
            TargetType::UnPairNullCell => {
                for index in target_array {
                    let index = *index;
                    let cell = self.tile_map.map.get(index as usize);
                    if cell.is_none() {
                        return false;
                    }
                    let cell = cell.unwrap();
                    if cell.id <= CellType::Valid as u32 {
                        return false;
                    }
                    if cell.user_id != 0 {
                        return false;
                    }
                    if cell.pair_index.is_some() {
                        return false;
                    }
                }
                return true;
            } //地图块上的玩家
            TargetType::CellPlayer => {}
        }
        true
    }

    //检测地图块是否选择
    pub fn check_choice_index(&self, index: usize) -> bool {
        let res = self.tile_map.map.get(index);
        if res.is_none() {
            return false;
        }
        let cell = res.unwrap();
        //校验地图块合法性
        if cell.id < CellType::Valid as u32 {
            return false;
        }
        //校验地图块是否被锁住
        if cell.check_is_locked() {
            return false;
        }
        //校验是否是世界块
        if cell.is_world {
            return false;
        }
        true
    }

    ///新建战斗回合定时器任务
    pub fn build_battle_turn_task(&self) {
        let next_turn_index = self.next_turn_index;
        let user_id = self.turn_orders.get(next_turn_index);
        if user_id.is_none() {
            error!(
                "user_id is none!next_turn_index:{},turn_orders:{:?}",
                next_turn_index, self.turn_orders
            );
            return;
        }
        let user_id = user_id.unwrap();
        let time_limit = self.turn_limit_time;
        let mut task = Task::default();
        task.delay = time_limit;
        task.cmd = TaskCmd::BattleTurnTime as u16;

        let mut map = serde_json::Map::new();
        map.insert("user_id".to_owned(), serde_json::Value::from(*user_id));
        task.data = serde_json::Value::from(map);
        let res = self.task_sender.send(task);
        if res.is_err() {
            error!("{:?}", res.err().unwrap());
        }
    }

    ///计算范围
    /// 当targets和scope_temp为None时,以⭕️为校验范围有没有人
    /// 当targets为None,scope_temp为Some则校验scope_temp范围内有没有人
    /// 当targets和scope_temp都不为None时，校验targets是否在scope_temp范围内
    pub fn cal_scope(
        &self,
        user_id: u32,
        center_index: isize,
        target_type: TargetType,
        targets: Option<Vec<u32>>,
        scope_temp: Option<&SkillScopeTemp>,
    ) -> anyhow::Result<Vec<u32>> {
        let mut v = Vec::new();
        //相邻，直接拿常量
        if targets.is_none() && scope_temp.is_none() {
            let ss = TRIGGER_SCOPE_NEAR;
            for cell_index in ss.iter() {
                let index = center_index + *cell_index;
                if index < 0 {
                    continue;
                }
                let index = index as usize;
                let cell = self.tile_map.map.get(index);
                if cell.is_none() {
                    continue;
                }
                let cell = cell.unwrap();
                if cell.user_id <= 0 {
                    continue;
                }
                //不能选中自己
                if cell.user_id == user_id {
                    continue;
                }
                v.push(cell.user_id);
            }
        } else if targets.is_none() && scope_temp.is_some() {
            let scope_temp = scope_temp.unwrap();
            //否则校验选中的区域
            for dir in scope_temp.scope.iter() {
                for scope_index in dir.direction.iter() {
                    let index = center_index + *scope_index as isize;
                    if index < 0 {
                        continue;
                    }
                    let index = index as usize;
                    let cell = self.tile_map.map.get(index);
                    if cell.is_none() {
                        continue;
                    }
                    let cell = cell.unwrap();
                    if cell.user_id <= 0 {
                        continue;
                    }
                    //不能选中自己
                    if cell.user_id == user_id {
                        continue;
                    }
                    v.push(cell.user_id);
                }
            }
        } else {
            let targets = targets.unwrap();
            let scope_temp = scope_temp.unwrap();
            //否则校验选中的区域
            for dir in scope_temp.scope.iter() {
                for scope_index in dir.direction.iter() {
                    for index in targets.iter() {
                        let res = center_index - *scope_index as isize;
                        if res != *index as isize {
                            continue;
                        }
                        let cell = self.tile_map.map.get(res as usize);
                        if cell.is_none() {
                            warn!("there is no cell!index:{}", res);
                            continue;
                        }
                        let cell = cell.unwrap();
                        let other_user = cell.user_id;
                        //如果目标不能是自己，就跳过
                        if (target_type == TargetType::OtherAllPlayer
                            || target_type == TargetType::OtherAnyPlayer)
                            && other_user == user_id
                        {
                            continue;
                        }
                        let cter = self.get_battle_cter(Some(other_user));
                        if let Err(e) = cter {
                            warn!("{:?}", e);
                            continue;
                        }
                        let cter = cter.unwrap();
                        v.push(cter.user_id);
                    }
                }
            }
        }
        Ok(v)
    }
}
