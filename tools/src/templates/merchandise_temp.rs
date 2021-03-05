use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::HashMap;

///商品模版
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct MerchandiseTemp {
    pub id: u32,                  //商品id
    pub price: i32,               //价格
    pub effect_type: u8,          //效果类型
    pub effect_value: i32,        //效果类型
    pub room_type: Vec<u8>,       //房间模式
    pub character_type: Vec<u8>,  //角色类型
    pub other_condition: u32,     //其他条件
    pub turn_limit_buy_times: u8, //每个turn限制购买次数
}

impl Template for MerchandiseTemp {}

#[derive(Debug, Default, Clone)]
pub struct MerchandiseTempMgr {
    pub temps: HashMap<u32, MerchandiseTemp>, //key:id value:itemtemp
}

impl MerchandiseTempMgr {
    #[warn(unreachable_code)]
    pub fn get_temp(&self, id: &u32) -> anyhow::Result<&MerchandiseTemp> {
        let res = self.temps.get(id);
        if res.is_none() {
            let str = format!("ItemTemp is none for id:{}", id);
            anyhow::bail!(str)
        };
        Ok(res.unwrap())
    }

    pub fn init(&mut self, t: Vec<MerchandiseTemp>) {
        for tt in t {
            self.temps.insert(tt.id, tt);
        }
    }
}

impl TemplateMgrTrait for MerchandiseTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }

    fn clear(&mut self) {
        self.temps.clear();
    }
}
