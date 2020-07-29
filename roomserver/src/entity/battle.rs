use super::*;
use crate::entity::character::{BattleCharacter, Buff, Item};
use crate::entity::map_data::{Cell, CellType, TileMap};
use crate::entity::room::MEMBER_MAX;
use crate::handlers::battle_handler::{Delete, Find};
use crate::task_timer::{Task, TaskCmd};
use crate::TEMPLATES;
use log::{error, info, warn};
use protobuf::Message;
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::str::FromStr;
use tools::cmd_code::ClientCode;
use tools::protos::base::ActionUnitPt;
use tools::protos::battle::S_ACTION_NOTICE;
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

///默认每个turn翻地图块次数
pub static TURN_DEFAULT_OPEN_CELL_TIMES: u8 = 2;

///触发范围一圈不包括中心
pub static TRIGGER_SCOPE_NEAR: [isize; 6] = [-6, -5, -1, 1, 5, 6];

///技能判定枚举
pub enum SkillID {
    ChangeIndex = 111, //换地图块位置
    ShowIndex = 112,   //展示地图块
}

//技能消耗类型
pub enum SkillConsumeType {
    Energy = 1, //能量
}

///回合行为类型
#[derive(Clone, Debug, PartialEq)]
pub enum BattleCterState {
    Alive = 0,
    Die = 1,
}

///回合行为类型
#[derive(Clone, Debug, PartialEq)]
pub enum ActionType {
    ///无效值
    None = 0,
    ///普通攻击
    Attack = 1,
    ///使用道具
    UseItem = 2,
    ///跳过turn
    Skip = 3,
    ///翻块
    Open = 4,
    ///使用技能
    Skill = 5,
}

impl From<u32> for ActionType {
    fn from(action_type: u32) -> Self {
        match action_type {
            1 => ActionType::Attack,
            2 => ActionType::UseItem,
            3 => ActionType::Skip,
            4 => ActionType::Open,
            5 => ActionType::Skill,
            _ => ActionType::None,
        }
    }
}

///目标类型枚举
#[derive(Clone, Debug, PartialEq)]
pub enum TargetType {
    None = 0,            //无效目标
    Cell = 1,            //地图块
    AnyPlayer = 2,       //任意玩家
    PlayerSelf = 3,      //玩家自己
    AllPlayer = 4,       //所有玩家
    OtherAllPlayer = 5,  //除自己外所有玩家
    OtherAnyPlayer = 6,  //除自己外任意玩家
    UnOpenCell = 7,      //未翻开的地图块
    UnPairCell = 8,      //未配对的地图块
    NullCell = 9,        //空的地图块，上面没人
    UnPairNullCell = 10, //未配对的地图块
    CellPlayer = 11,     //地图块上的玩家
}

impl From<u32> for TargetType {
    fn from(value: u32) -> Self {
        match value {
            1 => TargetType::Cell,
            2 => TargetType::AnyPlayer,
            3 => TargetType::PlayerSelf,
            4 => TargetType::AllPlayer,
            5 => TargetType::OtherAllPlayer,
            6 => TargetType::OtherAnyPlayer,
            7 => TargetType::UnOpenCell,
            8 => TargetType::UnPairCell,
            9 => TargetType::NullCell,
            10 => TargetType::UnPairNullCell,
            11 => TargetType::CellPlayer,
            _ => TargetType::None,
        }
    }
}

///元素类型
pub enum ElementType {
    Nature = 1, //生命元素
    Water = 2,  //水元素
    Earth = 3,  //土元素
    Fire = 4,   //火元素
}

///行动单位
#[derive(Clone, Debug, Default)]
pub struct ActionUnit {
    pub team_id: u32,
    pub user_id: u32,
    pub turn_index: u32,
    pub actions: Vec<Action>,
}

#[derive(Clone, Debug, Default)]
pub struct Action {
    action_type: u8,
    action_value: u32,
}
///房间战斗数据封装
#[derive(Clone, Debug)]
pub struct BattleData {
    pub tile_map: TileMap,                          //地图数据
    pub choice_orders: [u32; 4],                    //选择顺序里面放玩家id
    pub next_choice_index: usize,                   //下一个选择的下标
    pub next_turn_index: usize,                     //下个turn的下标
    pub turn_action: ActionUnit,                    //当前回合数据单元封装
    pub turn_orders: [u32; 4],                      //turn行动队列，里面放玩家id
    pub battle_cter: HashMap<u32, BattleCharacter>, //角色战斗数据
    task_sender: crossbeam::Sender<Task>,           //任务sender
    sender: TcpSender,                              //sender
}

impl BattleData {
    pub fn new(task_sender: crossbeam::Sender<Task>, sender: TcpSender) -> Self {
        BattleData {
            tile_map: TileMap::default(),
            choice_orders: [0; 4],
            next_choice_index: 0,
            next_turn_index: 0,
            turn_action: ActionUnit::default(),
            turn_orders: [0; 4],
            battle_cter: HashMap::new(),
            task_sender,
            sender,
        }
    }

    //检测地图块是否可以翻
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

    ///下个回合
    pub fn next_turn(&mut self) {
        //计算下一个回合
        self.add_next_turn_index();
        //开始回合触发
        self.turn_start_settlement();

        //todo 通知客户端
    }

    pub fn add_next_turn_index(&mut self) {
        self.next_turn_index += 1;
        let index = self.next_turn_index;
        if index >= (MEMBER_MAX - 1) as usize {
            return;
        }
        let user_id = self.get_turn_user(Some(index));
        if let Ok(user_id) = user_id {
            if user_id != 0 {
                return;
            }
            self.add_next_turn_index();
        } else {
            warn!("{:?}", user_id.err().unwrap());
        }
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
            return anyhow::bail!(str);
        }
        let user_id = *res.unwrap();
        Ok(user_id)
    }

    pub fn get_next_turn_user(&self) -> anyhow::Result<u32> {
        let index = self.next_turn_index;
        let res = self.turn_orders.get(index);
        if let None = res {
            let str = format!("get_next_turn_user is none!index:{}", index);
            anyhow::bail!(str)
        }
        let user_id = *res.unwrap();
        Ok(user_id)
    }

    ///翻地图块
    pub fn open_cell(&mut self, index: usize) {
        //todo 此处应该计算技能cd
        let user_id = self.get_next_turn_user();
        if let Err(e) = user_id {
            error!("{:?}", e);
            return;
        }
        let user_id = user_id.unwrap();
        let is_pair;
        unsafe {
            let battle_cters = &mut self.battle_cter as *mut HashMap<u32, BattleCharacter>;
            let battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();
            //处理配对和角色换位置逻辑
            is_pair = self.handler_cell_pair(user_id, index);
            //处理配对成功触发buff逻辑
            if is_pair {
                self.handler_cell_pair_buff(user_id, index);
            }
            //处理地图块额外其他的buff
            self.handler_cell_extra_buff(user_id, index);

            //处理移动后的事件
            self.handler_cter_move(user_id, index);

            //更新最近一次翻的下标
            battle_cter.recently_open_cell_index = index as isize;

            //翻块次数-1
            battle_cter.residue_open_times -= 1;

            //玩家技能cd-1
            for skill in battle_cter.skills.iter_mut() {
                skill.cd_times -= 1;
                if skill.cd_times <= 0 {
                    skill.cd_times = skill.skill_temp.cd as i8;
                }
            }

            //todo 下发到客户端

            //如果没有剩余翻块次数了，就下一个turn
            if battle_cter.residue_open_times <= 0 {
                self.next_turn();
            }
        }
    }

    ///处理角色移动之后的事件
    pub unsafe fn handler_cter_move(&mut self, user_id: u32, index: usize) {
        let index = index as isize;
        let battle_cters = &mut self.battle_cter as *mut HashMap<u32, BattleCharacter>;
        let cter = self.battle_cter.get_mut(&user_id).unwrap();

        //踩到别人到范围
        for other_cter in battle_cters.as_mut().unwrap().values_mut() {
            let cter_index = other_cter.cell_index as isize;
            for buff in other_cter.buff_array.iter() {
                if buff.id != 1 {
                    continue;
                }

                for scope_index in TRIGGER_SCOPE_NEAR.iter() {
                    let res = cter_index + scope_index;
                    if index != res {
                        continue;
                    }
                    cter.sub_hp(buff.buff_temp.par1 as i32);
                    break;
                }
                if cter.is_died() {
                    //todo  处理角色死亡事件
                    break;
                }
            }
            //触发别人进入自己的范围
            if cter.user_id == other_cter.user_id {
                continue;
            }
            for buff in cter.buff_array.iter() {
                if buff.id != 1 {
                    continue;
                }
                for scope_index in TRIGGER_SCOPE_NEAR.iter() {
                    let res = index + scope_index;
                    if cter_index != res {
                        continue;
                    }
                    other_cter.sub_hp(buff.buff_temp.par1 as i32);
                    if other_cter.is_died() {
                        //todo  处理角色死亡事件
                        break;
                    }
                    break;
                }
            }
        }
    }

    ///处理地图块额外其他buff
    pub unsafe fn handler_cell_extra_buff(&mut self, user_id: u32, index: usize) {
        let battle_cters = &mut self.battle_cter as *mut HashMap<u32, BattleCharacter>;

        let battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();

        let cell = self.tile_map.map.get_mut(index).unwrap();

        for buff in cell.extra_buff.iter() {}
    }

    ///处理地图块配对逻辑
    pub unsafe fn handler_cell_pair(&mut self, user_id: u32, index: usize) -> bool {
        let battle_cters = &mut self.battle_cter as *mut HashMap<u32, BattleCharacter>;

        let battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();

        let recently_open_cell_index = battle_cter.recently_open_cell_index;
        let recently_open_cell_id = self
            .tile_map
            .map
            .get_mut(recently_open_cell_index as usize)
            .unwrap()
            .id;

        let cell = self.tile_map.map.get_mut(index).unwrap() as *mut Cell;
        let cell = &mut *cell;
        let is_pair;
        let last_cell = self
            .tile_map
            .map
            .get_mut(recently_open_cell_index as usize)
            .unwrap() as *mut Cell;
        let last_cell = &mut *last_cell;
        let cell_id = cell.id;
        //如果配对了，则修改地图块配对的下标
        if cell_id == recently_open_cell_id {
            cell.pair_index = Some(recently_open_cell_index as usize);
            last_cell.pair_index = Some(index);
            is_pair = true;
        } else {
            is_pair = false;
        }

        //判断改地图块上面有没有角色，有的话将目标位置的玩家挪到操作玩家的位置上
        if cell.user_id > 0 {
            let target_cter = self.battle_cter.get_mut(&cell_id).unwrap();
            target_cter.cell_index = battle_cter.cell_index;

            let source_cell = self.tile_map.map.get_mut(battle_cter.cell_index).unwrap();
            source_cell.user_id = target_cter.user_id;
        }
        //改变角色位置
        battle_cter.cell_index = index;
        cell.user_id = battle_cter.user_id;
        is_pair
    }

    ///处理地图块配对成功之后的buff
    pub unsafe fn handler_cell_pair_buff(&mut self, user_id: u32, index: usize) {
        let battle_cters = self.battle_cter.borrow_mut() as *mut HashMap<u32, BattleCharacter>;
        let battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();
        let cell = self.tile_map.map.get(index).unwrap();
        let last_index = battle_cter.recently_open_cell_index as usize;
        let last_cell = self.tile_map.map.get(last_index).unwrap();
        let cell_temp = TEMPLATES.get_cell_ref().get_temp(&cell.id).unwrap();
        for buff in cell.buff.iter() {
            //获得道具
            if [30011, 30021, 30031, 30041].contains(&buff.id) {
                let item_id = buff.buff_temp.par1;
                let item = TEMPLATES.get_item_ref().get_temp(&item_id);
                if let Err(e) = item {
                    error!("{:?}", e);
                    continue;
                }
                let item_temp = item.unwrap();
                let skill_id = item_temp.trigger_skill;
                let skill_temp = TEMPLATES.get_skill_ref().get_temp(&skill_id);
                if let Err(e) = skill_temp {
                    error!("{:?}", e);
                    continue;
                }
                let skill_temp = skill_temp.unwrap();
                let item = Item {
                    id: item_id,
                    skill_temp,
                };
                //判断目标类型，若是地图块上的玩家，则判断之前那个地图块上有没有玩家，有就给他道具
                if buff.buff_temp.target == TargetType::CellPlayer as u32 {
                    let last_cell_user = battle_cters.as_mut().unwrap().get_mut(&last_cell.user_id);
                    if let Some(last_cell_user) = last_cell_user {
                        last_cell_user.items.insert(item_id, item.clone());
                    }
                }
                battle_cter.items.insert(item_id, item);
            //todo 通知客户端
            } else if 30012 == buff.id {
                //配对恢复生命
                if buff.buff_temp.target == TargetType::CellPlayer as u32 {
                    let last_cell_user = battle_cters.as_mut().unwrap().get_mut(&last_cell.user_id);
                    if let Some(last_cell_user) = last_cell_user {
                        last_cell_user.add_hp(buff.buff_temp.par1 as i32);
                    }
                }
                //恢复生命值
                battle_cter.add_hp(buff.buff_temp.par1 as i32);
            //todo 通知客户端
            } else if 30022 == buff.id {
                //获得一个buff

                let buff_temp = TEMPLATES.get_buff_ref().get_temp(&buff.buff_temp.par1);
                if let Err(e) = buff_temp {
                    error!("{:?}", e);
                    continue;
                }
                let buff_temp = buff_temp.unwrap();
                let buff = Buff::from(buff_temp);
                let target_type = TargetType::from(buff.buff_temp.target);

                //如果目标类型是地图块上的玩家
                if target_type == TargetType::CellPlayer {
                    let last_cell_user = battle_cters.as_mut().unwrap().get_mut(&last_cell.user_id);
                    if let Some(last_cell_user) = last_cell_user {
                        last_cell_user.buff_array.push(buff.clone());
                    }
                }
                //恢复生命值
                battle_cter.buff_array.push(buff);
            //todo 通知客户端
            } else if [30022].contains(&buff.id) {
                //获得buff
                let buff_temp = TEMPLATES.get_buff_ref().get_temp(&buff.buff_temp.par1);
                if let Err(e) = buff_temp {
                    error!("{:?}", e);
                    continue;
                }
                let buff_temp = buff_temp.unwrap();
                let buff = Buff::from(buff_temp);

                //判断目标类型，若是地图块上的玩家，则判断之前那个地图块上有没有玩家，有就给他道具
                if buff.buff_temp.target == TargetType::CellPlayer as u32 {
                    let last_cell_user = battle_cters.as_mut().unwrap().get_mut(&last_cell.user_id);
                    if let Some(last_cell_user) = last_cell_user {
                        last_cell_user.buff_array.push(buff.clone());
                    }
                }
                battle_cter.buff_array.push(buff);
            //todo 通知客户端
            } else if [30032].contains(&buff.id) {
                //相临的玩家技能cd增加
                let isize_index = index as isize;
                for cter in self.battle_cter.values_mut() {
                    if cter.user_id == user_id {
                        continue;
                    }
                    let cter_index = cter.cell_index;
                    for scope_index in TRIGGER_SCOPE_NEAR.iter() {
                        let res = isize_index + *scope_index;
                        if cter_index != cter_index {
                            continue;
                        }
                        for skill in cter.skills.iter_mut() {
                            skill.cd_times += buff.buff_temp.par1 as i8;
                        }
                    }
                }
            //todo 通知客户端
            } else if [30042].contains(&buff.id) {
                //相临都玩家造成技能伤害
                let scope_temp = TEMPLATES
                    .get_skill_scope_ref()
                    .get_temp(&buff.buff_temp.scope);
                if let Err(e) = scope_temp {
                    error!("{:?}", e);
                    continue;
                }
                let scope_temp = scope_temp.unwrap();
                let isize_index = index as i32;
                for cter in self.battle_cter.values_mut() {
                    if cter.user_id == user_id {
                        continue;
                    }
                    let other_index = cter.cell_index as i32;
                    for dir in scope_temp.scope.iter() {
                        for scope_index in dir.direction.iter() {
                            let res = isize_index + *scope_index;
                            if other_index != res {
                                continue;
                            }
                            //造成技能伤害
                            cter.sub_hp(buff.buff_temp.par1 as i32);
                        }
                    }
                }
            //todo 通知客户端
            } else if [9].contains(&buff.id) {
                //处理世界块的逻辑
                //配对属性一样的地图块+hp
                //查看配对的cell的属性是否与角色属性匹配
                if cell_temp.element != battle_cter.element {
                    return;
                }
                //获得buff
                battle_cter.add_hp(buff.buff_temp.par1 as i32);
                //todo 通知客户端
            }
        }
    }

    ///回合开始触发
    pub fn turn_start_settlement(&mut self) {
        let user_id = self.get_turn_user(None);
        if let Err(e) = user_id {
            error!("{:?}", e);
            return;
        }
        let user_id = user_id.unwrap();
        let mut battle_cter = self.battle_cter.get_mut(&user_id);
        if let None = battle_cter {
            error!("battle_cter is None!user_id:{}", user_id);
            return;
        }
        let battle_cter = battle_cter.unwrap();

        //玩家身上所有buff持续轮次-1
        let mut need_remove = Vec::new();
        let mut index = 0_usize;
        for buff in battle_cter.buff_array.iter_mut() {
            buff.keep_times -= 1;
            if buff.keep_times <= 0 {
                need_remove.push(index);
            }
            index += 1;
        }
        //删除buff
        for index in need_remove {
            battle_cter.buff_array.remove(index);
        }

        //创建战斗检测定时器任务
        self.build_battle_turn_task();
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

    ///普通攻击
    pub fn attack(&mut self, user_id: u32, targets: Vec<u32>) {
        let damege = self.calc_damage(user_id);
        for target_id in targets.iter() {
            let reduce_damage = self.calc_reduce_damage(*target_id);
            let res = damege - reduce_damage;
            let battle_cter = self.battle_cter.get_mut(&user_id).unwrap();
            battle_cter.sub_hp(res as i32);

            //todo 将计算结果推送给客户端
        }
    }

    ///计算伤害
    fn calc_damage(&self, user_id: u32) -> isize {
        let battle_cter = self.battle_cter.get(&user_id).unwrap();
        let damage = battle_cter.atk;
        //todo 此处应该加上角色身上的buff加成
        damage as isize
    }

    ///计算减伤
    fn calc_reduce_damage(&self, user_id: u32) -> isize {
        let battle_cter = self.battle_cter.get(&user_id).unwrap();
        let mut value = battle_cter.defence;
        //todo 此处应该加上角色身上的减伤buff
        value as isize
    }

    pub fn send_2_client(&mut self, cmd: ClientCode, user_id: u32, bytes: Vec<u8>) {
        let bytes = Packet::build_packet_bytes(cmd as u32, user_id, bytes, true, true);
        self.get_sender_mut().write(bytes);
    }

    pub fn get_sender_mut(&mut self) -> &mut TcpSender {
        self.sender.borrow_mut()
    }

    //跳过回合
    pub fn skip_turn(&mut self) {
        //返回客户端
        let mut san = S_ACTION_NOTICE::new();
        let mut apt = ActionUnitPt::new();
        apt.set_field_type(ActionType::Skip as u32);
        apt.set_from_user(self.get_next_turn_user().unwrap());
        san.action_uints.push(apt);
        let res = san.write_to_bytes();
        if let Err(e) = res {
            error!("{:?}", e);
            return;
        }
        let bytes = res.unwrap();
        for member_id in self.battle_cter.clone().keys() {
            self.send_2_client(ClientCode::ActionNotice, *member_id, bytes.clone());
        }
        //下一个turn
        self.next_turn();
    }

    ///校验是否翻过块
    pub fn check_is_open(&self) -> bool {
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

    ///使用道具,道具都是一次性的，用完了就删掉
    pub fn use_item(&mut self, item_id: u32) {
        let user_id = self.get_next_turn_user().unwrap();
        let battle_cter = self.battle_cter.get(&user_id).unwrap();
        let item = battle_cter.items.get(&item_id).unwrap();
        let mut targets = Vec::new();
        targets.push(user_id);
        self.use_skill(item.skill_temp.id, targets);
        let battle_cter = self.battle_cter.get_mut(&user_id).unwrap();
        //用完了就删除
        battle_cter.items.remove(&item_id);
    }

    ///使用技能
    pub fn use_skill(&mut self, skill_id: u32, target_array: Vec<u32>) {
        //如果目标
        if target_array.is_empty() {
            return;
        }
        let user_id = *self.turn_orders.get(self.next_turn_index).unwrap();
        unsafe {
            //战斗角色
            let battle_cter_ptr =
                self.battle_cter.get_mut(&user_id).unwrap() as *mut BattleCharacter;
            let battle_cter = battle_cter_ptr.as_mut().unwrap();
            //战斗角色身上的技能
            let skill = battle_cter.skills.get_mut((skill_id as usize)).unwrap();
            //校验cd
            if skill.cd_times > 0 {
                warn!(
                    "can not use this skill!skill_id:{},cd:{}",
                    skill_id, skill.cd_times
                );
                return;
            }
            //技能判定
            let skill_judge = skill.skill_temp.skill_judge;
            if skill_judge != 0 {
                let skill_judge_temp = TEMPLATES.get_skill_judge_ref().get_temp(&(skill_id as u32));
                if let Ok(skill_judge) = skill_judge_temp {
                    // todo  目前没有判定条件，先留着
                }
            }

            let target = skill.skill_temp.target;
            let target_type = TargetType::from(target);

            //校验所选目标范围
            let scope = skill.skill_temp.scope;
            let skill_scope_temp = TEMPLATES.get_skill_scope_ref().temps.get(&scope).unwrap();

            //校验目标类型
            let res = self.check_target_array(user_id, target_type, &target_array);
            if !res {
                return;
            }

            for direction in skill_scope_temp.scope.iter() {
                for value in direction.direction.iter() {
                    if value == &0 {
                        continue;
                    }
                }
            }

            //换地图块位置
            if skill_id == SkillID::ChangeIndex as u32 {
                if target_array.len() < 2 {
                    return;
                }
                let source_index = target_array.get(0).unwrap();
                let target_index = target_array.get(1).unwrap();

                let source_index = *source_index as usize;
                let target_index = *target_index as usize;
                self.change_index(user_id, source_index, target_index);
            } else if [20001, 112].contains(&skill_id) {
                //展示地图块
                if target_array.is_empty() {
                    return;
                }
                let index = *target_array.get(0).unwrap() as usize;
                self.show_index(index);
            } else if [121, 211, 221, 311, 322, 20002].contains(&skill_id) {
                //上持续性buff
                self.add_buff(skill_id, target_array);
            // self.damge_near_user_move_to(skill_id);
            } else if [212].contains(&skill_id) {
                //将1个地图块自动配对。本回合内不能攻击。
            } else if [222].contains(&skill_id) {
                //选择一个玩家，将其移动到一个空地图块上。
                if target_array.len() < 2 {
                    warn!("move_user,the target_array size is:{}", target_array.len());
                    return;
                }
                let target_user = target_array.get(0).unwrap();
                let target_index = target_array.get(1).unwrap();
                self.move_user(*target_user, *target_index as usize);
            } else if [321].contains(&skill_id) {
                //对你相邻的所有玩家造成1点技能伤害，并回复等于造成伤害值的生命值。
            } else if [411, 421, 20004, 20005].contains(&skill_id) {
                //造成技能伤害
            } else if [20003].contains(&skill_id) {
                //目标的技能CD-2。
            }
            //把技能cd+上
            skill.cd_times = skill.skill_temp.cd as i8;
            //todo 通知客户端
        }
    }

    ///移动玩家
    fn move_user(&mut self, target_user: u32, target_index: usize) {
        //校验下标的地图块
        let target_cell = self.tile_map.map.get_mut(target_index);
        if let None = target_cell {
            warn!("there is no cell!index:{}", target_index);
            return;
        }
        let target_cell = target_cell.unwrap();
        //校验有效性
        if target_cell.id < CellType::Valid as u32 {
            warn!("this cell can not be choice!index:{}", target_index);
            return;
        }
        //校验世界块
        if target_cell.is_world {
            warn!("world cell can not be choice!index:{}", target_index);
            return;
        }

        target_cell.user_id = target_user;

        let target_cter = self.battle_cter.get_mut(&target_user);
        if let None = target_cter {
            warn!("there is no cter!user_id:{}", target_user);
            return;
        }

        //更新目标玩家的下标
        let target_cter = target_cter.unwrap();
        let last_index = target_cter.cell_index;
        target_cter.cell_index = target_index;
        //重制之前地图块上的玩家id
        let last_cell = self.tile_map.map.get_mut(last_index).unwrap();
        last_cell.user_id = 0;

        //处理移动后事件
        unsafe {
            self.handler_cter_move(target_user, target_index);
        }
        //todo 通知客户的
    }

    fn check_target_array(
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
            } //地图块
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
            } //任意玩家
            TargetType::PlayerSelf => {}      //玩家自己
            TargetType::AllPlayer => {
                for member_id in target_array {
                    if !self.battle_cter.contains_key(&member_id) {
                        return false;
                    }
                }
                return true;
            } //所有玩家
            TargetType::OtherAllPlayer => {
                for member_id in target_array {
                    if member_id != &user_id && !self.battle_cter.contains_key(&user_id) {
                        return false;
                    }
                }
                return true;
            } //除自己外所有玩家
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
            } //除自己外任意玩家
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
            } //未翻开的地图块
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
            } //未配对的地图块
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
            } //未配对的空地图块
            TargetType::CellPlayer => {}
        }
        true
    }

    ///上buff
    pub fn add_buff(&mut self, skill_id: u32, target_array: Vec<u32>) {
        let use_id = self.get_next_turn_user().unwrap();
        //121, 211, 221, 311, 322, 20002
        let skill_temp = TEMPLATES.get_skill_ref().get_temp(&skill_id).unwrap();
        //先计算单体的
        let buff_id = skill_temp.buff as u32;
        let buff_temp = TEMPLATES.get_buff_ref().get_temp(&buff_id).unwrap();
        let buff = Buff::from(buff_temp);
        let target_type = TargetType::from(skill_temp.target);

        match target_type {
            TargetType::PlayerSelf => {
                let cter = self.battle_cter.get_mut(&use_id).unwrap();
                cter.buff_array.push(buff);
            }
            TargetType::UnPairNullCell => {
                let index = *target_array.get(0).unwrap() as usize;
                let cell = self.tile_map.map.get_mut(index);
                if cell.is_none() {
                    warn!("cell not find!index:{}", index);
                    return;
                }
                let cell = cell.unwrap();
                if cell.is_world {
                    warn!("world_cell can not be choice!index:{}", index);
                    return;
                }
                if cell.pair_index.is_some() {
                    warn!("this cell is already paired!index:{}", index);
                    return;
                }
                cell.extra_buff.push(buff);
            }
            _ => {}
        }
    }

    //其他玩家移动到与你相邻的地图块时，对其造成3点伤害。持续1轮。
    pub fn damge_near_user_move_to(&mut self, skill_id: u32) {
        let user_id = self.get_next_turn_user();
        if let Err(e) = user_id {
            error!("{:?}", e);
            return;
        }
        let user_id = user_id.unwrap();
        let battle_cter = self.battle_cter.get_mut(&user_id).unwrap();
        let skill = battle_cter.skills.get(skill_id as usize);
        //校验技能合法性
        if skill.is_none() {
            return;
        }
        let skill = skill.unwrap();

        //校验buff
        let buff_id = skill.skill_temp.buff as u32;
        let buff = TEMPLATES.get_buff_ref().get_temp(&buff_id);
        if let Err(e) = buff {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff.unwrap();
        let battle_cter = self.battle_cter.get_mut(&user_id);
        if battle_cter.is_none() {
            return;
        }
        let battle_cter = battle_cter.unwrap();
        let mut buff = Buff::from(buff_temp);
        battle_cter.buff_array.push(buff);
        //todo 通知客户端
    }

    ///展示地图块
    pub fn show_index(&mut self, index: usize) {
        //校验index合法性
        let cell = self.tile_map.map.get(index);
        if cell.is_none() {
            return;
        }
        //校验index合法性
        let cell = cell.unwrap();
        if cell.id < CellType::Valid as u32 {
            return;
        }

        //校验是否世界块
        if cell.is_world {
            return;
        }

        //翻开的地图块不能展示
        if let Some(pair_index) = cell.pair_index {
            if pair_index > 0 {
                return;
            }
        }

        //todo 下发给客户端
    }

    ///地图块换位置
    pub fn change_index(&mut self, user_id: u32, source_index: usize, target_index: usize) {
        let lock_skills = &TEMPLATES.get_skill_ref().lock_skills[..];
        let map_size = self.tile_map.map.len();
        //校验地图块
        if source_index > map_size || target_index > map_size {
            return;
        }
        let source_cell = self.tile_map.map.get(source_index).unwrap();
        let target_cell = self.tile_map.map.get(target_index).unwrap();

        //无效块不能换，锁定不能换
        if source_cell.id <= 1 || target_cell.id <= 1 {
            return;
        }
        //已配对的块不能换
        if source_cell.pair_index.is_some() || target_cell.pair_index.is_some() {
            return;
        }
        //锁定不能换
        for skill in source_cell.buff.iter() {
            if lock_skills.contains(&skill.id) {
                return;
            }
        }
        //锁定不能换
        for skill in target_cell.buff.iter() {
            if lock_skills.contains(&skill.id) {
                return;
            }
        }

        //先删掉
        let mut source_cell = self.tile_map.map.remove(source_index);
        let mut target_cell = self.tile_map.map.remove(target_index);

        //替换下标
        source_cell.index = target_index;
        target_cell.index = source_index;

        self.tile_map.map.insert(source_cell.index, source_cell);
        self.tile_map.map.insert(target_cell.index, target_cell);

        //todo 通知客户端
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
        let time_limit = TEMPLATES
            .get_constant_ref()
            .temps
            .get("battle_turn_limit_time");
        let mut task = Task::default();
        if let Some(time) = time_limit {
            let time = u64::from_str(time.value.as_str());
            match time {
                Ok(time) => {
                    task.delay = time + 500;
                }
                Err(e) => {
                    task.delay = 5000_u64;
                    error!("{:?}", e);
                }
            }
        } else {
            task.delay = 5000_u64;
            warn!("the battle_turn_limit_time of Constant config is None!pls check!");
        }
        task.cmd = TaskCmd::ChoiceTurnOrder as u16;

        let mut map = serde_json::Map::new();
        map.insert("user_id".to_owned(), serde_json::Value::from(*user_id));
        task.data = serde_json::Value::from(map);
        let res = self.task_sender.send(task);
        if res.is_err() {
            error!("{:?}", res.err().unwrap());
        }
    }
}
