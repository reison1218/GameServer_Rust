use crate::fsm::miner::{Miner, Robot};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use std::borrow::BorrowMut;
use std::time::Duration;

///pos操作类型
#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum LocationType {
    None = 0,
    KuangChang = 1,
    Bank = 2,
    Bar = 3,
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
    None = 0,                     //默认无意义状态
    EnterMineAndDigForNugget = 1, //进入矿产并开始挖矿
    GoBankAndDepositGold = 2,     //去银行并且换钱
    GoBarAndDrink = 3,            //去酒吧并喝酒
    GoHomeAndSleepTilRested = 4,  //回家睡觉
}

impl Default for Status {
    fn default() -> Self {
        Status::None
    }
}

#[derive(Default)]
pub struct GoBankAndDepositGold {
    pub status: Status,
}

#[derive(Default)]
pub struct EnterMineAndDigForNugget {
    pub status: Status,
}

#[derive(Default)]
pub struct GoHomeAndSleepTilRested {
    pub status: Status,
}

#[derive(Default)]
pub struct Drink {
    pub status: Status,
}

pub trait StatusAction: Send + 'static {
    fn enter(&self, cter: &Miner);
    fn execute(&self, cter: &Miner);
    fn exit(&self, cter: &Miner);
    fn get_status(&self) -> Status;
}

impl StatusAction for GoHomeAndSleepTilRested {
    fn enter(&self, cter: &Miner) {
        if cter.location_type.load() != LocationType::Home {
            cter.change_location(LocationType::Home);
        }
        self.execute(cter);
    }

    fn execute(&self, cter: &Miner) {
        println!("矿工一天的工作结束了，睡觉～");
        cter.fatigue.store(0);
        cter.thirst.store(0);
        println!("第二天到来，矿工该去上班了");
        let e = EnterMineAndDigForNugget {
            status: Status::EnterMineAndDigForNugget,
        };
        cter.change_status(Box::new(e));
    }

    fn exit(&self, cter: &Miner) {
        println!("退出睡觉模式,该去上班了！");
    }

    fn get_status(&self) -> Status {
        self.status
    }
}

impl StatusAction for GoBankAndDepositGold {
    fn enter(&self, cter: &Miner) {
        if cter.location_type.load() != LocationType::Bank {
            cter.change_location(LocationType::Bank);
        }
        self.execute(cter);
    }

    fn execute(&self, cter: &Miner) {
        println!("矿工在银行将金矿兑换成现金并存到银行！");
        let res = cter.money_in_back.load() + cter.gold_carried.load();
        cter.money_in_back.store(res);

        cter.gold_carried.store(0);
        println!(
            "存钱结束，矿工持有金矿数量:{},存有金币:{}",
            cter.gold_carried.load(),
            cter.money_in_back.load()
        );
        let g = GoHomeAndSleepTilRested {
            status: Status::GoHomeAndSleepTilRested,
        };
        cter.change_status(Box::new(g));
    }

    fn exit(&self, miner: &Miner) {
        println!("存钱结束，该回家了！");
    }

    fn get_status(&self) -> Status {
        self.status
    }
}

impl StatusAction for Drink {
    fn enter(&self, cter: &Miner) {
        if cter.location_type.load() != LocationType::Bar {
            cter.change_location(LocationType::Bar);
        }
        self.execute(cter);
    }

    fn execute(&self, cter: &Miner) {
        println!("矿工累了，喝啤酒解乏");
        cter.thirst.fetch_sub(1);
        cter.fatigue.store(0);
        if cter.thirst.load() == 0 {
            let e = EnterMineAndDigForNugget {
                status: Status::EnterMineAndDigForNugget,
            };
            cter.change_status(Box::new(e));
        } else {
            self.execute(cter);
        }
    }

    fn exit(&self, miner: &Miner) {
        println!("矿工休息结束！");
    }

    fn get_status(&self) -> Status {
        self.status
    }
}

impl StatusAction for EnterMineAndDigForNugget {
    fn enter(&self, cter: &Miner) {
        if cter.location_type.load() != LocationType::KuangChang {
            cter.change_location(LocationType::KuangChang);
        }
        self.execute(cter);
    }

    fn execute(&self, cter: &Miner) {
        std::thread::sleep(Duration::from_secs(1));
        println!("矿工开始挖金矿");
        cter.add_gold_carried();
        let gold_carried = cter.gold_carried.load();
        cter.thirst.fetch_add(1);
        cter.fatigue.fetch_add(1);
        if gold_carried >= 20 {
            let b = GoBankAndDepositGold {
                status: Status::GoBankAndDepositGold,
            };
            cter.change_status(Box::new(b));
        } else if cter.thirst.load() >= 10 {
            let d = Drink {
                status: Status::GoBarAndDrink,
            };
            cter.change_status(Box::new(d));
        } else {
            self.execute(cter);
        }
    }

    fn exit(&self, cter: &Miner) {
        if cter.thirst.load() >= 10 {
            println!("矿工口渴了，去酒吧喝酒");
        }
        if cter.gold_carried.load() >= 20 {
            println!("金矿包装满了，矿工去银行");
        }
    }

    fn get_status(&self) -> Status {
        self.status
    }
}
