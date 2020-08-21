use crate::battle::battle::BattleData;
use crate::battle::battle_enum::buff_type::{ADD_ATTACK_AND_AOE, RESET_MAP_ADD_ATTACK_BY_ALIVES};
use crate::battle::battle_enum::TargetType;
use crate::battle::battle_enum::{BattleCterState, SkillConsumeType};
use crate::room::character::BattleCharacter;
use crate::room::map_data::TileMap;
use crate::room::room::MEMBER_MAX;
use crate::TEMPLATES;
use log::{error, warn};
use protobuf::Message;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::Deref;
use std::str::FromStr;
use tools::cmd_code::ClientCode;
use tools::protos::base::{ActionUnitPt, SummaryDataPt};
use tools::protos::battle::S_SUMMARY_NOTICE;
use tools::protos::server_protocol::R_G_SUMMARY;

impl BattleData {
    ///处理结算
    /// 返回一个元组类型：是否结算，存活玩家数量，第一名的玩家列表
    pub unsafe fn handler_summary(&mut self) -> (bool, usize, Option<R_G_SUMMARY>) {
        let allive_count = self
            .battle_cter
            .values()
            .filter(|x| x.state == BattleCterState::Alive as u8)
            .count();
        let battle_cters_prt = self.battle_cter.borrow_mut() as *mut HashMap<u32, BattleCharacter>;
        let battle_cters = battle_cters_prt.as_mut().unwrap();
        //如果达到结算条件，则进行结算
        if allive_count <= 1 {
            let mut member: Option<u32> = None;
            for member_cter in self.battle_cter.values() {
                member = Some(member_cter.user_id);
            }
            if let Some(member) = member {
                self.rank_vec.push(vec![member]);
            }
            //等级
            let mut grade;
            let mut ssn = S_SUMMARY_NOTICE::new();
            let mut rank = 0_u32;

            let mut index = self.rank_vec.len();
            if index == 0 {
                return (false, allive_count, None);
            } else {
                index -= 1;
            }
            let mut max_grade = 2_i32;
            let max_grade_temp = TEMPLATES.get_constant_ref().temps.get("max_grade");
            match max_grade_temp {
                Some(max_grade_temp) => {
                    let res = u32::from_str(max_grade_temp.value.as_str());
                    match res {
                        Ok(res) => {
                            max_grade = res as i32;
                        }
                        Err(e) => {
                            error!("{:?}", e);
                        }
                    }
                }
                None => {
                    error!("max_grade is not find!");
                }
            }
            let mut rgs = R_G_SUMMARY::new();
            loop {
                for members in self.rank_vec.get(index) {
                    for member_id in members.iter() {
                        let cter = battle_cters.get_mut(member_id);
                        if cter.is_none() {
                            error!("handler_summary!cter is not find!user_id:{}", member_id);
                            continue;
                        }
                        let cter = cter.unwrap();
                        grade = cter.grade as i32;

                        //处理grade升级和降级
                        if rank == 0 {
                            grade += 1;
                        } else {
                            grade -= 1;
                        }
                        //满足条件就初始化
                        if grade > max_grade || grade <= 0 {
                            grade = 1;
                        }
                        let mut smp = SummaryDataPt::new();
                        smp.user_id = *member_id;
                        smp.cter_id = cter.cter_id;
                        smp.rank = rank;
                        smp.grade = grade as u32;
                        ssn.summary_datas.push(smp.clone());
                        rgs.summary_datas.push(smp);
                    }
                    rank += 1;
                }
                if index > 0 {
                    index -= 1;
                } else {
                    break;
                }
            }

            let res = ssn.write_to_bytes();

            match res {
                Ok(bytes) => {
                    let v = self.get_battle_cters_vec();
                    for member_id in v {
                        self.send_2_client(ClientCode::SummaryNotice, member_id, bytes.clone());
                    }
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            }
            return (true, allive_count, Some(rgs));
        }
        return (false, allive_count, None);
    }

    ///使用道具,道具都是一次性的，用完了就删掉
    /// user_id:使用道具的玩家
    /// item_id:道具id
    pub fn use_item(
        &mut self,
        user_id: u32,
        item_id: u32,
        au: &mut ActionUnitPt,
    ) -> Option<Vec<ActionUnitPt>> {
        let battle_cter = self.get_battle_cter(Some(user_id)).unwrap();
        let item = battle_cter.items.get(&item_id).unwrap();
        let skill_id = item.skill_temp.id;
        let mut targets = Vec::new();
        targets.push(user_id);
        let res = self.use_skill(user_id, skill_id, targets, au);
        let battle_cter = self.get_battle_cter_mut(Some(user_id)).unwrap();
        //用完了就删除
        battle_cter.items.remove(&item_id);
        res
    }

    ///跳过回合
    pub fn skip_turn(&mut self, _au: &mut ActionUnitPt) {
        //直接下一个turn
        self.next_turn();
    }

    ///使用技能
    /// user_id:使用技能的玩家id
    /// target_array目标数组
    pub fn use_skill(
        &mut self,
        user_id: u32,
        skill_id: u32,
        target_array: Vec<u32>,
        au: &mut ActionUnitPt,
    ) -> Option<Vec<ActionUnitPt>> {
        let mut au_vec: Option<Vec<ActionUnitPt>> = None;
        unsafe {
            //战斗角色
            let battle_cter_ptr =
                self.get_battle_cter_mut(Some(user_id)).unwrap() as *mut BattleCharacter;
            let battle_cter = battle_cter_ptr.as_mut().unwrap();
            //战斗角色身上的技能
            let skill = battle_cter.skills.get_mut(&skill_id).unwrap();
            //校验cd
            if skill.cd_times > 0 {
                let str = format!(
                    "can not use this skill!skill_id:{},cd:{}",
                    skill_id, skill.cd_times
                );
                warn!("{:?}", str);
                return None;
            }

            let target = skill.skill_temp.target;
            let target_type = TargetType::try_from(target as u8).unwrap();

            //技能判定
            let skill_judge = skill.skill_temp.skill_judge as u32;

            //校验目标类型
            let res = self.check_target_array(user_id, target_type, &target_array, skill_judge);
            if let Err(e) = res {
                let str = format!("{:?}", e);
                warn!("{:?}", str);
                return None;
            }

            //根据技能id去找函数指针里面的函数，然后进行执行
            let self_ptr = self as *mut BattleData;
            for skill_ids in self_ptr.as_ref().unwrap().skill_cmd_map.keys() {
                if !skill_ids.deref().contains(&skill_id) {
                    continue;
                }
                let fn_ptr = self.skill_cmd_map.get_mut(skill_ids.deref()).unwrap();
                au_vec = fn_ptr(self, user_id, skill_id, target_array, au);
                break;
            }

            //如果不是用能量的，则重制cd
            if skill.skill_temp.consume_type != SkillConsumeType::Energy as u8 {
                skill.reset_cd();
            } else if skill.skill_temp.consume_value > battle_cter.energy {
                battle_cter.energy = 0;
            } else {
                battle_cter.energy -= skill.skill_temp.consume_value;
            }
        }
        au_vec
    }

    ///普通攻击
    /// user_id:发动普通攻击的玩家
    /// targets:被攻击目标
    pub unsafe fn attack(&mut self, user_id: u32, targets: Vec<u32>, au: &mut ActionUnitPt) {
        let battle_cters = &mut self.battle_cter as *mut HashMap<u32, BattleCharacter>;
        let cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();
        let mut aoe_buff: Option<u32> = None;

        //塞选出ape的buff
        cter.buffs
            .values()
            .filter(|buff| ADD_ATTACK_AND_AOE.contains(&buff.id))
            .for_each(|buff| {
                aoe_buff = Some(buff.id);
            });

        let index = targets.get(0).unwrap();
        let target_cter = self.get_battle_cter_mut_by_cell_index(*index as usize);

        if let Err(e) = target_cter {
            warn!("{:?}", e);
            return;
        }

        let target_cter = target_cter.unwrap();
        let target_user_id = target_cter.user_id;
        let target_user_index = target_cter.cell_index;
        if target_user_id == user_id {
            let str = format!("the attack target can not be Self!user_id:{}", user_id);
            warn!("{:?}", str);
            return;
        }

        //扣血
        let target_pt = self.deduct_hp(user_id, target_user_id, None, true);

        if let Err(e) = target_pt {
            error!("{:?}", e);
            return;
        }
        let mut target_pt = target_pt.unwrap();
        //目标被攻击，触发目标buff
        self.attacked_trigger_buffs(target_user_id, &mut target_pt);

        au.targets.push(target_pt);
        //检查aoebuff
        if let Some(buff) = aoe_buff {
            let buff = TEMPLATES.get_buff_ref().get_temp(&buff);
            if let Err(e) = buff {
                warn!("{:?}", e);
                return;
            }
            let v = self.cal_scope(
                user_id,
                target_user_index as isize,
                TargetType::OtherAnyPlayer,
                None,
                None,
            );

            //目标周围的玩家
            for target_user in v {
                if target_user_id == target_user {
                    continue;
                }
                //扣血
                let target_pt = self.deduct_hp(user_id, target_user, None, false);
                match target_pt {
                    Ok(mut target_pt) => {
                        //目标被攻击，触发目标buff
                        self.attacked_trigger_buffs(target_user, &mut target_pt);
                        au.targets.push(target_pt);
                    }
                    Err(e) => error!("{:?}", e),
                }
            }
        }
        cter.is_can_attack = false;
    }

    ///刷新地图
    pub fn reset(&mut self, is_world_cell: Option<bool>) -> anyhow::Result<()> {
        let res = TileMap::init(self.battle_cter.len() as u32, is_world_cell)?;
        self.tile_map = res;
        let cter_size = self.battle_cter.len();
        unsafe {
            //触发地图刷新的触发buff
            for cter in self.battle_cter.values_mut() {
                let cter = cter as *mut BattleCharacter;
                for buff in cter.as_mut().unwrap().buffs.values_mut() {
                    if RESET_MAP_ADD_ATTACK_BY_ALIVES.contains(&buff.id) {
                        for _ in 0..cter_size {
                            cter.as_mut().unwrap().trigger_add_damage_buff(buff.id);
                        }
                    }
                }
            }
        }
        Ok(())
    }
    ///翻地图块
    pub fn open_cell(&mut self, index: usize, au: &mut ActionUnitPt) -> Option<Vec<ActionUnitPt>> {
        let user_id = self.get_turn_user(None);
        if let Err(e) = user_id {
            warn!("{:?}", e);
            return None;
        }
        let user_id = user_id.unwrap();
        let is_pair;
        unsafe {
            let au_ptr = au as *mut ActionUnitPt;
            let battle_cters = &mut self.battle_cter as *mut HashMap<u32, BattleCharacter>;
            let battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id);
            if let None = battle_cter {
                error!("battle_cter is not find!user_id:{}", user_id);
                return None;
            }
            let battle_cter = battle_cter.unwrap();

            //先移动
            let v = self.handler_cter_move(user_id, index);
            if let Err(e) = v {
                warn!("{:?}", e);
                return None;
            }
            let v = v.unwrap();
            //判断玩家死了没
            if battle_cter.is_died() {
                return Some(v);
            }
            //再配对
            is_pair = self.handler_cell_pair(user_id, au_ptr.as_mut().unwrap());

            //处理翻地图块触发buff
            let res = self.open_cell_trigger_buff(user_id, au_ptr.as_mut().unwrap(), is_pair);
            if let Err(_) = res {
                return None;
            }

            //处理配对成功与否后的数据
            if is_pair {
                //状态改为可以进行攻击
                battle_cter.is_can_attack = true;
                //如果配对了，则清除上一次翻的地图块
                battle_cter.set_recently_open_cell_index(None);
                self.tile_map.un_pair_count -= 2;
            } else {
                //更新最近一次翻的下标
                battle_cter.set_recently_open_cell_index(Some(index));
            }

            battle_cter.is_opened_cell = true;

            //翻块次数-1
            battle_cter.residue_open_times -= 1;

            //玩家技能cd-1
            battle_cter
                .skills
                .values_mut()
                .for_each(|skill| skill.sub_cd(None));
            Some(v)
        }
    }
    ///下个turn
    pub fn next_turn(&mut self) {
        //计算下一个回合
        self.add_next_turn_index();
        //开始回合触发
        self.turn_start_summary();
        //给客户端推送战斗turn推送
        self.send_battle_turn_notice();
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
        let user_id = user_id.unwrap();
        let battle_cter = self.battle_cter.get_mut(&user_id);
        if let None = battle_cter {
            error!("battle_cter is None!user_id:{}", user_id);
            return;
        }
        //结算玩家自己的
        let battle_cter = battle_cter.unwrap();
        battle_cter.turn_reset();

        //结算玩家加在别的玩家身上的
        for cter in self.battle_cter.values_mut() {
            if cter.user_id == user_id {
                continue;
            }
            let mut delete = Vec::new();
            for buff in cter.buffs.values_mut() {
                if buff.user_id != user_id {
                    continue;
                }
                buff.sub_keep_times();
                if buff.keep_times > 0 {
                    continue;
                }
                delete.push(buff.id);
            }
            for buff_id in delete {
                cter.buffs.remove(&buff_id);
            }
        }

        let mut delete = HashMap::new();
        //结算该玩家加在地图块上的buff
        for cell in self.tile_map.map.iter_mut() {
            for buff_index in 0..cell.buffs.len() {
                let buff = cell.buffs.get_mut(buff_index).unwrap();
                if buff.user_id != user_id {
                    continue;
                }
                buff.sub_keep_times();
                if buff.keep_times > 0 {
                    continue;
                }
                if !delete.contains_key(&cell.index) {
                    delete.insert(cell.index, Vec::new());
                }
                delete.get_mut(&cell.index).unwrap().push(buff_index);
            }
        }

        //删掉buff
        for (cell_index, buff_indexs) in delete.iter() {
            let cell = self.tile_map.map.get_mut(*cell_index).unwrap();
            for buff_index in buff_indexs {
                cell.buffs.remove(*buff_index);
            }
        }
    }

    ///下一个
    pub fn add_next_turn_index(&mut self) {
        self.next_turn_index += 1;
        let index = self.next_turn_index;
        if index >= MEMBER_MAX as usize {
            self.next_turn_index = 0;
        }

        let user_id = self.get_turn_user(None);
        if let Ok(user_id) = user_id {
            if user_id == 0 {
                self.add_next_turn_index();
                return;
            }

            let cter = self.battle_cter.get(&user_id);
            match cter {
                Some(cter) => {
                    if cter.state == BattleCterState::Die as u8 {
                        self.add_next_turn_index();
                        return;
                    }
                }
                None => {
                    warn!("add_next_turn_index cter is none!user_id:{}", user_id);
                }
            }
        } else {
            warn!("{:?}", user_id.err().unwrap());
        }
    }
}
