use crate::battle::battle_buff::Buff;
use crate::TEMPLATES;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::slice::Iter;

pub enum CellType {
    InValid = 0,
    UnUse = 1,
    Valid = 2,
}

///地图
#[derive(Debug, Default, Clone)]
pub struct TileMap {
    pub id: u32,                           //地图id
    pub map: Vec<Cell>,                    //地图格子vec
    pub world_cell_map: HashMap<u32, u32>, //世界块map，index，cellid
    pub un_pair_count: i32,                //未配对地图块数量
}

///块的封装结构体
#[derive(Debug, Default, Clone)]
pub struct Cell {
    pub id: u32,                   //块的配置id
    pub index: usize,              //块的下标
    pub buff: Vec<Buff>,           //块的效果
    pub is_world: bool,            //是否世界块
    pub element: u8,               //地图块的属性
    pub extra_buff: Vec<Buff>,     //额外玩家对其添加的buff
    pub user_id: u32,              //这个地图块上面的玩家
    pub pair_index: Option<usize>, //与之配对的下标
}

impl Cell {
    pub fn check_is_locked(&self) -> bool {
        for buff in self.buff.iter() {
            if buff.id == 321 {
                return true;
            }
        }
        false
    }
}

impl TileMap {
    //给客户端显示用的
    pub fn get_cells(&self) -> Vec<u32> {
        let mut v = Vec::new();
        for cell in self.map.iter() {
            if cell.id <= 2 {
                continue;
            }
            v.push(cell.id);
        }
        v
    }

    pub fn get_able_cells(&self) -> Vec<u32> {
        let mut v = Vec::new();
        let tile_map_mgr = crate::TEMPLATES.get_tile_map_ref();
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
    pub fn init(member_count: u32, is_open_world_cell: bool) -> anyhow::Result<Self> {
        let tile_map_mgr = TEMPLATES.get_tile_map_ref();
        let res = tile_map_mgr.member_temps.get(&member_count);
        if let None = res {
            let str = format!("there is no map config for member_count:{}", member_count);
            anyhow::bail!(str)
        }
        let res = res.unwrap();
        let res = res.get(&is_open_world_cell);
        if let None = res {
            let str = format!(
                "there is no map config for member_count:{},is_open_world_cell:{}",
                member_count, is_open_world_cell
            );
            anyhow::bail!(str)
        }

        let res = res.unwrap();
        let mut rand = rand::thread_rng();
        let map_random_index = rand.gen_range(0, res.len());
        let tile_map_temp = res.get(map_random_index).unwrap();
        let mut tmd = TileMap::default();
        tmd.id = tile_map_temp.id;
        tmd.map = Vec::with_capacity(30);

        let mut map = [(0, false); 30];
        let mut index = 0;
        for i in tile_map_temp.map.iter() {
            let mut cell = Cell::default();
            cell.index = index;
            map[index] = (*i, false);
            index += 1;
        }
        let mut empty_v = Vec::new();
        //填充空的格子占位下标
        for index in 0..tile_map_temp.map.len() {
            let res = tile_map_temp.map.get(index).unwrap();
            if *res != 2 {
                continue;
            }
            empty_v.push(index);
        }

        //确定worldcell
        if tile_map_temp.world_cell != 0 {
            let index = rand.gen_range(0, empty_v.len());
            let index_value = empty_v.get(index).unwrap();
            let index_value = *index_value;

            map[index_value] = (tile_map_temp.world_cell, true);
            empty_v.remove(index);
            tmd.world_cell_map
                .insert(index_value as u32, tile_map_temp.world_cell);
        }
        //这里是为了去重，进行拷贝
        let mut random_vec = TEMPLATES.get_cell_ref().type_vec.clone();
        //然后就是rare_cell
        for cell_rare in tile_map_temp.cell_rare.iter() {
            let type_vec = TEMPLATES
                .get_cell_ref()
                .rare_map
                .get(&cell_rare.rare)
                .unwrap()
                .clone();
            let mut size = 0;
            'out: loop {
                if size >= cell_rare.count {
                    break 'out;
                }
                for cell_type in type_vec.iter() {
                    if size >= cell_rare.count {
                        break 'out;
                    }
                    //先随出celltype列表中的一个
                    let mut cell_v = hs_2_v(&random_vec.get(cell_type).unwrap());
                    if cell_v.len() == 0 {
                        continue;
                    }
                    let index = rand.gen_range(0, cell_v.len());
                    let cell_id = *cell_v.get(index).unwrap();
                    for _ in 1..=2 {
                        //然后再随机放入地图里
                        let index = rand.gen_range(0, empty_v.len());
                        let index_value = empty_v.get(index).unwrap();
                        map[*index_value] = (cell_id, false);
                        empty_v.remove(index);
                        size += 1;
                    }
                    cell_v.remove(index);
                    //删掉选中的，进行去重
                    random_vec.get_mut(cell_type).unwrap().remove(&cell_id);
                }
            }
        }
        let mut index = 0;
        let mut un_pair_count = 0;
        for (cell_id, is_world) in map.iter() {
            let mut cell = Cell::default();
            cell.id = *cell_id;
            cell.index = index;
            cell.is_world = *is_world;
            let mut buffs: Option<Iter<u32>> = None;
            if cell.is_world {
                let world_cell = TEMPLATES.get_world_cell_ref().temps.get(cell_id).unwrap();
                buffs = Some(world_cell.buff.iter());
            } else if cell_id > &(CellType::Valid as u32) {
                let cell_temp = TEMPLATES.get_cell_ref().temps.get(cell_id).unwrap();
                buffs = Some(cell_temp.buff.iter());
                cell.element = cell_temp.element;
                un_pair_count += 1;
            }
            if let Some(buffs) = buffs {
                let mut buff_array = Vec::new();
                for buff_id in buffs {
                    let buff_temp = crate::TEMPLATES.get_buff_ref().get_temp(buff_id).unwrap();
                    let buff = Buff::from(buff_temp);
                    buff_array.push(buff);
                }
                cell.buff = buff_array;
            }
            tmd.map.push(cell);
            index += 1;
        }
        tmd.un_pair_count = un_pair_count;
        Ok(tmd)
    }
}

fn hs_2_v(hs: &HashSet<u32>) -> Vec<u32> {
    let mut v = Vec::new();
    for i in hs.iter() {
        v.push(*i);
    }
    v
}
