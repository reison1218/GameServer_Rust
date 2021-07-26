use std::convert::TryFrom;

use super::battle::BattleData;
use super::mission::random_mission;
use log::error;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use protobuf::Message;
use tools::cmd_code::ClientCode;
use tools::protos::battle::S_BUY_NOTICE;

///商品类型
#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum MerchandisType {
    ///加HP
    Hp = 1,
    ///加攻击力
    Attack = 2,
    ///所有技能减CD
    SkillCd = 3,
    ///加能量
    Energy = 4,
    ///获得一个新任务
    Mission = 5,
}

///购物
pub fn handler_buy(battle_data: &mut BattleData, user_id: u32, merchandise_id: u32) {
    let battle_data_ptr = battle_data as *mut BattleData;
    let battle_player = battle_data.battle_player.get_mut(&user_id).unwrap();
    let merchandise_temp = crate::TEMPLATES.merchandise_temp_mgr();
    let temp = merchandise_temp.get_temp(&merchandise_id).unwrap();
    //扣金币
    battle_player.add_gold(-temp.price);
    //添加购买次数
    battle_player.merchandise_data.add_buy_times(merchandise_id);
    let effect_value = temp.effect_value;
    //开始执行给玩家商品
    let mt = MerchandisType::try_from(temp.effect_type).unwrap();
    match mt {
        MerchandisType::Hp => {
            let _ = battle_data.add_hp(Some(user_id), user_id, effect_value as i16, None);
        }
        MerchandisType::Attack => {
            battle_player.get_current_cter_mut().base_attr.atk += temp.effect_value as u8;
        }
        MerchandisType::SkillCd => {
            //玩家技能cd
            battle_player.get_current_cter_mut().sub_skill_cd(Some(effect_value as i8));
        }
        MerchandisType::Energy => {
            battle_player.get_current_cter_mut().add_energy(effect_value as i8);
        }
        MerchandisType::Mission => {
            random_mission(battle_data, user_id);
        }
    }

    //返回客户端
    let mut proto = S_BUY_NOTICE::new();
    proto.set_user_id(user_id);
    proto.set_merchandise_id(merchandise_id);
    let bytes = proto.write_to_bytes();
    match bytes {
        Ok(bytes) => {
            battle_data.send_2_all_client(ClientCode::BuyNoice, bytes);
        }
        Err(e) => {
            error!("{:?}", e);
        }
    }
    let battle_player = battle_data.battle_player.get_mut(&user_id).unwrap();
    //如果是机器人，继续行动
    if battle_player.is_robot() {
        battle_player.robot_start_action(battle_data_ptr);
    }
}
