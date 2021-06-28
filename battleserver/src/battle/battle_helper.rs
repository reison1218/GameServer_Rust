use crate::battle::battle::BattleData;
use crate::battle::battle_enum::skill_judge_type::{
    HP_LIMIT_GT, LIMIT_ROUND_TIMES, LIMIT_TURN_TIMES,
};
use crate::battle::battle_enum::{AttackState, EffectType, TargetType, TRIGGER_SCOPE_NEAR_TEMP_ID};
use crate::battle::battle_trigger::TriggerEvent;
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

use super::battle_enum::buff_type::TRAPS;
use super::battle_enum::{ActionType, BattlePlayerState};
use super::mission::{trigger_mission, MissionTriggerType};
use super::{battle_enum::skill_judge_type::PAIR_LIMIT, battle_player::BattlePlayer};
use crate::JsonValue;

impl BattleData {
    ///检测地图刷新
    pub fn check_refresh_map(&mut self) -> bool {
        let allive_count = self
            .battle_player
            .values()
            .filter(|x| x.status.battle_state == BattlePlayerState::Normal)
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
        let battle_player = self.battle_player.get(&user_id);
        if let None = battle_player {
            return;
        }
        let battle_player = battle_player.unwrap();
        for index in battle_player.flow_data.open_map_cell_vec_history.iter() {
            let map_cell = self.tile_map.map_cells.get_mut(*index);
            if let Some(map_cell) = map_cell {
                if map_cell.pair_index.is_none() {
                    map_cell.open_user = 0;
                }
            }
        }
    }

    pub fn choice_index_next_turn(&mut self) {
        let battle_data_ptr = self as *mut BattleData;
        self.next_turn_index += 1;
        let index = self.next_turn_index;
        if index >= MEMBER_MAX as usize {
            self.next_turn_index = 0;
        }
        let user_id = self.get_turn_user(None);
        if let Ok(user_id) = user_id {
            if user_id == 0 {
                self.choice_index_next_turn();
                return;
            }

            let battle_player_res = self.get_battle_player_mut(Some(user_id), false);
            match battle_player_res {
                Ok(battle_player) => {
                    if battle_player.is_died() {
                        self.choice_index_next_turn();
                        return;
                    }
                    if battle_player.robot_data.is_some() {
                        battle_player.robot_start_action(battle_data_ptr);
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

    ///下一个
    pub fn add_next_turn(&mut self) {
        let battle_data_ptr = self as *mut BattleData;
        self.next_turn_index += 1;
        self.turn += 1;
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
                self.add_next_turn();
                return;
            }

            let battle_player_res = self.get_battle_player_mut(Some(user_id), false);
            match battle_player_res {
                Ok(battle_player) => {
                    if battle_player.is_died() {
                        self.add_next_turn();
                        return;
                    }
                    if battle_player.robot_data.is_some() {
                        battle_player.robot_start_action(battle_data_ptr);
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
        target_index: usize,
        au: &mut ActionUnitPt,
    ) -> anyhow::Result<(bool, Vec<(u32, ActionUnitPt)>)> {
        let battle_players = &mut self.battle_player as *mut HashMap<u32, BattlePlayer>;
        let battle_player = battle_players.as_mut().unwrap().get_mut(&user_id).unwrap();
        let source_battle_player_index = battle_player.get_map_cell_index();
        let tile_map_ptr = self.tile_map.borrow_mut() as *mut TileMap;
        let target_map_cell = tile_map_ptr
            .as_mut()
            .unwrap()
            .map_cells
            .get_mut(target_index)
            .unwrap();

        let mut is_change_index_both = false;
        let title_map_mut = tile_map_ptr.as_mut().unwrap();
        //判断改地图块上面有没有角色，有的话将目标位置的玩家挪到操作玩家的位置上
        if target_map_cell.user_id > 0 {
            let target_user = target_map_cell.user_id;
            //先判断目标位置的角色是否有不动泰山被动技能
            self.before_moved_trigger(user_id, target_user)?;
            //如果没有，则改变目标玩家的位置
            let target_player = self.get_battle_player_mut(Some(target_user), true).unwrap();
            target_player.cter.move_index(source_battle_player_index);
            let source_map_cell = title_map_mut
                .map_cells
                .get_mut(source_battle_player_index)
                .unwrap();
            source_map_cell.user_id = target_player.get_user_id();
            is_change_index_both = true;
        } else {
            //重制之前地图块上的玩家id
            let source_map_cell = title_map_mut
                .map_cells
                .get_mut(source_battle_player_index)
                .unwrap();
            source_map_cell.user_id = 0;
        }
        //改变角色位置
        battle_player.cter.move_index(target_index);
        target_map_cell.user_id = user_id;

        let index = target_index as isize;
        //移动位置后触发事件
        let res = self.after_move_trigger(battle_player, index, is_change_index_both);
        //如果角色没死，就把翻的地图块id传给客户端
        if !res.0 {
            au.action_value.push(target_map_cell.id);
        }
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
        let mut player_res: Option<&mut BattlePlayer> = None;
        let mut map_cell_res: Option<&mut MapCell> = None;
        let mut lost_buff = None;
        let battle_players = self.battle_player.borrow_mut() as *mut HashMap<u32, BattlePlayer>;
        if user_id.is_some() {
            let user_id = user_id.unwrap();
            let battle_player = self.get_battle_player_mut(Some(user_id), true);
            if let Err(_) = battle_player {
                return lost_buff;
            }
            let battle_player = battle_player.unwrap();
            let buff = battle_player.cter.battle_buffs.get_buff_mut(buff_id);
            if buff.is_none() {
                return lost_buff;
            }
            player_res = Some(battle_player);
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
        if player_res.is_some() {
            buff = player_res
                .as_mut()
                .unwrap()
                .cter
                .battle_buffs
                .get_buff_mut(buff_id);
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
        let cfg_keep_time = buff.buff_temp.keep_time;
        let cfg_trigger_time = buff.buff_temp.trigger_times;
        //判断是否减去keep_times
        if cfg_keep_time > 0 && is_turn_index {
            if buff.turn_index.is_some() && buff.turn_index.unwrap() == next_turn_index {
                buff.sub_keep_times();
            }
        } else if cfg_trigger_time > 0 && !is_turn_index {
            //判断是否减去处罚次数
            buff.sub_trigger_timesed()
        }

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
                let from_player = battle_players.as_mut().unwrap().get_mut(&from_user);
                if let Some(from_player) = from_player {
                    if let Some(from_skill) = buff.from_skill {
                        let skill = from_player.cter.skills.get_mut(&from_skill);
                        if let Some(skill) = skill {
                            skill.is_active = false;
                        }
                    }
                }
            }
            //如果是玩家身上的
            if let Some(player) = player_res {
                player.cter.remove_buff(buff_id);
                lost_buff = Some(buff_id);
                let user_id = player.get_user_id();
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
        target_user: u32,
        hp: i16,
        buff_id: Option<u32>,
    ) -> anyhow::Result<TargetPt> {
        let battle_player = self.get_battle_player_mut(Some(target_user), true)?;

        if battle_player.is_died() {
            anyhow::bail!(
                "this battle_player is died! user_id:{},cter_id:{}",
                target_user,
                battle_player.get_cter_id()
            )
        }
        battle_player.add_hp(hp);

        let target_pt =
            self.build_target_pt(from_user, target_user, EffectType::Cure, hp as u32, buff_id)?;
        Ok(target_pt)
    }

    ///计算减伤
    pub fn calc_reduce_damage(&self, from_user: u32, target_player: &mut BattlePlayer) -> i16 {
        let target_user = target_player.get_user_id();
        let scope_temp = TEMPLATES
            .skill_scope_temp_mgr()
            .get_temp(&TRIGGER_SCOPE_NEAR_TEMP_ID);
        if let Err(_) = scope_temp {
            return target_player.cter.base_attr.defence as i16;
        }
        let scope_temp = scope_temp.unwrap();
        let (_, user_v) = self.cal_scope(
            target_user,
            target_player.get_map_cell_index() as isize,
            TargetType::None,
            None,
            Some(scope_temp),
        );
        let res = user_v.contains(&from_user);
        target_player.cter.calc_reduce_damage(res)
    }

    pub fn get_alive_player_num(&self) -> usize {
        let alive_count = self
            .battle_player
            .values()
            .filter(|x| x.status.battle_state == BattlePlayerState::Normal)
            .count();
        alive_count
    }

    pub fn new_target_pt(&self, user_id: u32) -> anyhow::Result<TargetPt> {
        let battle_player = self.get_battle_player(Some(user_id), false)?;
        let mut target_pt = TargetPt::new();
        target_pt
            .target_value
            .push(battle_player.get_map_cell_index() as u32);
        Ok(target_pt)
    }

    ///扣血
    pub unsafe fn deduct_hp(
        &mut self,
        from: u32,
        target: u32,
        skill_damege: Option<i16>,
        target_pt: &mut TargetPt,
        mut is_last_one: bool,
    ) -> anyhow::Result<u32> {
        let battle_data_ptr = self as *mut BattleData;

        let mut ep = EffectPt::new();
        ep.effect_type = EffectType::SkillDamage as u32;

        let target_player = battle_data_ptr
            .as_mut()
            .unwrap()
            .get_battle_player_mut(Some(target), true);
        if let Err(e) = target_player {
            let str = format!("{:?}", e);
            error!("{:?}", str);
            anyhow::bail!("{:?}", str)
        }
        let target_player = target_player.unwrap();
        let target_user_id = target_player.user_id;

        let mut res;
        //如果是普通攻击，要算上减伤
        if skill_damege.is_none() {
            let from_player = battle_data_ptr
                .as_mut()
                .unwrap()
                .get_battle_player_mut(Some(from), true)?;
            let attack_damage = from_player.cter.calc_damage();
            let reduce_damage = self.calc_reduce_damage(from_player.get_user_id(), target_player);
            ep.effect_type = EffectType::AttackDamage as u32;
            res = attack_damage - reduce_damage;
            if res < 0 {
                res = 0;
            }
            let (gd_buff_id, gd_is_remove) = target_player.cter.trigger_attack_damge_gd();
            if gd_buff_id > 0 {
                let mut te_pt = TriggerEffectPt::new();
                te_pt.set_buff_id(gd_buff_id);
                target_pt.passiveEffect.push(te_pt);
                if gd_is_remove {
                    let lost_buff = self.consume_buff(
                        gd_buff_id,
                        Some(target_player.get_user_id()),
                        None,
                        false,
                    );
                    if let Some(lost_buff) = lost_buff {
                        target_pt.lost_buffs.push(lost_buff);
                    }
                }
                res = 0;
            } else {
                target_player.status.is_attacked = true;
            }
        } else {
            res = skill_damege.unwrap();
        }
        ep.effect_value = res as u32;
        target_pt.effects.push(ep);
        let is_die = target_player.add_hp(-res);

        //判断目标角色是否死亡
        if is_die {
            let player_count = self.get_alive_player_num();
            if player_count == 1 {
                is_last_one = true;
            }
            self.after_cter_died_trigger(from, target_user_id, is_last_one, false);
        }
        Ok(res as u32)
    }

    ///更新翻地图块队列
    pub fn exec_open_map_cell(&mut self, user_id: u32, index: usize) {
        let battle_player = self.battle_player.get_mut(&user_id).unwrap();
        //将翻的地图块放到翻开的队列
        battle_player.add_open_map_cell(index);

        //更新地图块打开人
        let map_cell = self.tile_map.map_cells.get_mut(index).unwrap();
        map_cell.open_user = user_id;
        let element = map_cell.element as u32;
        let res;
        let temp = crate::TEMPLATES
            .constant_temp_mgr()
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
        battle_player.add_gold(res as i32);
        //触发任务
        trigger_mission(
            self,
            user_id,
            vec![
                (MissionTriggerType::OpenCell, 1),
                (MissionTriggerType::GetGold, res as u16),
            ],
            (element, 0),
        );
    }

    ///处理地图块配对逻辑
    pub fn handler_map_cell_pair(&mut self, user_id: u32, index: usize) -> bool {
        let battle_player = self.battle_player.get_mut(&user_id);
        if let None = battle_player {
            error!("battle_player is not find!user_id:{}", user_id);
            return false;
        }
        let battle_player = battle_player.unwrap();
        let map_cell = self.tile_map.map_cells.get_mut(index);
        if let None = map_cell {
            error!("map_cell is not find!map_cell_index:{}", index);
            return false;
        }
        let map_cell_ptr = map_cell.unwrap() as *mut MapCell;
        unsafe {
            let mut is_pair = false;
            let mut rm_index = 0;
            let map_cell_mut = map_cell_ptr.as_mut().unwrap();
            let map_cell_id = map_cell_mut.id;

            for (vec_index, &open_index) in
                battle_player.flow_data.open_map_cell_vec.iter().enumerate()
            {
                //处理配对逻辑
                let res = self.tile_map.map_cells.get_mut(open_index);
                if res.is_none() {
                    continue;
                }
                let match_map_cell = res.unwrap();

                //不匹配就跳过
                if match_map_cell.id != map_cell_id {
                    continue;
                }
                let match_map_cell_index = match_map_cell.index;

                map_cell_mut.pair_index = Some(match_map_cell_index);
                match_map_cell.pair_index = Some(index);
                is_pair = true;
                battle_player.status.is_pair = is_pair;
                let attack_state = battle_player.get_attack_state();
                //状态改为可以进行攻击
                if attack_state != AttackState::Locked {
                    battle_player.change_attack_able();
                } else {
                    warn!(
                        "could not set battle_player'attack_state!attack_state:{:?},user_id:{}",
                        battle_player.get_attack_state(),
                        battle_player.get_user_id()
                    );
                }
                self.tile_map.un_pair_map.remove(&match_map_cell.index);
                self.tile_map.un_pair_map.remove(&map_cell_mut.index);
                info!(
                    "user:{} open map_cell pair! last_map_cell:{},now_map_cell:{}",
                    battle_player.get_user_id(),
                    match_map_cell_index,
                    index
                );
                is_pair = true;
                rm_index = vec_index;
                break;
            }
            if !is_pair {
                battle_player.flow_data.open_map_cell_vec.push(index);
            } else {
                battle_player.flow_data.open_map_cell_vec.remove(rm_index);
                battle_player.pair_reward_movement_points();
            }
            is_pair
        }
    }

    ///发送战斗turn推送
    pub fn send_battle_turn_notice(&mut self) {
        //最终推送的proto
        let mut push_map = HashMap::new();
        //地图块上的buff proto,因为如果是陷阱，只有部分人才能看到，所以要单独封装
        let mut cell_buff_map = HashMap::new();

        //初始化
        for battle_player in self.battle_player.values() {
            let user_id = battle_player.get_user_id();
            let mut sbtn = S_BATTLE_TURN_NOTICE::new();
            sbtn.set_user_id(self.get_turn_user(None).unwrap());
            push_map.insert(user_id, sbtn);
            cell_buff_map.insert(user_id, HashMap::new());
        }

        //角色身上的
        for sbtn in push_map.values_mut() {
            for battle_player in self.battle_player.values() {
                let cter_pt = battle_player.convert_to_battle_cter_pt();
                sbtn.cters.push(cter_pt);
            }
        }

        //地图块身上的
        for map_cell in self.tile_map.map_cells.iter() {
            let index = map_cell.index as u32;
            let mut cbp = CellBuffPt::new();
            cbp.index = index;
            for res in cell_buff_map.values_mut() {
                res.insert(index, cbp.clone());
            }
            for buff in map_cell.buffs.values() {
                let buff_id = buff.get_id();
                let buff_function_id = buff.function_id;
                if map_cell.passive_buffs.contains(&buff_id) {
                    continue;
                }
                let mut buff_pt = BuffPt::new();
                buff_pt.buff_id = buff_id;
                buff_pt.trigger_timesed = buff.trigger_timesed as u32;
                buff_pt.keep_times = buff.keep_times as u32;

                if TRAPS.contains(&buff_function_id) {
                    for &view_user in buff.trap_view_users.iter() {
                        let res = cell_buff_map.get_mut(&view_user).unwrap();
                        let cbp = res.get_mut(&index).unwrap();
                        cbp.buffs.push(buff_pt.clone());
                    }
                } else {
                    for res in cell_buff_map.values_mut() {
                        let cbp = res.get_mut(&index).unwrap();
                        cbp.buffs.push(buff_pt.clone())
                    }
                }
            }
        }

        for (user_id, mut sbtn) in push_map {
            let res = cell_buff_map.remove(&user_id);
            if let Some(res) = res {
                for (_, cpt) in res {
                    if !cpt.buffs.is_empty() {
                        sbtn.cell_buffs.push(cpt);
                    }
                }
            }

            let bytes = sbtn.write_to_bytes().unwrap();
            self.send_2_client(ClientCode::BattleTurnNotice, user_id, bytes);
        }
    }

    ///获得战斗角色可变借用指针
    pub fn get_battle_player_mut(
        &mut self,
        user_id: Option<u32>,
        is_alive: bool,
    ) -> anyhow::Result<&mut BattlePlayer> {
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
        let battle_player = self.battle_player.get_mut(&_user_id);
        if let None = battle_player {
            anyhow::bail!("battle_player not find!user_id:{}", _user_id)
        }
        let battle_player = battle_player.unwrap();
        if is_alive && battle_player.is_died() {
            anyhow::bail!(
                "this battle_player is already died!user_id:{},cter_id:{}",
                _user_id,
                battle_player.get_cter_id()
            )
        }
        Ok(battle_player)
    }

    pub fn send_2_client(&mut self, cmd: ClientCode, user_id: u32, bytes: Vec<u8>) {
        let battle_player = self.get_battle_player(Some(user_id), false);
        match battle_player {
            Ok(battle_player) => {
                if battle_player.is_robot() {
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
        let cmd = cmd.into_u32();
        for battle_player in self.battle_player.values() {
            if battle_player.robot_data.is_some() {
                continue;
            }
            let user_id = battle_player.user_id;

            let bytes_res = Packet::build_packet_bytes(cmd, user_id, bytes.clone(), true, true);
            let res = self.tcp_sender.send(bytes_res);
            if let Err(e) = res {
                error!("{:?}", e);
            }
        }
    }

    ///获取目标数组
    pub fn get_target_array(&self, user_id: u32, skill_id: u32) -> anyhow::Result<Vec<usize>> {
        let res = TEMPLATES.skill_temp_mgr().get_temp(&skill_id);
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
                    //必须已经翻开
                    if map_cell.open_user == 0 {
                        continue;
                    }
                    //排除自己和上面没人的地图块
                    if map_cell.user_id == user_id || map_cell.user_id == 0 {
                        continue;
                    }
                    let target_player = self.battle_player.get(&map_cell.user_id);
                    if let None = target_player {
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
        //如果为空，则不校验
        if target_array.is_empty() {
            return Ok(());
        }
        let center_index = *target_array.get(0).unwrap() as usize;
        //先判断中心是否是
        self.check_choice_index(center_index, false, false, false, true, false, false)?;
        //校验其他目标类型
        match target_type {
            //无效目标
            TargetType::None => {
                anyhow::bail!("this target_type is invaild!target_type:{:?}", target_type)
            }
            //任意玩家
            TargetType::AnyPlayer => {
                let mut v = Vec::new();
                for &index in target_array {
                    let battle_player = self.get_battle_player_by_map_cell_index(index as usize)?;
                    v.push(battle_player.get_user_id());
                    break;
                }
                self.check_user_target(&v[..], None)?; //不包括自己的其他玩家
            } //玩家自己
            TargetType::PlayerSelf => {
                if target_array.len() > 1 {
                    anyhow::bail!("this target_type is invaild!target_type:{:?}", target_type)
                }
                for &index in target_array {
                    let battle_player = self.get_battle_player_by_map_cell_index(index as usize)?;
                    if battle_player.get_user_id() != user_id {
                        anyhow::bail!("this target_type is invaild!target_type:{:?}", target_type)
                    }
                }
            } //玩家自己
            //全图玩家
            TargetType::AllPlayer => {
                let mut v = Vec::new();
                for &index in target_array {
                    let battle_player = self.get_battle_player_by_map_cell_index(index as usize)?;
                    v.push(battle_player.get_user_id());
                }
                self.check_user_target(&v[..], None)?; //不包括自己的其他玩家
            }
            TargetType::OtherAllPlayer => {
                let mut v = Vec::new();
                for &index in target_array {
                    let battle_player = self.get_battle_player_by_map_cell_index(index as usize)?;
                    v.push(battle_player.get_user_id());
                }
                //除自己所有玩家
                self.check_user_target(&v[..], Some(user_id))?
            } //除自己外任意玩家
            TargetType::OtherAnyPlayer => {
                let mut v = Vec::new();
                for &index in target_array {
                    let battle_player = self.get_battle_player_by_map_cell_index(index as usize)?;
                    v.push(battle_player.get_user_id());
                    break;
                }
                //除自己所有玩家
                self.check_user_target(&v[..], Some(user_id))?
            }
            TargetType::SelfScopeOthers => {
                let mut v = Vec::new();
                for &index in target_array {
                    let battle_player = self.get_battle_player_by_map_cell_index(index as usize)?;
                    v.push(battle_player.get_user_id());
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
                    self.check_choice_index(index, false, false, false, false, false, false)?;
                }
            }
            //未翻开的地图块
            TargetType::UnOpenMapCell => {
                for &index in target_array {
                    self.check_choice_index(index as usize, false, true, true, true, false, false)?;
                }
            } //未配对的地图块
            TargetType::UnPairMapCell => {
                for &index in target_array {
                    self.check_choice_index(index as usize, false, false, true, true, true, false)?;
                }
            } //空的地图块
            TargetType::NullMapCell => {
                for &index in target_array {
                    self.check_choice_index(
                        index as usize,
                        false,
                        false,
                        false,
                        true,
                        false,
                        true,
                    )?;
                }
            } //空的地图块，上面没人
            TargetType::UnPairNullMapCell => {
                for &index in target_array {
                    let index = index as usize;
                    self.check_choice_index(index, false, false, false, false, false, true)?;
                }
            }
            TargetType::OpenedMapCell => {
                for &index in target_array {
                    let index = index as usize;
                    self.check_choice_index(index, true, false, false, true, false, false)?;
                }
            }
            //其他目标类型
            TargetType::MapCellOtherPlayer => {}
            //未翻开，且未锁定
            TargetType::UnOpenMapCellAndUnLock => {
                for &index in target_array {
                    let index = index as usize;
                    self.check_choice_index(index, false, true, false, true, true, false)?;
                }
            }
            //未锁定空地图块
            TargetType::UnLockNullMapCell => {
                for &index in target_array {
                    let index = index as usize;
                    self.check_choice_index(index, false, false, false, true, true, true)?;
                }
            }
            //未翻开的空地图块
            TargetType::UnOpenNullMapCell => {
                for &index in target_array {
                    let index = index as usize;
                    self.check_choice_index(index, false, true, true, true, false, true)?;
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
            if !self.battle_player.contains_key(&member_id) {
                anyhow::bail!("battle_player is not find!user_id:{}", member_id)
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
        is_check_close: bool,
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
        let res = match map_cell.cell_type {
            MapCellType::Valid => true,
            MapCellType::WorldCell => true,
            MapCellType::MarketCell => true,
            _ => false,
        };

        if !res {
            anyhow::bail!(
                "this is map_cell can not be choice!index:{},cell_id:{},cell_type:{:?}",
                map_cell.index,
                map_cell.id,
                map_cell.cell_type
            )
        }
        if is_check_close && map_cell.open_user == 0 {
            anyhow::bail!("this map_cell already closed!index:{}", map_cell.index)
        }
        if is_check_open && map_cell.open_user > 0 {
            anyhow::bail!("this map_cell already opened!index:{}", map_cell.index)
        } else if is_check_open && map_cell.pair_index.is_some() {
            anyhow::bail!("this map_cell already pair!index:{}", map_cell.index)
        }
        if is_check_pair && map_cell.pair_index.is_some() {
            anyhow::bail!("this map_cell already pair!index:{}", map_cell.index)
        }
        if is_check_world && map_cell.is_world() {
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
        //如果不限制时间,直接跳过
        if time_limit == 0 {
            return;
        }
        let mut task = Task::default();
        task.delay = time_limit;
        task.cmd = TaskCmd::BattleTurnTime;
        task.turn = self.turn;
        let mut map = serde_json::Map::new();
        map.insert("user_id".to_owned(), JsonValue::from(*user_id));
        task.data = JsonValue::from(map);
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
        let target_player = self.get_battle_player(Some(target_user), true)?;
        let mut target_pt = TargetPt::new();
        target_pt
            .target_value
            .push(target_player.get_map_cell_index() as u32);
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
        let center_map_cell = self.tile_map.map_cells.get(center_index as usize);
        if let None = center_map_cell {
            warn!("cal_scope,cal_scope is none!center_index:{}", center_index);
            return (Vec::new(), Vec::new());
        }
        let center_map_cell = center_map_cell.unwrap();
        if targets.is_none() && scope_temp.is_none() {
            let res = TEMPLATES
                .skill_scope_temp_mgr()
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

                    let battle_player = self.get_battle_player(Some(other_user), true);
                    if let Err(e) = battle_player {
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
                        let battle_player = self.get_battle_player(Some(other_user), true);
                        if let Err(e) = battle_player {
                            warn!("{:?}", e);
                            continue;
                        }
                        let battle_player = battle_player.unwrap();
                        if v_u.contains(&battle_player.get_user_id()) {
                            continue;
                        }
                        v_u.push(battle_player.get_user_id());
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
        let judge_temp = TEMPLATES.skill_judge_temp_mgr().get_temp(&skill_judge)?;
        let target_type = TargetType::try_from(judge_temp.target);
        if let Err(e) = target_type {
            warn!("{:?}", e);
            anyhow::bail!("{:?}", e)
        }
        let battle_player = self.get_battle_player(Some(user_id), true).unwrap();
        let target_type = target_type.unwrap();
        match target_type {
            TargetType::PlayerSelf => {
                if HP_LIMIT_GT == judge_temp.id
                    && battle_player.cter.base_attr.hp <= judge_temp.par1 as i16
                {
                    let err_str = format!(
                        "HP_LIMIT_GT!hp of cter <= {}!skill_judge_id:{}",
                        judge_temp.par1, judge_temp.id
                    );
                    warn!("{:?}", err_str);
                    anyhow::bail!("{:?}", err_str)
                } else if LIMIT_TURN_TIMES == judge_temp.id
                    && battle_player
                        .flow_data
                        .turn_limit_skills
                        .contains(&skill_id.unwrap())
                {
                    let err_str = format!("this turn already used this skill!user_id:{},skill_id:{},skill_judge_id:{}",
                    battle_player.get_user_id(),
                    skill_id.unwrap(),
                    skill_judge);
                    warn!("{:?}", err_str);
                    anyhow::bail!("{:?}", err_str)
                } else if LIMIT_ROUND_TIMES == judge_temp.id
                    && battle_player
                        .flow_data
                        .round_limit_skills
                        .contains(&skill_id.unwrap())
                {
                    let err_str = format!("this round already used this skill!user_id:{},skill_id:{},skill_judge_id:{}",
                    battle_player.get_user_id(),
                    skill_id.unwrap(),
                    skill_judge);
                    warn!("{:?}", err_str);
                    anyhow::bail!("{:?}", err_str)
                } else if PAIR_LIMIT == judge_temp.id
                    && !battle_player
                        .flow_data
                        .pair_usable_skills
                        .contains(&skill_id.unwrap())
                {
                    let err_str = format!("could not use this skill!palyer not pair!user_id:{},skill_id:{},skill_judge_id:{}",
                    battle_player.get_user_id(),
                    skill_id.unwrap(),
                    skill_judge);
                    warn!("{:?}", err_str);
                    anyhow::bail!("{:?}", err_str)
                }
            }
            _ => {}
        }
        Ok(())
    }
}

pub fn build_action_unit_pt(
    user_id: u32,
    action_type: ActionType,
    action_value: u32,
) -> ActionUnitPt {
    let mut au = ActionUnitPt::new();
    au.from_user = user_id;
    au.action_type = action_type.into_u32();
    au.action_value.push(action_value);
    au
}
