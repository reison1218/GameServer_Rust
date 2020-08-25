use crate::battle::battle::BattleData;
use crate::battle::battle_enum::skill_judge_type::HP_LIMIT_GT;
use crate::battle::battle_enum::{EffectType, TargetType, TRIGGER_SCOPE_NEAR_TEMP_ID};
use crate::room::character::BattleCharacter;
use crate::room::map_data::{Cell, CellType};
use crate::task_timer::{Task, TaskCmd};
use crate::TEMPLATES;
use log::{error, info, warn};
use protobuf::Message;
use std::collections::HashMap;
use tools::cmd_code::ClientCode;
use tools::protos::base::{BuffPt, CellBuffPt, EffectPt, TargetPt, TriggerEffectPt};
use tools::protos::battle::S_BATTLE_TURN_NOTICE;
use tools::templates::skill_scope_temp::SkillScopeTemp;
use tools::util::packet::Packet;

impl BattleData {
    ///加血
    pub fn add_hp(
        &mut self,
        from_user: Option<u32>,
        target: u32,
        hp: i32,
        buff_id: Option<u32>,
    ) -> anyhow::Result<TargetPt> {
        let cter = self.get_battle_cter_mut(Some(target))?;

        if cter.is_died() {
            anyhow::bail!(
                "this cter is died! user_id:{},cter_id:{}",
                target,
                cter.cter_id
            )
        }
        cter.add_hp(hp);
        let target_pt =
            self.build_target_pt(from_user, target, EffectType::Cure, hp as u32, buff_id)?;
        Ok(target_pt)
    }

    pub fn calc_reduce_damage(&self, from_user: u32, target_cter: &mut BattleCharacter) -> i32 {
        let target_user = target_cter.user_id;
        let target_index = target_cter.cell_index as isize;
        let user_v = self.cal_scope(target_user, target_index, TargetType::None, None, None);
        let res = user_v.contains(&from_user);
        target_cter.calc_reduce_damage(res)
    }

    ///扣血
    pub unsafe fn deduct_hp(
        &mut self,
        from: u32,
        target: u32,
        skill_damege: Option<i32>,
        need_rank: bool,
    ) -> anyhow::Result<TargetPt> {
        let battle_data_ptr = self as *mut BattleData;
        let mut target_pt = TargetPt::new();

        let mut ep = EffectPt::new();
        ep.effect_type = EffectType::SkillDamage as u32;

        let target_cter = battle_data_ptr
            .as_mut()
            .unwrap()
            .get_battle_cter_mut(Some(target))?;
        target_pt.target_value.push(target_cter.cell_index as u32);
        let mut res;
        //如果是普通攻击，要算上减伤
        if skill_damege.is_none() {
            let from_cter = battle_data_ptr
                .as_mut()
                .unwrap()
                .get_battle_cter_mut(Some(from))?;
            let attack_damage = from_cter.calc_damage();
            let reduce_damage = self.calc_reduce_damage(from_cter.user_id, target_cter);
            ep.effect_type = EffectType::AttackDamage as u32;
            res = attack_damage - reduce_damage;
            if res < 0 {
                res = 0;
            }
            let gd_buff = target_cter.trigger_attack_damge_gd();
            if gd_buff.0 > 0 {
                let mut te_pt = TriggerEffectPt::new();
                te_pt.set_buff_id(gd_buff.0);
                target_pt.passiveEffect.push(te_pt);
                if gd_buff.1 {
                    target_pt.lost_buffs.push(gd_buff.0);
                }
                res = 0;
            } else {
                target_cter.is_attacked = true;
            }
        } else {
            res = skill_damege.unwrap();
        }
        ep.effect_value = res as u32;
        target_pt.effects.push(ep);
        let is_die = target_cter.sub_hp(res);

        //判断目标角色是否死亡
        if is_die {
            let mut rank_vec_size = self.rank_vec.len();
            if rank_vec_size != 0 {
                rank_vec_size -= 1;
            }
            //判断是否需要排行
            if need_rank {
                self.rank_vec.push(Vec::new());
            }
            let v = self.rank_vec.get_mut(rank_vec_size);
            if v.is_none() {
                error!("rank_vec can not find data!rank_vec_size:{}", rank_vec_size);
                return Ok(target_pt);
            }
            v.unwrap().push(target);

            let cell = self.tile_map.get_cell_mut_by_user_id(target);
            if let Some(cell) = cell {
                cell.user_id = 0;
            }
        }
        Ok(target_pt)
    }

    ///处理地图块配对逻辑
    pub unsafe fn handler_cell_pair(&mut self, user_id: u32) -> bool {
        let battle_cters = &mut self.battle_cter as *mut HashMap<u32, BattleCharacter>;

        let battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id);
        if let None = battle_cter {
            error!("cter is not find!user_id:{}", user_id);
            return false;
        }
        let battle_cter = battle_cter.unwrap();

        let index = battle_cter.cell_index;
        let cell = self.tile_map.map.get_mut(index);
        if let None = cell {
            error!("cell is not find!cell_index:{}", index);
            return false;
        }
        let cell_ptr = cell.unwrap() as *mut Cell;
        let cell_mut = cell_ptr.as_mut().unwrap();
        let mut is_pair = false;
        let cell_id = cell_mut.id;
        let recently_open_cell_index = battle_cter.recently_open_cell_index;
        let mut recently_open_cell_id: Option<u32> = None;
        if let Some(recently_open_cell_index) = recently_open_cell_index {
            let res = self.tile_map.map.get_mut(recently_open_cell_index);
            if let None = res {
                error!("cell not find!cell_index:{}", recently_open_cell_index);
                return false;
            }
            let last_cell = res.unwrap() as *mut Cell;
            self.tile_map.map.get_mut(recently_open_cell_index as usize);
            recently_open_cell_id = Some(last_cell.as_ref().unwrap().id);
            let last_cell = &mut *last_cell;
            //如果配对了，则修改地图块配对的下标
            if let Some(id) = recently_open_cell_id {
                if cell_id == id {
                    cell_mut.pair_index = Some(recently_open_cell_index as usize);
                    last_cell.pair_index = Some(index);
                    is_pair = true;
                }
            } else {
                is_pair = false;
            }
        }
        //配对了就封装
        if is_pair && recently_open_cell_index.is_some() {
            info!(
                "user:{} open cell pair! last_cell:{},now_cell:{}",
                battle_cter.user_id,
                recently_open_cell_index.unwrap() as u32,
                index
            );
        }
        is_pair
    }
    ///发送战斗turn推送
    pub fn send_battle_turn_notice(&mut self) {
        let mut sbtn = S_BATTLE_TURN_NOTICE::new();
        sbtn.set_user_id(self.get_turn_user(None).unwrap());
        //角色身上的
        for cter in self.battle_cter.values() {
            let cter_pt = cter.convert_to_battle_cter();
            sbtn.cters.push(cter_pt);
        }

        //地图块身上的
        for cell in self.tile_map.map.iter() {
            let mut cbp = CellBuffPt::new();
            cbp.index = cell.index as u32;
            for buff in cell.buffs.iter() {
                if cell.passive_buffs.contains(&buff.id) {
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
        for user_id in self.battle_cter.clone().keys() {
            self.send_2_client(ClientCode::BattleTurnNotice, *user_id, bytes.clone());
        }
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
            let str = format!("battle_cter not find!user_id:{}", _user_id);
            anyhow::bail!("{:?}", str)
        }
        Ok(cter.unwrap())
    }

    pub fn send_2_client(&mut self, cmd: ClientCode, user_id: u32, bytes: Vec<u8>) {
        let bytes = Packet::build_packet_bytes(cmd as u32, user_id, bytes, true, true);
        self.get_sender_mut().write(bytes);
    }

    ///检查目标数组
    pub fn check_target_array(
        &self,
        user_id: u32,
        target_type: TargetType,
        target_array: &[u32],
        skill_judge: u32,
    ) -> anyhow::Result<()> {
        match target_type {
            //无效目标
            TargetType::None => {
                let str = format!("this target_type is invaild!target_type:{:?}", target_type);
                anyhow::bail!(str)
            }
            //任意玩家
            TargetType::AnyPlayer => {
                let mut v = Vec::new();
                for index in target_array {
                    let res = self.get_battle_cter_by_cell_index(*index as usize);
                    if let Ok(cter) = res {
                        check_skill_judge(cter, skill_judge)?;
                        v.push(cter.user_id);
                    }
                }
                self.check_user_target(&v[..], None)? //不包括自己的其他玩家
            } //玩家自己
            TargetType::PlayerSelf => {
                let cter = self.get_battle_cter(Some(user_id)).unwrap();
                check_skill_judge(cter, skill_judge)?;
            } //玩家自己
            //全图玩家
            TargetType::AllPlayer => {
                let mut v = Vec::new();
                for index in target_array {
                    let res = self.get_battle_cter_by_cell_index(*index as usize);
                    if let Ok(cter) = res {
                        check_skill_judge(cter, skill_judge)?;
                        v.push(cter.user_id);
                    }
                }
                self.check_user_target(&v[..], None)?; //不包括自己的其他玩家
            }
            TargetType::OtherAllPlayer => {
                let mut v = Vec::new();
                for index in target_array {
                    let res = self.get_battle_cter_by_cell_index(*index as usize);
                    if let Ok(cter) = res {
                        check_skill_judge(cter, skill_judge)?;
                        v.push(cter.user_id);
                    }
                }
                //除自己所有玩家
                self.check_user_target(&v[..], Some(user_id))?
            } //除自己外任意玩家
            TargetType::OtherAnyPlayer => {
                let mut v = Vec::new();
                for index in target_array {
                    let res = self.get_battle_cter_by_cell_index(*index as usize);
                    if let Ok(cter) = res {
                        check_skill_judge(cter, skill_judge)?;
                        v.push(cter.user_id);
                    }
                }
                //除自己所有玩家
                self.check_user_target(&v[..], Some(user_id))?
            }
            //地图块
            TargetType::Cell => {
                //校验地图块下标有效性
                for index in target_array {
                    let index = *index as usize;
                    self.check_choice_index(index, false, false, false, false)?
                }
            }
            //未翻开的地图块
            TargetType::UnOpenCell => {
                for index in target_array {
                    self.check_choice_index(*index as usize, true, true, true, false)?;
                }
            } //未配对的地图块
            TargetType::UnPairCell => {
                for index in target_array {
                    self.check_choice_index(*index as usize, true, true, true, true)?
                }
            } //空的地图块
            TargetType::NullCell => {
                for index in target_array {
                    self.check_choice_index(*index as usize, true, true, false, true)?
                }
            } //空的地图块，上面没人
            TargetType::UnPairNullCell => {
                for index in target_array {
                    let index = *index as usize;
                    self.check_choice_index(index, false, false, false, true)?
                }
            } //地图块上的玩家
            TargetType::CellPlayer => {}
        }
        Ok(())
    }

    ///检测目标玩家
    pub fn check_user_target(&self, vec: &[u32], check_self_id: Option<u32>) -> anyhow::Result<()> {
        for member_id in vec.iter() {
            let member_id = *member_id;
            //校验有没有
            if !self.battle_cter.contains_key(&member_id) {
                let str = format!("battle_cter is not find!user_id:{}", member_id);
                anyhow::bail!(str)
            }
            //校验是不是自己
            if check_self_id.is_some() && member_id == check_self_id.unwrap() {
                let str = format!("target_user_id==self!target_user_id:{}", member_id);
                anyhow::bail!(str)
            }
        }
        Ok(())
    }

    //检测地图块是否选择
    pub fn check_choice_index(
        &self,
        index: usize,
        is_check_pair: bool,
        is_check_world: bool,
        is_check_locked: bool,
        is_check_has_user: bool,
    ) -> anyhow::Result<()> {
        let res = self.tile_map.map.get(index);
        if res.is_none() {
            let str = format!("this cell is not find!index:{}", index);
            anyhow::bail!(str)
        }
        let cell = res.unwrap();

        if cell.id < CellType::Valid as u32 {
            let str = format!(
                "auto_pair_cell, this is cell can not be choice!index:{}",
                cell.index
            );
            anyhow::bail!(str)
        }

        let cell = res.unwrap();
        if is_check_pair && cell.pair_index.is_some() {
            let str = format!(
                "auto_pair_cell, this cell already pair!index:{}",
                cell.index
            );
            anyhow::bail!(str)
        }

        if is_check_world && cell.is_world {
            let str = format!("world_cell can not be choice!index:{}", cell.index);
            anyhow::bail!(str)
        }
        if is_check_locked && cell.check_is_locked() {
            let str = format!("this cell is locked!index:{}", cell.index);
            anyhow::bail!(str)
        }
        if is_check_has_user && cell.user_id > 0 {
            let str = format!("this cell has user!index:{}", cell.index);
            anyhow::bail!(str)
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
        task.cmd = TaskCmd::BattleTurnTime as u16;

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
        let target_cter = self.get_battle_cter(Some(target_user))?;
        let mut target_pt = TargetPt::new();
        target_pt.target_value.push(target_cter.cell_index as u32);
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
    ) -> Vec<u32> {
        let mut v = Vec::new();
        let center_cell = self.tile_map.map.get(center_index as usize).unwrap();
        //相邻，直接拿常量
        if targets.is_none() && scope_temp.is_none() {
            let scope_temp = TEMPLATES
                .get_skill_scope_ref()
                .get_temp(&TRIGGER_SCOPE_NEAR_TEMP_ID);
            if let Err(e) = scope_temp {
                error!("{:?}", e);
                return v;
            }
            let scope_temp = scope_temp.unwrap();

            for direction_temp2d in scope_temp.scope2d.iter() {
                for coord_temp in direction_temp2d.direction2d.iter() {
                    let x = center_cell.x + coord_temp.x;
                    let y = center_cell.y + coord_temp.y;
                    let cell_index = self.tile_map.coord_map.get(&(x, y));
                    if let None = cell_index {
                        continue;
                    }
                    let cell_index = cell_index.unwrap();
                    let cell = self.tile_map.map.get(*cell_index);
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
                    let other_user = cell.user_id;

                    //如果玩家id大于0
                    if other_user == 0 {
                        continue;
                    }
                    let cter = self.get_battle_cter(Some(other_user));
                    if let Err(e) = cter {
                        warn!("{:?}", e);
                        continue;
                    }
                    v.push(other_user);
                }
            }
        } else if targets.is_none() && scope_temp.is_some() {
            let scope_temp = scope_temp.unwrap();

            for direction_temp2d in scope_temp.scope2d.iter() {
                for coord_temp in direction_temp2d.direction2d.iter() {
                    let x = center_cell.x + coord_temp.x;
                    let y = center_cell.y + coord_temp.y;
                    let cell_index = self.tile_map.coord_map.get(&(x, y));
                    if let None = cell_index {
                        continue;
                    }
                    let cell_index = cell_index.unwrap();
                    let cell = self.tile_map.map.get(*cell_index);
                    if cell.is_none() {
                        continue;
                    }
                    let cell = cell.unwrap();
                    if cell.user_id <= 0 {
                        continue;
                    }
                    //如果目标不能是自己，就跳过
                    if (target_type == TargetType::OtherAllPlayer
                        || target_type == TargetType::OtherAnyPlayer)
                        && cell.user_id == user_id
                    {
                        continue;
                    }
                    let other_user = cell.user_id;
                    //如果玩家id大于0
                    if other_user == 0 {
                        continue;
                    }

                    let cter = self.get_battle_cter(Some(other_user));
                    if let Err(e) = cter {
                        warn!("{:?}", e);
                        continue;
                    }
                    v.push(other_user);
                }
            }
        } else {
            let targets = targets.unwrap();
            let scope_temp = scope_temp.unwrap();
            //否则校验选中的区域
            for dir in scope_temp.scope2d.iter() {
                for coord_temp in dir.direction2d.iter() {
                    let x = center_cell.x + coord_temp.x;
                    let y = center_cell.y + coord_temp.y;
                    let cell_index = self.tile_map.coord_map.get(&(x, y));
                    if let None = cell_index {
                        continue;
                    }
                    let cell_index = cell_index.unwrap();
                    let cell = self.tile_map.map.get(*cell_index);
                    if let None = cell {
                        continue;
                    }
                    let cell = cell.unwrap();
                    for index in targets.iter() {
                        if cell.index as u32 != *index {
                            continue;
                        }
                        let other_user = cell.user_id;
                        //如果目标不能是自己，就跳过
                        if (target_type == TargetType::OtherAllPlayer
                            || target_type == TargetType::OtherAnyPlayer)
                            && other_user == user_id
                        {
                            continue;
                        }
                        //如果玩家id大于0
                        if other_user == 0 {
                            continue;
                        }
                        let cter = self.get_battle_cter(Some(other_user));
                        if let Err(e) = cter {
                            warn!("{:?}", e);
                            continue;
                        }
                        let cter = cter.unwrap();
                        if v.contains(&cter.user_id) {
                            continue;
                        }
                        v.push(cter.user_id);
                    }
                }
            }
        }
        v
    }
}

///校验技能条件
pub fn check_skill_judge(cter: &BattleCharacter, skill_judge: u32) -> anyhow::Result<()> {
    if skill_judge == 0 {
        return Ok(());
    }
    let judge_temp = TEMPLATES.get_skill_judge_ref().get_temp(&skill_judge)?;
    if HP_LIMIT_GT.contains(&judge_temp.id) && cter.hp <= judge_temp.par1 as i32 {
        anyhow::bail!(
            "HP_LIMIT_GT!hp of cter <= {}!skill_judge_id:{}",
            judge_temp.par1,
            judge_temp.id
        )
    }
    Ok(())
}
