use crate::robot::cter::{Miner, Robot};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

///pos操作类型
#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum LocationType {
    None = 0,
    KuangChang = 1,
    Bank = 2,
    JiuBa = 3,
    Home = 4,
}

impl Default for LocationType {
    fn default() -> Self {
        LocationType::None
    }
}

///pos操作类型
#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Status {
    None = 0,
    EnterMineAndDigForNugget = 1,
    VisitBankAndDepositGold = 2,
    Drink = 3,
    GoHomeAndSleepTilRested = 4,
}

impl Default for Status {
    fn default() -> Self {
        Status::None
    }
}

pub struct VisitBankAndDepositGold {
    pub status: Status,
}

pub struct EnterMineAndDigForNugget {
    pub status: Status,
}

pub struct GoHomeAndSleepTilRested {
    pub status: Status,
}

pub struct Drink {
    pub status: Status,
}

pub struct Bank {
    pub status: Status,
}

pub trait StatusAction {
    fn enter(&self, cter: &mut Miner);
    fn execute(&self, cter: &mut Miner);
    fn exit(&mut self, cter: &mut Miner);
    fn get_status(&self) -> Status;
}

impl StatusAction for GoHomeAndSleepTilRested {
    fn enter(&self, cter: &mut Miner) {
        if cter.location_type != LocationType::Home {
            cter.change_location(LocationType::Home);
        }
        self.execute(cter);
    }

    fn execute(&self, cter: &mut Miner) {
        println!("矿工一天的工作结束了，睡觉～");
        cter.fatigue = 0;
        cter.thirst = 0;
        println!("第二天到来，矿工该去上班了");
        let e = EnterMineAndDigForNugget {
            status: Status::EnterMineAndDigForNugget,
        };
        cter.change_status(Box::new(e));
    }

    fn exit(&mut self, cter: &mut Miner) {
        println!("退出睡觉模式,该去上班了！");
    }

    fn get_status(&self) -> Status {
        self.status
    }
}

impl StatusAction for Bank {
    fn enter(&self, cter: &mut Miner) {
        if cter.location_type != LocationType::Bank {
            cter.change_location(LocationType::Bank);
        }
        self.execute(cter);
    }

    fn execute(&self, cter: &mut Miner) {
        println!("矿工在银行将金矿兑换成现金并存到银行！");
        cter.money_in_back += cter.gold_carried;
        cter.gold_carried = 0;
        println!(
            "存钱结束，矿工持有金矿数量:{},存有金币:{}",
            cter.gold_carried, cter.money_in_back
        );
        let g = GoHomeAndSleepTilRested {
            status: Status::GoHomeAndSleepTilRested,
        };
        cter.change_status(Box::new(g));
    }

    fn exit(&mut self, miner: &mut Miner) {
        println!("存钱结束，该回家了！");
    }

    fn get_status(&self) -> Status {
        self.status
    }
}

impl StatusAction for Drink {
    fn enter(&self, cter: &mut Miner) {
        if cter.location_type != LocationType::JiuBa {
            cter.change_location(LocationType::JiuBa);
        }
        self.execute(cter);
    }

    fn execute(&self, cter: &mut Miner) {
        println!("矿工累了，喝啤酒解乏");
        cter.thirst -= 1;
        cter.fatigue = 0;
        if cter.thirst == 0 {
            let e = EnterMineAndDigForNugget {
                status: Status::EnterMineAndDigForNugget,
            };
            cter.change_status(Box::new(e));
        } else {
            self.execute(cter);
        }
    }

    fn exit(&mut self, miner: &mut Miner) {
        println!("矿工休息结束！");
    }

    fn get_status(&self) -> Status {
        self.status
    }
}

impl StatusAction for EnterMineAndDigForNugget {
    fn enter(&self, cter: &mut Miner) {
        if cter.location_type != LocationType::KuangChang {
            cter.change_location(LocationType::KuangChang);
        }
        self.execute(cter);
    }

    fn execute(&self, cter: &mut Miner) {
        println!("矿工开始挖金矿");
        cter.add_gold_carried();
        cter.thirst += 1;
        cter.fatigue += 1;
        if cter.gold_carried >= 20 {
            let b = Bank {
                status: Status::VisitBankAndDepositGold,
            };
            cter.change_status(Box::new(b));
        } else if cter.thirst >= 10 {
            let d = Drink {
                status: Status::Drink,
            };
            cter.change_status(Box::new(d));
        } else {
            self.execute(cter);
        }
    }

    fn exit(&mut self, cter: &mut Miner) {
        if cter.thirst >= 10 {
            println!("矿工口渴了，去酒吧喝酒");
        }
        if cter.gold_carried >= 20 {
            println!("金矿包装满了，矿工去银行");
        }
    }

    fn get_status(&self) -> Status {
        self.status
    }
}
