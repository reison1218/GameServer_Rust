use crate::battle::battle::BattleData;
use crate::battle::battle_enum::buff_type::{
    ADD_ATTACK_AND_AOE, MANUAL_MOVE_AND_PAIR_ADD_ENERGY, PAIR_WOOD_ADD_ATTACK, RESET_MAP_ADD_ATTACK,
};
use crate::battle::battle_enum::{DamageType, SkillConsumeType};
use crate::battle::battle_enum::{TargetType, TRIGGER_SCOPE_NEAR_TEMP_ID};
use crate::battle::battle_skill::Skill;
use crate::battle::battle_trigger::TriggerEvent;
use crate::room::map_data::MapCellType;
use crate::room::map_data::TileMap;
use crate::TEMPLATES;
use log::{error, warn};
use protobuf::Message;
use std::convert::TryFrom;
use tools::cmd_code::ClientCode;
use tools::protos::base::{ActionUnitPt, SummaryDataPt};
use tools::protos::battle::S_SUMMARY_NOTICE;
use tools::protos::server_protocol::B_S_SUMMARY;

use super::battle_enum::BattlePlayerState;
use super::mission::{trigger_mission, MissionTriggerType};

impl BattleData {
    ///处理战斗结算核心逻辑，不管地图刷新逻辑
    /// 返回一个元组类型：是否结算，存活玩家数量，第一名的玩家列表
    pub fn summary(&mut self) -> Vec<B_S_SUMMARY> {
        let allive_count = self
            .battle_player
            .values()
            .filter(|x| x.status.battle_state == BattlePlayerState::Normal)
            .count();

        //回客户端消息
        let mut ssn = S_SUMMARY_NOTICE::new();
        let mut need_summary = false;
        let mut bgs = Vec::new();
        let (leave_user, punishment) = self.leave_user;
        let self_ptr = self as *mut BattleData;
        //判断结算方式
        if allive_count <= 1 {
            //如果房间就只有最后一个人了，直接计算
            need_summary = true;
            //如果达到结算条件，则进行结算
            if self.room_type.is_boss_type() {
                self.summary_for_world_boss();
            } else {
                let mut user_id;
                unsafe {
                    let self_mut = self_ptr.as_mut().unwrap();
                    for battle_player in self.battle_player.values() {
                        if battle_player.is_died() {
                            continue;
                        }

                        let str = format!(
                            "player died!because is battle_over,user_id:{}",
                            battle_player.get_user_id()
                        );
                        user_id = battle_player.get_user_id();
                        self_mut.after_player_died_trigger(
                            user_id,
                            user_id,
                            true,
                            false,
                            Some(str),
                        );
                    }
                }
            }
        } else if leave_user > 0 {
            //如果有玩家退出房间
            need_summary = true;
        }
        if need_summary {
            for spa_v in self.summary_vec.iter_mut() {
                for su in spa_v.iter_mut() {
                    let smp: SummaryDataPt = su.clone().into();
                    ssn.summary_datas.push(smp.clone());
                    if !su.push_to_server && self.room_type.is_match_type() {
                        let mut bg = B_S_SUMMARY::new();
                        bg.set_room_type(self.room_type.into_u32());
                        bg.set_summary_data(smp);
                        bgs.push(bg);
                        su.push_to_server = true;
                    }
                }
            }
        }

        if ssn.summary_datas.len() > 0 {
            let res = ssn.write_to_bytes();
            match res {
                Ok(bytes) => {
                    let v = self.get_battle_players_vec();
                    for member_id in v {
                        //强退的人不发
                        if member_id == leave_user && punishment {
                            continue;
                        }
                        self.send_2_client(ClientCode::SummaryNotice, member_id, bytes.clone());
                    }
                    self.leave_user = (0, false);
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
        bgs
    }

    ///使用技能
    /// user_id:使用技能的玩家id
    /// target_array目标数组
    pub fn use_skill(
        &mut self,
        cter_id: u32,
        skill_id: u32,
        is_item: bool,
        target_array: Vec<u32>,
        au: &mut ActionUnitPt,
    ) -> anyhow::Result<Option<Vec<(u32, ActionUnitPt)>>> {
        let mut au_vec: Option<Vec<(u32, ActionUnitPt)>> = None;
        let self_ptr = self as *mut BattleData;
        unsafe {
            let self_mut = self_ptr.as_mut().unwrap();
            //战斗角色
            let res = self_mut.get_battle_cter_mut(cter_id, true);
            if let Err(e) = res {
                error!("{:?}", e);
                anyhow::bail!("")
            }
            let battle_cter = res.unwrap();
            let user_id = battle_cter.get_user_id();
            //战斗角色身上的技能
            let mut skill_s;
            let skill;
            if is_item {
                let res = TEMPLATES.skill_temp_mgr().get_temp(&skill_id);
                if let Err(e) = res {
                    error!("{:?}", e);
                    anyhow::bail!("")
                }
                let skill_temp = res.unwrap();
                skill_s = Some(Skill::from_skill_temp(skill_temp, true));
                skill = skill_s.as_mut().unwrap();
            } else {
                skill = battle_cter.skills.get_mut(&skill_id).unwrap();
                let res = self.before_use_skill_trigger(cter_id);
                if let Err(e) = res {
                    warn!("{:?}", e);
                    anyhow::bail!("")
                }
            }

            let target = skill.skill_temp.target;
            let target_type = TargetType::try_from(target as u8).unwrap();

            //技能判定
            let skill_judge = skill.skill_temp.skill_judge as u32;
            let skill_function_id = skill.function_id;
            //校验目标类型
            let res = self.check_target_array(user_id, target_type, &target_array);
            if let Err(e) = res {
                warn!("{:?},skill_id:{}", e, skill_id);
                anyhow::bail!("")
            }

            //校验技能可用判定条件
            if skill_judge > 0 {
                self.check_skill_judge(cter_id, skill_judge, Some(skill_id), None)?;
            }

            //根据技能id去找函数指针里面的函数，然后进行执行
            let self_ptr = self as *mut BattleData;
            for skill_function_ids in self_ptr.as_ref().unwrap().skill_function_cmd_map.keys() {
                if !skill_function_ids.contains(&skill_function_id) {
                    continue;
                }
                let fn_ptr = self
                    .skill_function_cmd_map
                    .get_mut(skill_function_ids)
                    .unwrap();
                au_vec = fn_ptr(self, cter_id, skill_id, target_array, au);
                break;
            }

            //如果不是用能量的，则用完技能之后重制cd,把cd加上
            if skill.skill_temp.consume_type != SkillConsumeType::Energy as u8 {
                if skill.cd_times <= 0 {
                    skill.reset_cd();
                }
            } else {
                let mut v = skill.skill_temp.consume_value as i8;
                v = v * -1;
                battle_cter.add_energy(v);
            }
            //使用技能后触发
            self.after_use_skill_trigger(cter_id, skill_id, is_item, au);
        }
        Ok(au_vec)
    }

    ///普通攻击
    /// user_id:发动普通攻击的玩家
    /// targets:被攻击目标
    pub unsafe fn attack(
        &mut self,
        cter_id: u32,
        targets: Vec<u32>,
        au: &mut ActionUnitPt,
    ) -> anyhow::Result<()> {
        let self_ptr = self as *mut BattleData;
        let self_mut = self_ptr.as_mut().unwrap();

        let battle_cter = self_mut.get_battle_cter_mut(cter_id, true);
        if let Err(e) = battle_cter {
            warn!("{:?}", e);
            return Ok(());
        }
        let battle_cter = battle_cter.unwrap();
        let user_id = battle_cter.get_user_id();
        let mut aoe_buff: Option<u32> = None;
        //塞选出ape的buff
        battle_cter
            .battle_buffs
            .buffs()
            .values()
            .filter(|buff| ADD_ATTACK_AND_AOE.contains(&buff.function_id))
            .for_each(|buff| {
                aoe_buff = Some(buff.get_id());
            });

        let index = targets.get(0).unwrap();
        let target_cter = self.get_battle_cter_mut_by_map_cell_index(*index as usize);
        if let Err(e) = target_cter {
            warn!("{:?}", e);
            anyhow::bail!("")
        }

        let target_cter = target_cter.unwrap();
        let target_cter_id = target_cter.get_cter_id();
        let target_user_index = target_cter.get_map_cell_index() as u32;
        if target_cter_id == cter_id {
            warn!("the attack target can not be Self!user_id:{}", cter_id);
            anyhow::bail!("")
        }

        let mut target_v = vec![];
        target_v.push((target_cter_id, DamageType::Attack(0)));
        //检查aoebuff
        if let Some(buff) = aoe_buff {
            let buff = TEMPLATES.buff_temp_mgr().get_temp(&buff);
            if let Err(e) = buff {
                warn!("{:?}", e);
                anyhow::bail!("")
            }
            let scope_temp = TEMPLATES
                .skill_scope_temp_mgr()
                .get_temp(&TRIGGER_SCOPE_NEAR_TEMP_ID);
            if let Err(e) = scope_temp {
                warn!("{:?}", e);
                anyhow::bail!("")
            }
            let scope_temp = scope_temp.unwrap();
            let (_, v) = self.cal_scope(
                cter_id,
                target_user_index as isize,
                TargetType::OtherAnyPlayer,
                None,
                Some(scope_temp),
            );

            //目标周围的玩家
            for index in 0..v.len() {
                let target_user = v.get(index).unwrap();
                let target_user = *target_user;
                if target_cter_id == target_user {
                    continue;
                }
                target_v.push((target_user, DamageType::Attack(0)));
            }
        }
        self.batch_deduct_hp(cter_id, target_v, au);

        let battle_player = self_mut.get_battle_player_mut(None, true).unwrap();
        battle_player.change_attack_none();
        //攻击奖励移动点数
        battle_player.attack_reward_movement_points();
        au.set_is_reward_move_points(true);
        //触发翻地图块任务
        trigger_mission(
            self,
            user_id,
            vec![(MissionTriggerType::Attack, 1)],
            (target_cter_id, 0),
        );
        Ok(())
    }

    ///刷新地图
    pub fn reset_map(&mut self, season_is_open: bool, last_map_id: u32) -> anyhow::Result<()> {
        //地图刷新前触发buff
        self.before_map_refresh_buff_trigger();

        unsafe {
            let self_ptr = self as *mut BattleData;
            let self_mut = self_ptr.as_mut().unwrap();
            let mut buff_function_id;
            let mut buff_id;
            let mut cter_id;
            let mut rm_minons = vec![];
            //刷新角色状态和触发地图刷新的触发buff
            for battle_player in self.battle_player.values_mut() {
                if battle_player.is_died() {
                    continue;
                }
                rm_minons.extend_from_slice(battle_player.round_reset().as_slice());

                for cter in battle_player.cters.values() {
                    if cter.is_died() {
                        continue;
                    }
                    cter_id = cter.get_cter_id();
                    let cter_mut = self_mut.get_battle_cter_mut(cter_id, true).unwrap();
                    for buff in cter.battle_buffs.buffs().values() {
                        buff_function_id = buff.function_id;
                        buff_id = buff.get_id();
                        //刷新地图增加攻击力
                        if RESET_MAP_ADD_ATTACK.contains(&buff_function_id) {
                            cter_mut.add_buff(
                                Some(cter_id),
                                None,
                                buff_id,
                                Some(self.next_turn_index),
                            );
                        }
                        //匹配相同元素的地图块加攻击，在地图刷新的时候，攻击要减回来
                        if PAIR_WOOD_ADD_ATTACK == buff_function_id {
                            cter_mut.remove_damage_buff(buff_id);
                        }
                    }
                }
            }
            for id in rm_minons {
                self.cter_player.remove(&id);
            }
        }
        let res = TileMap::init(self, season_is_open, last_map_id)?;
        self.last_map_id = res.id;
        self.tile_map = res;
        self.reflash_map_turn = Some(self.next_turn_index);
        //回合开始的时候触发
        self.round_start_trigger();
        Ok(())
    }

    ///翻地图块
    pub fn open_map_cell(
        &mut self,
        cter_id: u32,
        index: usize,
        au: &mut ActionUnitPt,
    ) -> anyhow::Result<Option<Vec<(u32, ActionUnitPt)>>> {
        let user_id = self.get_turn_user(None);
        if let Err(e) = user_id {
            warn!("{:?}", e);
            anyhow::bail!("open_map_cell fail!")
        }
        let user_id = user_id.unwrap();
        let fail_err_str = format!(
            "open_map_cell fail!user_id:{},index:{}",
            user_id, self.next_turn_index
        );
        let mut is_pair = false;
        unsafe {
            let self_ptr = self as *mut BattleData;
            let self_mut = self_ptr.as_mut().unwrap();
            let battle_player = self_mut.get_battle_player_mut(Some(user_id), true);
            if let Err(e) = battle_player {
                error!("{:?}", e);
                anyhow::bail!("{:?}", fail_err_str.as_str())
            }
            let battle_player = battle_player.unwrap();
            let battle_cter = self_ptr
                .as_mut()
                .unwrap()
                .get_battle_cter_mut(cter_id, true)
                .unwrap();

            //先移动
            let v = self_ptr
                .as_mut()
                .unwrap()
                .handler_cter_move(cter_id, index, au, true);
            if let Err(e) = v {
                warn!("{:?}", e);
                anyhow::bail!("{:?}", fail_err_str.as_str())
            }

            let (is_died, v) = v.unwrap();
            //判断玩家死了没
            if is_died {
                return Ok(Some(v));
            }
            //减去移动点数
            battle_player.flow_data.residue_movement_points -= 1;
            //玩家技能cd-1
            battle_cter.sub_skill_cd(None);
            //设置是否可以结束turn状态
            battle_player.set_is_can_end_turn(true);
            let (cter_id, _) = battle_player.current_cter;

            //判断是否商店
            let map_cell = self.tile_map.map_cells.get(index).unwrap();
            let is_market = map_cell.cell_type == MapCellType::MarketCell;
            if !is_market {
                //打开地图块
                self.exec_open_map_cell(user_id, index);

                //再配对
                is_pair = self.handler_map_cell_pair(user_id, index);
            }
            let mut buff_function_id;
            let mut buff_id;
            //消耗移动点干点什么，配对了又干点什么
            for buff in battle_cter.battle_buffs.buffs().values() {
                buff_function_id = buff.function_id;
                buff_id = buff.get_id();
                //移动加能量，配对加能量
                if MANUAL_MOVE_AND_PAIR_ADD_ENERGY.contains(&buff_function_id) {
                    self_mut.manual_move_and_pair(Some(cter_id), cter_id, buff_id, is_pair, au);
                }
            }

            //如果是商店不要往下走
            if is_market {
                return Ok(Some(v));
            }

            //处理翻地图块触发buff
            let res = self.open_map_cell_trigger(cter_id, au, is_pair);
            if let Err(e) = res {
                anyhow::bail!("{:?}", e)
            }
            Ok(Some(v))
        }
    }

    ///下个turn
    pub fn next_turn(&mut self, need_push_battle_turn_notice: bool) {
        //本回合结束
        self.turn_end_trigger();
        //计算下一个回合
        self.add_next_turn(need_push_battle_turn_notice);

        //创建战斗turn定时器任务
        self.build_battle_turn_task();
    }

    ///回合开始触发
    pub fn turn_start_summary(&mut self) {
        let user_id = self.get_turn_user(None);
        if let Err(e) = user_id {
            error!("{:?}", e);
            return;
        }
        let battle_data = self as *mut BattleData;
        let user_id = user_id.unwrap();
        unsafe {
            let self_mut = self;
            self_mut.turn_start_trigger();
            //容错处理，如果没有地图块可以翻了，就允许不翻块的情况下结束turn
            if user_id > 0 {
                let battle_player = self_mut.get_battle_player(Some(user_id), true);
                if let Ok(_) = battle_player {
                    let mut is_can_skip_turn: bool = true;
                    for &index in self_mut.tile_map.un_pair_map.keys() {
                        let map_cell = self_mut.tile_map.map_cells.get(index);
                        if let None = map_cell {
                            continue;
                        }
                        let map_cell = map_cell.unwrap();
                        if map_cell.check_is_locked() {
                            continue;
                        }
                        is_can_skip_turn = false;
                        break;
                    }

                    //turn结算玩家
                    let battle_player = self_mut.battle_player.get_mut(&user_id).unwrap();
                    if battle_player.has_stone_buff() {
                        is_can_skip_turn = true;
                    }
                    battle_player.set_is_can_end_turn(is_can_skip_turn);
                }
            }

            //结算玩家身上的buff
            for battle_player in self_mut.battle_player.values_mut() {
                let mut cter_id;
                for battle_cter in battle_player.cters.values_mut() {
                    cter_id = battle_cter.base_attr.cter_id;
                    for buff in battle_cter.battle_buffs.buffs().values() {
                        //如果是永久buff,则跳过
                        if buff.permanent {
                            continue;
                        }
                        let buff_id = buff.get_id();
                        battle_data.as_mut().unwrap().consume_buff(
                            buff_id,
                            Some(cter_id),
                            None,
                            true,
                        );
                    }
                }
            }

            //结算该玩家加在地图块上的buff
            for map_cell in self_mut.tile_map.map_cells.iter_mut() {
                for buff in map_cell.buffs.values() {
                    if buff.permanent {
                        continue;
                    }
                    let buff_id = buff.get_id();
                    battle_data.as_mut().unwrap().consume_buff(
                        buff_id,
                        None,
                        Some(map_cell.index),
                        true,
                    );
                }
            }
        }
    }
}
