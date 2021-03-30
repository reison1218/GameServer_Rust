use crate::TEMPLATES;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use rand::{random, Rng};
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::env;
use std::slice::Iter;
use tools::templates::buff_temp::BuffTemp;
use tools::templates::template::TemplatesMgr;
use tools::templates::tile_map_temp::TileMapTempMgr;

pub fn generate_map() {
    TileMap::init(RoomType::Custom, 4001, 4, 4001).unwrap();
}

///房间类型
#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RoomType {
    None = 0,         //无效
    Custom = 1,       //自定义房间
    Match = 2,        //匹配房间
    SeasonPve = 3,    //赛季PVE房间
    WorldBossPve = 4, //世界boss房间
}

impl RoomType {
    pub fn into_u8(self) -> u8 {
        let res: u8 = self.into();
        res
    }

    pub fn into_u32(self) -> u32 {
        let res: u8 = self.into();
        res as u32
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

impl From<u8> for TargetType {
    fn from(value: u8) -> Self {
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

#[derive(Debug, Clone)]
pub struct Direction {
    pub direction: &'static Vec<isize>,
}

#[derive(Clone, Debug)]
pub struct Buff {
    pub id: u32,
    pub buff_temp: &'static BuffTemp,
    pub trigger_timesed: i8,   //已经触发过的次数
    pub keep_times: i8,        //剩余持续轮数
    pub scope: Vec<Direction>, //buff的作用范围
    pub permanent: bool,       //是否永久
    pub user_id: u32,          //来源的玩家id
}

impl Buff {
    pub fn get_target(&self) -> TargetType {
        let target_type = TargetType::from(self.buff_temp.target);
        target_type
    }

    pub(crate) fn sub_trigger_timesed(&mut self) {
        self.trigger_timesed -= 1;
        if self.trigger_timesed < 0 {
            self.trigger_timesed = 0;
        }
    }

    pub(crate) fn sub_keep_times(&mut self) {
        self.keep_times -= 1;
        if self.keep_times < 0 {
            self.keep_times = 0;
        }
    }
}

impl From<&'static BuffTemp> for Buff {
    fn from(bt: &'static BuffTemp) -> Self {
        let mut b = Buff {
            id: bt.id,
            trigger_timesed: bt.trigger_times as i8,
            keep_times: bt.keep_time as i8,
            buff_temp: bt,
            scope: Vec::new(),
            permanent: bt.keep_time == 0 && bt.trigger_times == 0,
            user_id: 0,
        };
        let mut v = Vec::new();
        let scope_temp = TEMPLATES.skill_scope_temp_mgr().get_temp(&bt.scope);
        if let Ok(scope_temp) = scope_temp {
            if !scope_temp.scope.is_empty() {
                for direction in scope_temp.scope.iter() {
                    let dir = Direction {
                        direction: &direction.direction,
                    };
                    v.push(dir);
                }
                b.scope = v;
            }
        }
        b
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum MapCellType {
    InValid = 0,
    UnUse = 1,
    Valid = 2,
}

impl MapCellType {
    pub fn into_u8(self) -> u8 {
        let res: u8 = self.into();
        res
    }

    pub fn into_u32(self) -> u32 {
        let res = self.into_u8();
        res as u32
    }
}

///地图
#[derive(Debug, Default, Clone)]
pub struct TileMap {
    pub id: u32,                                   //地图id
    pub map_cells: [MapCell; 30],                  //地图格子vec
    pub coord_map: HashMap<(isize, isize), usize>, //坐标对应格子
    pub world_cell_map: HashMap<u32, u32>,         //世界块map，index，map_cellid
    pub un_pair_map: HashMap<usize, u32>,          //未配对的地图块map
}

///块的封装结构体
///块的封装结构体
#[derive(Debug, Default, Clone)]
pub struct MapCell {
    pub id: u32,                   //块的配置id
    pub index: usize,              //块的下标
    pub buffs: HashMap<u32, Buff>, //块的效果
    pub is_world: bool,            //是否世界块
    pub element: u8,               //地图块的属性
    pub passive_buffs: Vec<u32>,   //额外玩家对其添加的buff
    pub user_id: u32,              //这个地图块上面的玩家
    pub pair_index: Option<usize>, //与之配对的下标
    pub x: isize,                  //x轴坐标
    pub y: isize,                  //y轴坐标
}

impl MapCell {
    pub fn check_is_locked(&self) -> bool {
        for buff in self.buffs.values() {
            if buff.id == 321 {
                return true;
            }
        }
        false
    }
}

impl TileMap {
    pub fn get_able_cells(&self) -> Vec<u32> {
        let mut v = Vec::new();
        let tile_map_mgr = crate::TEMPLATES.tile_map_temp_mgr();
        let tile_map_temp = tile_map_mgr.temps.get(&4001_u32).unwrap();
        //填充空的格子占位下标
        for index in 0..tile_map_temp.map.len() {
            let res = tile_map_temp.map.get(index).unwrap();
            if *res != 2 {
                continue;
            }
            v.push(index as u32);
        }
        v
    }

    ///初始化战斗地图数据
    pub fn init(
        room_type: RoomType,
        mut season_id: u32,
        mut member_count: u8,
        last_map_id: u32,
    ) -> anyhow::Result<Self> {
        //先算人数
        if member_count == 3 {
            member_count += 1;
        }
        //创建随机结构体实例
        let mut rand = rand::thread_rng();
        if room_type == RoomType::Match {
            //否则进行随机，0-1，0代表不开启世界块
            let res = rand.gen_range(0..2);
            if res > 0 {
                unsafe {
                    season_id = 4001;
                }
            }
        }

        //拿到地图配置管理器
        let tile_map_mgr = TEMPLATES.tile_map_temp_mgr();
        let tile_map_temp_vec;
        //有世界块的逻辑
        if season_id > 0 {
            //获得赛季配置，里面包括人数，位置，数组对应关系的map
            let res = tile_map_mgr.season_temps.get(&season_id);
            if let None = res {
                anyhow::bail!("there is no map config for season_id:{}", season_id)
            }

            let map = res.unwrap();
            //拿到相对人数的配置
            let res = map.get(&member_count);
            if let None = res {
                anyhow::bail!("there is no map config for member_count:{}", member_count)
            }
            let tile_map_temp_map = res.unwrap();
            let world_cell_index;
            //计算处世界块位置，如果上次id不为0，则拿上次的世界块位置
            let tile_map_temp = tile_map_mgr.get_temp(last_map_id);
            if let Ok(tile_map_temp) = tile_map_temp {
                world_cell_index = tile_map_temp.world_cell_index;
            } else {
                //如果为0，则随机从位置里面拿一个出来
                let mut index_v = Vec::new();
                for cell_index in tile_map_temp_map.keys() {
                    index_v.push(*cell_index);
                }
                let index = rand.gen_range(0..index_v.len());
                world_cell_index = *index_v.get(index).unwrap();
            }
            //拿到世界地图块位置的所有配置
            tile_map_temp_vec = tile_map_temp_map.get(&world_cell_index).unwrap();
        } else {
            //无世界块的逻辑
            let res = tile_map_mgr.member_temps.get(&member_count);
            if let None = res {
                anyhow::bail!("there is no map config for member_count:{}", member_count)
            }
            tile_map_temp_vec = res.unwrap().get(&false).unwrap();
        }
        let mut tile_map_temp_v = Vec::new();
        //这次随机出来的地图，不能与上次一样
        for tmt in tile_map_temp_vec {
            if tmt.id == last_map_id {
                continue;
            }
            tile_map_temp_v.push(tmt.clone());
        }

        let map_random_index = rand.gen_range(0..tile_map_temp_v.len());
        let tile_map_temp = tile_map_temp_v.get(map_random_index).unwrap();
        let mut tmp = TileMap::default();
        tmp.id = tile_map_temp.id;

        let mut map = [(0, false); 30];
        tile_map_temp.map.iter().enumerate().for_each(|i| {
            let index = i.0;
            let value = i.1;
            let mut map_cell = MapCell::default();
            map_cell.index = index;
            map[index] = (*value, false);
        });

        let mut empty_v = Vec::new();

        //填充空的格子占位下标
        for index in 0..tile_map_temp.map.len() {
            let res = tile_map_temp.map.get(index).unwrap();
            if *res != MapCellType::Valid.into_u32() {
                continue;
            }
            empty_v.push(index);
        }

        //确定worldmap_cell
        if tile_map_temp.world_cell != 0 {
            let index = rand.gen_range(0..empty_v.len());
            let index_value = empty_v.get(index).unwrap();
            let index_value = *index_value;

            map[index_value] = (tile_map_temp.world_cell, true);
            empty_v.remove(index);
            tmp.world_cell_map
                .insert(index_value as u32, tile_map_temp.world_cell);
        }
        //这里是为了去重，进行拷贝
        let mut random_vec = TEMPLATES.cell_temp_mgr().type_vec.clone();
        //然后就是rare_map_cell
        for map_cell_rare in tile_map_temp.cell_rare.iter() {
            let type_vec = TEMPLATES
                .cell_temp_mgr()
                .rare_map
                .get(&map_cell_rare.rare)
                .unwrap()
                .clone();
            let mut size = 0;
            'out: loop {
                if size >= map_cell_rare.count {
                    break 'out;
                }
                for map_cell_type in type_vec.iter() {
                    if size >= map_cell_rare.count {
                        break 'out;
                    }
                    //先随出map_celltype列表中的一个
                    let mut map_cell_v = hs_2_v(&random_vec.get(map_cell_type).unwrap());
                    if map_cell_v.len() == 0 {
                        continue;
                    }
                    let index = rand.gen_range(0..map_cell_v.len());
                    let map_cell_id = *map_cell_v.get(index).unwrap();
                    for _ in 1..=2 {
                        //然后再随机放入地图里
                        let index = rand.gen_range(0..empty_v.len());
                        let index_value = empty_v.get(index).unwrap();
                        map[*index_value] = (map_cell_id, false);
                        empty_v.remove(index);
                        size += 1;
                    }
                    map_cell_v.remove(index);
                    //删掉选中的，进行去重
                    random_vec
                        .get_mut(map_cell_type)
                        .unwrap()
                        .remove(&map_cell_id);
                }
            }
        }
        let mut index = 0;
        let mut x = 0_isize;
        let mut y = 0_isize;
        for (map_cell_id, is_world) in map.iter() {
            if x > 5 {
                x = 0;
                y += 1;
            }

            if y >= 5 {
                y = 0;
            }
            let mut map_cell = MapCell::default();
            map_cell.id = *map_cell_id;
            map_cell.index = index;
            map_cell.is_world = *is_world;
            map_cell.x = x;
            map_cell.y = y;
            tmp.coord_map.insert((x, y), index);
            x += 1;
            let mut buffs: Option<Iter<u32>> = None;
            if map_cell.is_world {
                let world_map_cell = TEMPLATES
                    .world_cell_temp_mgr()
                    .temps
                    .get(map_cell_id)
                    .unwrap();
                buffs = Some(world_map_cell.buff.iter());
            } else if map_cell_id > &MapCellType::Valid.into_u32() {
                let map_cell_temp = TEMPLATES.cell_temp_mgr().temps.get(map_cell_id).unwrap();
                buffs = Some(map_cell_temp.buff.iter());
                map_cell.element = map_cell_temp.element;
            }
            if let Some(buffs) = buffs {
                let mut buff_map = HashMap::new();
                for buff_id in buffs {
                    let buff_temp = crate::TEMPLATES.buff_temp_mgr().get_temp(buff_id).unwrap();
                    let buff = Buff::from(buff_temp);
                    buff_map.insert(buff.id, buff);
                    map_cell.passive_buffs.push(*buff_id);
                }
                map_cell.buffs = buff_map;
            }
            tmp.map_cells[index] = map_cell;
            index += 1;
        }
        Ok(tmp)
    }
}

fn hs_2_v(hs: &HashSet<u32>) -> Vec<u32> {
    let mut v = Vec::new();
    for i in hs.iter() {
        v.push(*i);
    }
    v
}
