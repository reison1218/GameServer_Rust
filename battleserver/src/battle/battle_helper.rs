use crate::battle::battle::BattleData;
use crate::battle::battle_enum::skill_judge_type::{
    HP_LIMIT_GT, LIMIT_ROUND_TIMES, LIMIT_TURN_TIMES,
};
use crate::battle::battle_enum::{
    AttackState, BattleCterState, EffectType, TargetType, TRIGGER_SCOPE_NEAR_TEMP_ID,
};
use crate::battle::battle_trigger::TriggerEvent;
use crate::room::character::BattleCharacter;
use crate::room::map_data::{MapCell, MapCellType, TileMap};
use crate::room::MEMBER_MAX;
use crate::task_timer::{Task, TaskCmd};
use crate::TEMPLATES;
use log::{error, info, warn};
use protobuf::Message;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;
use tools::cmd_code::ClientCode;
use tools::protos::base::{ActionUnitPt, BuffPt, CellBuffPt, EffectPt, TargetPt, TriggerEffectPt};
use tools::protos::battle::S_BATTLE_TURN_NOTICE;
use tools::templates::skill_scope_temp::SkillScopeTemp;
use tools::util::packet::Packet;

impl BattleData {
    ///检测地图刷新
    pub fn check_refresh_map(&mut self) -> bool {
        let allive_count = self
            .battle_cter
            .values()
            .filter(|x| x.status.state == BattleCterState::Alive)
            .count();

        let un_open_count = self.tile_map.un_pair_map.len();
        let mut need_reflash_map = false;
        if un_open_count <= 2 {
            need_reflash_map = true;
        }
        if allive_count >= 2 && need_reflash_map {
            return true;
        }
        false
    }

    pub fn clear_open_cells(&mut self) {
        let index = self.next_turn_index;
        let res = self.turn_orders.get(index);
        if res.is_none() {
            return;
        }
        let user_id = *res.unwrap();
        let cter = self.battle_cter.get(&user_id);
        if let None = cter {
            return;
        }
        let cter = cter.unwrap();
        for index in cter.flow_data.open_map_cell_vec.iter() {
            let map_cell = self.tile_map.map_cells.get_mut(*index);
            if let Some(map_cell) = map_cell {
                map_cell.open_user = 0;
            }
        }
    }

    ///下一个
    pub fn add_next_turn_index(&mut self) {
        self.next_turn_index += 1;
        self.add_total_turn_times();
        let index = self.next_turn_index;
        if index >= MEMBER_MAX as usize {
            self.next_turn_index = 0;
        }
        //开始回合触发
        self.turn_start_summary();

        let user_id = self.get_turn_user(None);
        if let Ok(user_id) = user_id {
            if user_id == 0 {
                self.add_next_turn_index();
                return;
            }

            let cter_res = self.get_battle_cter(Some(user_id), false);
            match cter_res {
                Ok(cter) => {
                    if cter.is_died() {
                        self.add_next_turn_index();
                        return;
                    }
                    if cter.robot_data.is_some() {
                        cter.robot_start_action();
                    }
                }
                Err(e) => {
                    warn!("{:?}", e);
                }
            }
        } else {
            warn!("{:?}", user_id.err().unwrap());
        }
    }

    ///处理角色移动之后的事件,返回元组类型，第一个表示移动角色死了没，第二个封装proto
    pub unsafe fn handler_cter_move(
        &mut self,
        user_id: u32,
        index: usize,
        au: &mut ActionUnitPt,
    ) -> anyhow::Result<(bool, Vec<ActionUnitPt>)> {
        let battle_cters = &mut self.battle_cter as *mut HashMap<u32, BattleCharacter>;
        let battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();
        let battle_cter_index = battle_cter.get_map_cell_index();
        let tile_map_ptr = self.tile_map.borrow_mut() as *mut TileMap;
        let map_cell = tile_map_ptr
            .as_mut()
            .unwrap()
            .map_cells
            .get_mut(index)
            .unwrap();
        au.action_value.push(map_cell.id);
        let mut is_change_index_both = false;
        let title_map_mut = tile_map_ptr.as_mut().unwrap();
        //判断改地图块上面有没有角色，有的话将目标位置的玩家挪到操作玩家的位置上
        if map_cell.user_id > 0 {
            let target_user = map_cell.user_id;
            //先判断目标位置的角色是否有不动泰山被动技能
            self.before_moved_trigger(user_id, target_user)?;
            //如果没有，则改变目标玩家的位置
            let target_cter = self.get_battle_cter_mut(Some(target_user), true).unwrap();
            target_cter.move_index(battle_cter_index);
            let source_map_cell = title_map_mut.map_cells.get_mut(battle_cter_index).unwrap();
            source_map_cell.user_id = target_cter.get_user_id();
            is_change_index_both = true;
        } else {
            //重制之前地图块上的玩家id
            let last_map_cell = title_map_mut.map_cells.get_mut(battle_cter_index).unwrap();
            last_map_cell.user_id = 0;
        }
        //改变角色位置
        battle_cter.move_index(index);
        map_cell.user_id = user_id;

        let index = index as isize;
        //移动位置后触发事件
        let res = self.after_move_trigger(battle_cter, index, is_change_index_both);
        Ok(res)
    }

    ///消耗buff
    pub unsafe fn consume_buff(
        &mut self,
        buff_id: u32,
        user_id: Option<u32>,
        map_cell_index: Option<usize>,
        is_turn_index: bool,
    ) -> Option<u32> {
        let next_turn_index = self.next_turn_index;
        let mut cter_res: Option<&mut BattleCharacter> = None;
        let mut map_cell_res: Option<&mut MapCell> = None;
        let mut lost_buff = None;
        let cters = self.battle_cter.borrow_mut() as *mut HashMap<u32, BattleCharacter>;
        if user_id.is_some() {
            let user_id = user_id.unwrap();
            let cter = self.get_battle_cter_mut(Some(user_id), true);
            if let Err(e) = cter {
                error!("{:?}", e);
                return lost_buff;
            }
            let cter = cter.unwrap();
            let buff = cter.battle_buffs.buffs.get_mut(&buff_id);
            if buff.is_none() {
                return lost_buff;
            }
            cter_res = Some(cter);
        } else if map_cell_index.is_some() {
            let map_cell_index = map_cell_index.unwrap();
            let map_cell = self.tile_map.map_cells.get_mut(map_cell_index);
            if map_cell.is_none() {
                return lost_buff;
            }
            let map_cell = map_cell.unwrap();
            let buff = map_cell.buffs.get_mut(&buff_id);
            if buff.is_none() {
                return lost_buff;
            }
            map_cell_res = Some(map_cell);
        }

        let buff;
        if cter_res.is_some() {
            buff = cter_res
                .as_mut()
                .unwrap()
                .battle_buffs
                .buffs
                .get_mut(&buff_id);
        } else if map_cell_res.is_some() {
            buff = map_cell_res.as_mut().unwrap().buffs.get_mut(&buff_id);
        } else {
            return lost_buff;
        }
        let buff = buff.unwrap();
        //如果是永久的buff,直接返回
        if buff.permanent {
            return lost_buff;
        }
        let need_remove;
        if is_turn_index && buff.turn_index.is_some() && buff.turn_index.unwrap() == next_turn_index
        {
            buff.sub_keep_times();
        } else if !is_turn_index {
            buff.sub_trigger_timesed()
        }
        let cfg_keep_time = buff.buff_temp.keep_time;
        let cfg_trigger_time = buff.buff_temp.trigger_times;
        //判断触发次数
        if cfg_keep_time == 0 && buff.trigger_timesed <= 0 {
            need_remove = true;
        } else if cfg_trigger_time == 0 && buff.keep_times <= 0 {
            //判断持续时间
            need_remove = true;
        } else if (cfg_trigger_time > 0 && cfg_keep_time > 0)
            && (buff.keep_times == 0 || buff.trigger_timesed == 0)
        {
            //判断双条件
            need_remove = true;
        } else {
            //不然不用删除
            need_remove = false;
        }

        //如果要删除
        if need_remove {
            //处理玩家技能状态
            if buff.from_user.is_some() {
                let from_user = buff.from_user.unwrap();
                let from_cter = cters.as_mut().unwrap().get_mut(&from_user);
                if let Some(from_cter) = from_cter {
                    if let Some(from_skill) = buff.from_skill {
                        let skill = from_cter.skills.get_mut(&from_skill);
                        if let Some(skill) = skill {
                            skill.is_active = false;
                        }
                    }
                }
            }
            //如果是玩家身上的
            if let Some(cter) = cter_res {
                cter.remove_buff(buff_id);
                lost_buff = Some(buff_id);
                let user_id = cter.get_user_id();
                self.buff_lost_trigger(user_id, buff_id);
            } else if let Some(map_cell) = map_cell_res {
                //如果是地图块上面的
                map_cell.remove_buff(buff_id);
                lost_buff = Some(buff_id);
            }
        }
        lost_buff
    }

    ///加血
    pub fn add_hp(
        &mut self,
        from_user: Option<u32>,
        target: u32,
        hp: i16,
        buff_id: Option<u32>,
    ) -> anyhow::Result<TargetPt> {
        let cter = self.get_battle_cter_mut(Some(target), true)?;

        if cter.is_died() {
            anyhow::bail!(
                "this cter is died! user_id:{},cter_id:{}",
                target,
                cter.get_cter_id()
            )
        }
        cter.add_hp(hp);
        let target_pt =
            self.build_target_pt(from_user, target, EffectType::Cure, hp as u32, buff_id)?;
        Ok(target_pt)
    }

    ///计算减伤
    pub fn calc_reduce_damage(&self, from_user: u32, target_cter: &mut BattleCharacter) -> i16 {
        let target_user = target_cter.get_user_id();
        let scope_temp = TEMPLATES
            .get_skill_scope_temp_mgr_ref()
            .get_temp(&TRIGGER_SCOPE_NEAR_TEMP_ID);
        if let Err(_) = scope_temp {
            return target_cter.base_attr.defence as i16;
        }
        let scope_temp = scope_temp.unwrap();
        let (_, user_v) = self.cal_scope(
            target_user,
            target_cter.get_map_cell_index() as isize,
            TargetType::None,
            None,
            Some(scope_temp),
        );
        let res = user_v.contains(&from_user);
        target_cter.calc_reduce_damage(res)
    }

    pub fn get_alive_player_num(&self) -> usize {
        let alive_count = self
            .battle_cter
            .values()
            .filter(|x| x.status.state == BattleCterState::Alive)
            .count();
        alive_count
    }

    ///扣血
    pub unsafe fn deduct_hp(
        &mut self,
        from: u32,
        target: u32,
        skill_damege: Option<i16>,
        is_last_one: bool,
    ) -> anyhow::Result<TargetPt> {
        let battle_data_ptr = self as *mut BattleData;
        let mut target_pt = TargetPt::new();

        let mut ep = EffectPt::new();
        ep.effect_type = EffectType::SkillDamage as u32;

        let target_cter = battle_data_ptr
            .as_mut()
            .unwrap()
            .get_battle_cter_mut(Some(target), true)?;
        let target_user_id = target_cter.base_attr.user_id;
        target_pt
            .target_value
            .push(target_cter.get_map_cell_index() as u32);
        let mut res;
        //如果是普通攻击，要算上减伤
        if skill_damege.is_none() {
            let from_cter = battle_data_ptr
                .as_mut()
                .unwrap()
                .get_battle_cter_mut(Some(from), true)?;
            let attack_damage = from_cter.calc_damage();
            let reduce_damage = self.calc_reduce_damage(from_cter.get_user_id(), target_cter);
            ep.effect_type = EffectType::AttackDamage as u32;
            res = attack_damage - reduce_damage;
            if res < 0 {
                res = 0;
            }
            let (gd_buff_id, gd_is_remove) = target_cter.trigger_attack_damge_gd();
            if gd_buff_id > 0 {
                let mut te_pt = TriggerEffectPt::new();
                te_pt.set_buff_id(gd_buff_id);
                target_pt.passiveEffect.push(te_pt);
                if gd_is_remove {
                    let lost_buff =
                        self.consume_buff(gd_buff_id, Some(target_cter.get_user_id()), None, false);
                    if let Some(lost_buff) = lost_buff {
                        target_pt.lost_buffs.push(lost_buff);
                    }
                }
                res = 0;
            } else {
                target_cter.status.is_attacked = true;
            }
        } else {
            res = skill_damege.unwrap();
        }
        ep.effect_value = res as u32;
        target_pt.effects.push(ep);
        let is_die = target_cter.add_hp(-res);

        //判断目标角色是否死亡
        if is_die {
            self.after_cter_died_trigger(target_user_id, is_last_one, false);
        }
        Ok(target_pt)
    }

    ///更新翻地图块队列
    pub fn exec_open_map_cell(
        &mut self,
        user_id: u32,
        index: usize,
        is_sub_residue_open_times: bool,
    ) {
        let cter = self.battle_cter.get_mut(&user_id).unwrap();
        //将翻的地图块放到翻开的队列
        cter.flow_data.open_map_cell_vec.push(index);

        //更新地图块打开人
        let map_cell = self.tile_map.map_cells.get_mut(index).unwrap();
        map_cell.open_user = user_id;

        //翻块次数-1
        if is_sub_residue_open_times {
            cter.flow_data.residue_open_times -= 1;
        }
        let res;
        let temp = crate::TEMPLATES
            .get_constant_temp_mgr_ref()
            .temps
            .get("reward_gold_open_cell");
        match temp {
            Some(temp) => {
                let value = u32::from_str(temp.value.as_str());
                match value {
                    Ok(value) => res = value,
                    Err(e) => {
                        error!("{:?}", e);
                        res = 1;
                    }
                }
            }
            None => {
                res = 1;
            }
        }
        //加金币
        cter.add_gold(res as i32);
    }

    ///处理地图块配对逻辑
    pub fn handler_map_cell_pair(&mut self, user_id: u32) -> bool {
        let battle_cter = self.battle_cter.get_mut(&user_id);
        if let None = battle_cter {
            error!("cter is not find!user_id:{}", user_id);
            return false;
        }
        let battle_cter = battle_cter.unwrap();
        let size = battle_cter.flow_data.open_map_cell_vec.len();
        //如果该turn第一次翻，或者已经配对了再翻，不用判断是否配对
        if size <= 1 {
            return false;
        }
        let index = *battle_cter
            .flow_data
            .open_map_cell_vec
            .get(size - 1)
            .unwrap();

        let map_cell = self.tile_map.map_cells.get_mut(index);
        if let None = map_cell {
            error!("map_cell is not find!map_cell_index:{}", index);
            return false;
        }
        let map_cell_ptr = map_cell.unwrap() as *mut MapCell;
        unsafe {
            let is_pair;

            //最近一次翻开的地图块
            let map_cell_mut = map_cell_ptr.as_mut().unwrap();
            let map_cell_id = map_cell_mut.id;

            //拿到上一个翻开的地图块
            let last_map_cell = *battle_cter
                .flow_data
                .open_map_cell_vec
                .get(size - 2)
                .unwrap();
            let last_map_cell = self.tile_map.map_cells.get_mut(last_map_cell).unwrap();
            let last_map_cell_id = last_map_cell.id;
            let last_map_cell_index = last_map_cell.index;

            //校验是否配对了
            if map_cell_id == last_map_cell_id {
                map_cell_mut.pair_index = Some(last_map_cell_index);
                last_map_cell.pair_index = Some(index);
                is_pair = true;
                battle_cter.status.is_pair = is_pair;
                let attack_state = battle_cter.get_attack_state();
                //状态改为可以进行攻击
                if attack_state != AttackState::Locked {
                    battle_cter.change_attack_able();
                } else {
                    warn!(
                        "could not set battle_cter'attack_state!attack_state:{:?},user_id:{}",
                        battle_cter.get_attack_state(),
                        battle_cter.get_user_id()
                    );
                }
                self.tile_map.un_pair_map.remove(&last_map_cell.index);
                self.tile_map.un_pair_map.remove(&map_cell_mut.index);
            } else {
                is_pair = false;
            }
            //配对了就封装
            if is_pair {
                info!(
                    "user:{} open map_cell pair! last_map_cell:{},now_map_cell:{}",
                    battle_cter.get_user_id(),
                    last_map_cell_index,
                    index
                );
            }
            is_pair
        }
    }

    ///发送战斗turn推送
    pub fn send_battle_turn_notice(&mut self) {
        let mut sbtn = S_BATTLE_TURN_NOTICE::new();
        sbtn.set_user_id(self.get_turn_user(None).unwrap());
        //角色身上的
        for cter in self.battle_cter.values() {
            let cter_pt = cter.convert_to_battle_cter_pt();
            sbtn.cters.push(cter_pt);
        }

        //地图块身上的
        for map_cell in self.tile_map.map_cells.iter() {
            let mut cbp = CellBuffPt::new();
            cbp.index = map_cell.index as u32;
            for buff in map_cell.buffs.values() {
                if map_cell.passive_buffs.contains(&buff.id) {
                    continue;
                }
                let mut buff_pt = BuffPt::new();
                buff_pt.buff_id = buff.id;
                buff_pt.trigger_timesed = buff.trigger_timesed as u32;
                buff_pt.keep_times = buff.keep_times as u32;
                cbp.buffs.push(buff_pt);
            }
            sbtn.cell_buffs.push(cbp);
        }

        let bytes = sbtn.write_to_bytes().unwrap();

        self.send_2_all_client(ClientCode::BattleTurnNotice, bytes);
    }

    ///获得战斗角色可变借用指针
    pub fn get_battle_cter_mut(
        &mut self,
        user_id: Option<u32>,
        is_alive: bool,
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
            anyhow::bail!("battle_cter not find!user_id:{}", _user_id)
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

    pub fn send_2_client(&mut self, cmd: ClientCode, user_id: u32, bytes: Vec<u8>) {
        let cter = self.get_battle_cter(Some(user_id), false);
        match cter {
            Ok(cter) => {
                if cter.is_robot() {
                    return;
                }
            }
            Err(_) => {}
        }
        let bytes = Packet::build_packet_bytes(cmd as u32, user_id, bytes, true, true);
        let res = self.get_sender_mut().send(bytes);
        if let Err(e) = res {
            error!("{:?}", e);
        }
    }

    pub fn send_2_all_client(&mut self, cmd: ClientCode, bytes: Vec<u8>) {
        for cter in self.battle_cter.values() {
            if cter.robot_data.is_some() {
                continue;
            }
            let user_id = cter.base_attr.user_id;
            let bytes_res =
                Packet::build_packet_bytes(cmd as u32, user_id, bytes.clone(), true, true);
            let res = self.tcp_sender.send(bytes_res);
            if let Err(e) = res {
                error!("{:?}", e);
            }
        }
    }

    ///获取目标数组
    pub fn get_target_array(&self, user_id: u32, skill_id: u32) -> anyhow::Result<Vec<usize>> {
        let res = TEMPLATES.get_skill_temp_mgr_ref().get_temp(&skill_id);
        if let Err(_) = res {
            anyhow::bail!("could not find skill temp of {}", skill_id)
        }
        let skill_temp = res.unwrap();
        let res = TargetType::try_from(skill_temp.target);
        if let Err(e) = res {
            anyhow::bail!("{:?}", e)
        }

        let mut v = Vec::new();
        let target_type = res.unwrap();
        match target_type {
            TargetType::MapCellOtherPlayer => {
                let element = skill_temp.par2 as u8;
                for map_cell in self.tile_map.map_cells.iter() {
                    let index = map_cell.index;
                    //排除自己和上面没人的地图块
                    if map_cell.user_id == user_id || map_cell.user_id == 0 {
                        continue;
                    }
                    let target_cter = self.battle_cter.get(&map_cell.user_id);
                    if let None = target_cter {
                        continue;
                    }

                    //匹配元素
                    if element > 0 && map_cell.element != element {
                        continue;
                    }
                    v.push(index);
                }
            }
            _ => {}
        }
        Ok(v)
    }

    ///获取并检查目标数组
    pub fn check_target_array(
        &self,
        user_id: u32,
        target_type: TargetType,
        target_array: &[u32],
    ) -> anyhow::Result<()> {
        match target_type {
            //无效目标
            TargetType::None => {
                anyhow::bail!("this target_type is invaild!target_type:{:?}", target_type)
            }
            //任意玩家
            TargetType::AnyPlayer => {
                let mut v = Vec::new();
                for &index in target_array {
                    let cter = self.get_battle_cter_by_map_cell_index(index as usize)?;
                    v.push(cter.get_user_id());
                    break;
                }
                self.check_user_target(&v[..], None)?; //不包括自己的其他玩家
            } //玩家自己
            TargetType::PlayerSelf => {
                if target_array.len() > 1 {
                    anyhow::bail!("this target_type is invaild!target_type:{:?}", target_type)
                }
                for &index in target_array {
                    let cter = self.get_battle_cter_by_map_cell_index(index as usize)?;
                    if cter.get_user_id() != user_id {
                        anyhow::bail!("this target_type is invaild!target_type:{:?}", target_type)
                    }
                }
            } //玩家自己
            //全图玩家
            TargetType::AllPlayer => {
                let mut v = Vec::new();
                for &index in target_array {
                    let cter = self.get_battle_cter_by_map_cell_index(index as usize)?;
                    v.push(cter.get_user_id());
                }
                self.check_user_target(&v[..], None)?; //不包括自己的其他玩家
            }
            TargetType::OtherAllPlayer => {
                let mut v = Vec::new();
                for &index in target_array {
                    let cter = self.get_battle_cter_by_map_cell_index(index as usize)?;
                    v.push(cter.get_user_id());
                }
                //除自己所有玩家
                self.check_user_target(&v[..], Some(user_id))?
            } //除自己外任意玩家
            TargetType::OtherAnyPlayer => {
                let mut v = Vec::new();
                for &index in target_array {
                    let cter = self.get_battle_cter_by_map_cell_index(index as usize)?;
                    v.push(cter.get_user_id());
                    break;
                }
                //除自己所有玩家
                self.check_user_target(&v[..], Some(user_id))?
            }
            TargetType::SelfScopeOthers => {
                let mut v = Vec::new();
                for &index in target_array {
                    let cter = self.get_battle_cter_by_map_cell_index(index as usize)?;
                    v.push(cter.get_user_id());
                    break;
                }
                //除自己所有玩家
                self.check_user_target(&v[..], Some(user_id))?
            }
            //地图块
            TargetType::MapCell => {
                //校验地图块下标有效性
                for &index in target_array {
                    let index = index as usize;
                    self.check_choice_index(index, false, false, false, false, false)?;
                }
            }
            //未翻开的地图块
            TargetType::UnOpenMapCell => {
                for &index in target_array {
                    self.check_choice_index(index as usize, true, true, true, false, false)?;
                }
            } //未配对的地图块
            TargetType::UnPairMapCell => {
                for &index in target_array {
                    self.check_choice_index(index as usize, false, true, true, true, false)?;
                }
            } //空的地图块
            TargetType::NullMapCell => {
                for &index in target_array {
                    self.check_choice_index(index as usize, false, true, true, false, true)?;
                }
            } //空的地图块，上面没人
            TargetType::UnPairNullMapCell => {
                for &index in target_array {
                    let index = index as usize;
                    self.check_choice_index(index, false, false, false, false, true)?;
                }
            }
            TargetType::OpenedMapCell => {
                for &index in target_array {
                    let index = index as usize;
                    self.check_choice_index(index, true, true, true, false, false)?;
                }
            }
            //其他目标类型
            TargetType::MapCellOtherPlayer => {}
            //未翻开，且未锁定
            TargetType::UnOpenMapCellAndUnLock => {
                for &index in target_array {
                    let index = index as usize;
                    self.check_choice_index(index, true, false, true, true, false)?;
                }
            }
            //未锁定空地图块
            TargetType::UnLockNullMapCell => {
                for &index in target_array {
                    let index = index as usize;
                    self.check_choice_index(index, false, false, true, true, true)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    ///检测目标玩家
    pub fn check_user_target(&self, vec: &[u32], check_self_id: Option<u32>) -> anyhow::Result<()> {
        for member_id in vec.iter() {
            let member_id = *member_id;
            //校验有没有
            if !self.battle_cter.contains_key(&member_id) {
                anyhow::bail!("battle_cter is not find!user_id:{}", member_id)
            }
            //校验是不是自己
            if check_self_id.is_some() && member_id == check_self_id.unwrap() {
                anyhow::bail!("target_user_id==self!target_user_id:{}", member_id)
            }
        }
        Ok(())
    }

    //检测地图块是否选择
    pub fn check_choice_index(
        &self,
        index: usize,
        is_check_open: bool,
        is_check_pair: bool,
        is_check_world: bool,
        is_check_locked: bool,
        is_check_has_user: bool,
    ) -> anyhow::Result<()> {
        let res = self.tile_map.map_cells.get(index);
        if res.is_none() {
            anyhow::bail!("this map_cell is not find!index:{}", index)
        }
        let map_cell = res.unwrap();

        if map_cell.id < MapCellType::Valid.into_u32() {
            anyhow::bail!(
                "this is map_cell can not be choice!index:{}",
                map_cell.index
            )
        }

        let map_cell = res.unwrap();
        if is_check_open && map_cell.open_user > 0 {
            anyhow::bail!("this map_cell already opened!index:{}", map_cell.index)
        } else if is_check_open && map_cell.pair_index.is_some() {
            anyhow::bail!("this map_cell already pair!index:{}", map_cell.index)
        }
        if is_check_pair && map_cell.pair_index.is_some() {
            anyhow::bail!("this map_cell already pair!index:{}", map_cell.index)
        }
        if is_check_world && map_cell.is_world {
            anyhow::bail!("world_map_cell can not be choice!index:{}", map_cell.index)
        }
        if is_check_locked && map_cell.check_is_locked() {
            anyhow::bail!("this map_cell is locked!index:{}", map_cell.index)
        }
        if is_check_has_user && map_cell.user_id > 0 {
            anyhow::bail!("this map_cell has user!index:{}", map_cell.index)
        }
        Ok(())
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
        task.cmd = TaskCmd::BattleTurnTime;

        let mut map = serde_json::Map::new();
        map.insert("user_id".to_owned(), serde_json::Value::from(*user_id));
        task.data = serde_json::Value::from(map);
        let res = self.task_sender.send(task);
        if res.is_err() {
            error!("{:?}", res.err().unwrap());
        }
    }

    ///构建targetpt
    pub fn build_target_pt(
        &self,
        from_user: Option<u32>,
        target_user: u32,
        effect_type: EffectType,
        effect_value: u32,
        buff_id: Option<u32>,
    ) -> anyhow::Result<TargetPt> {
        let target_cter = self.get_battle_cter(Some(target_user), true)?;
        let mut target_pt = TargetPt::new();
        target_pt
            .target_value
            .push(target_cter.get_map_cell_index() as u32);
        if from_user.is_some() && from_user.unwrap() == target_user && buff_id.is_some() {
            let mut tep = TriggerEffectPt::new();
            tep.set_field_type(effect_type.into_u32());
            tep.set_value(effect_value);
            tep.buff_id = buff_id.unwrap();
            target_pt.passiveEffect.push(tep);
        } else {
            let mut ep = EffectPt::new();
            ep.effect_type = effect_type.into_u32();
            ep.effect_value = effect_value;
            target_pt.effects.push(ep);
        }
        Ok(target_pt)
    }

    ///计算范围,返回一个元组类型，前面一个是范围，后面一个是范围内的合法玩家
    /// 当targets和scope_temp为None时,以⭕️为校验范围有没有人
    /// 当targets为None,scope_temp为Some则校验scope_temp范围内有没有人
    /// 当targets和scope_temp都不为None时，校验targets是否在scope_temp范围内
    pub fn cal_scope(
        &self,
        user_id: u32,
        center_index: isize,
        target_type: TargetType,
        targets: Option<Vec<u32>>,
        mut scope_temp: Option<&SkillScopeTemp>,
    ) -> (Vec<usize>, Vec<u32>) {
        let mut v_u = Vec::new();
        let mut v = Vec::new();
        let center_map_cell = self.tile_map.map_cells.get(center_index as usize).unwrap();
        if targets.is_none() && scope_temp.is_none() {
            let res = TEMPLATES
                .get_skill_scope_temp_mgr_ref()
                .get_temp(&TRIGGER_SCOPE_NEAR_TEMP_ID);
            if let Err(e) = res {
                warn!("{:?}", e);
                return (Vec::new(), Vec::new());
            }
            let res = res.unwrap();
            scope_temp = Some(res);
        }
        //没有目标，只有范围
        if targets.is_none() && scope_temp.is_some() {
            let scope_temp = scope_temp.unwrap();

            for direction_temp2d in scope_temp.scope2d.iter() {
                for coord_temp in direction_temp2d.direction2d.iter() {
                    let x = center_map_cell.x + coord_temp.x;
                    let y = center_map_cell.y + coord_temp.y;
                    let map_cell_index = self.tile_map.coord_map.get(&(x, y));
                    if let None = map_cell_index {
                        continue;
                    }
                    let map_cell_index = map_cell_index.unwrap();
                    let map_cell = self.tile_map.map_cells.get(*map_cell_index);
                    if map_cell.is_none() {
                        continue;
                    }
                    v.push(*map_cell_index);
                    let map_cell = map_cell.unwrap();
                    if map_cell.user_id <= 0 {
                        continue;
                    }
                    //如果目标不能是自己，就跳过
                    if (target_type == TargetType::OtherAllPlayer
                        || target_type == TargetType::SelfScopeOthers
                        || target_type == TargetType::SelfScopeAnyOthers
                        || target_type == TargetType::OtherAnyPlayer)
                        && map_cell.user_id == user_id
                    {
                        continue;
                    }
                    let other_user = map_cell.user_id;
                    //如果玩家id大于0
                    if other_user == 0 {
                        continue;
                    }

                    let cter = self.get_battle_cter(Some(other_user), true);
                    if let Err(e) = cter {
                        warn!("{:?}", e);
                        continue;
                    }
                    v_u.push(other_user);
                }
            }
        } else {
            //两者都有
            let targets = targets.unwrap();
            let scope_temp = scope_temp.unwrap();
            //否则校验选中的区域
            for dir in scope_temp.scope2d.iter() {
                for coord_temp in dir.direction2d.iter() {
                    let x = center_map_cell.x + coord_temp.x;
                    let y = center_map_cell.y + coord_temp.y;
                    let map_cell_index = self.tile_map.coord_map.get(&(x, y));
                    if let None = map_cell_index {
                        warn!("there is no map_cell for {:?}", (x, y));
                        continue;
                    }
                    let map_cell_index = map_cell_index.unwrap();
                    let map_cell = self.tile_map.map_cells.get(*map_cell_index);
                    if let None = map_cell {
                        continue;
                    }
                    v.push(*map_cell_index);
                    let map_cell = map_cell.unwrap();
                    for index in targets.iter() {
                        if map_cell.index as u32 != *index {
                            continue;
                        }
                        let other_user = map_cell.user_id;
                        //如果目标不能是自己，就跳过
                        if (target_type == TargetType::OtherAllPlayer
                            || target_type == TargetType::SelfScopeOthers
                            || target_type == TargetType::SelfScopeAnyOthers
                            || target_type == TargetType::OtherAnyPlayer)
                            && map_cell.user_id == user_id
                        {
                            continue;
                        }
                        //如果玩家id大于0
                        if other_user == 0 {
                            continue;
                        }
                        let cter = self.get_battle_cter(Some(other_user), true);
                        if let Err(e) = cter {
                            warn!("{:?}", e);
                            continue;
                        }
                        let cter = cter.unwrap();
                        if v_u.contains(&cter.get_user_id()) {
                            continue;
                        }
                        v_u.push(cter.get_user_id());
                    }
                }
            }
        }
        (v, v_u)
    }

    ///校验技能条件
    pub fn check_skill_judge(
        &self,
        user_id: u32,
        skill_judge: u32,
        skill_id: Option<u32>,
        _: Option<Vec<u32>>,
    ) -> anyhow::Result<()> {
        if skill_judge == 0 {
            return Ok(());
        }
        let judge_temp = TEMPLATES
            .get_skill_judge_temp_mgr_ref()
            .get_temp(&skill_judge)?;
        let target_type = TargetType::try_from(judge_temp.target);
        if let Err(e) = target_type {
            anyhow::bail!("{:?}", e)
        }
        let cter = self.get_battle_cter(Some(user_id), true).unwrap();
        let target_type = target_type.unwrap();

        match target_type {
            TargetType::PlayerSelf => {
                if HP_LIMIT_GT == judge_temp.id && cter.base_attr.hp <= judge_temp.par1 as i16 {
                    anyhow::bail!(
                        "HP_LIMIT_GT!hp of cter <= {}!skill_judge_id:{}",
                        judge_temp.par1,
                        judge_temp.id
                    )
                } else if LIMIT_TURN_TIMES == judge_temp.id
                    && cter
                        .flow_data
                        .turn_limit_skills
                        .contains(&skill_id.unwrap())
                {
                    anyhow::bail!(
                        "this turn already used this skill!cter_id:{},skill_id:{},skill_judge_id:{}",
                        cter.get_cter_id(),
                        skill_id.unwrap(),
                        skill_judge,
                    )
                } else if LIMIT_ROUND_TIMES == judge_temp.id
                    && cter
                        .flow_data
                        .round_limit_skills
                        .contains(&skill_id.unwrap())
                {
                    anyhow::bail!(
                        "this round already used this skill!cter_id:{},skill_id:{},skill_judge_id:{}",
                        cter.get_cter_id(),
                        skill_id.unwrap(),
                        skill_judge,
                    )
                }
            }
            _ => {}
        }
        Ok(())
    }
}
