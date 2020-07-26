use std::env;
use std::collections::{HashMap, HashSet};
use tools::templates::tile_map_temp::TileMapTempMgr;
use rand::{random, Rng};
use std::borrow::Borrow;
use tools::templates::template::TemplatesMgr;

pub fn generate_map(){
    let path = env::current_dir().unwrap();
    let str = path.as_os_str().to_str().unwrap();
    let res = str.to_string()+"/template";
    let  temp_mgr = tools::templates::template::init_temps_mgr(res.as_str());
    let tmd = TileMap::init(&temp_mgr);
    println!("{:?}",tmd);
}

///块的封装结构体
#[derive(Debug, Default, Clone)]
pub struct Cell {
    pub id: u32,        //块的配置id
    pub index: usize,   //块的下标
    pub buff: Vec<u32>, //块的效果
    pub is_world: bool, //是否世界块
}

#[derive(Default,Debug)]
struct TileMap{
    id:u32,//地图id
    map:Vec<Cell>,//地图格子vec
    world_cell_map:HashMap<u32,u32>,//世界块map，index，cellid
}

impl TileMap{
    pub fn init(temp_mgr:&TemplatesMgr)->Self{
        let tile_map_mgr = temp_mgr.get_tile_map_ref();
        let tile_map_temp = tile_map_mgr.temps.get(&4001_u32).unwrap();
        let mut tmd = TileMap::default();
        tmd.id = 4001_u32;
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
        let mut rand = rand::thread_rng();
        //先随机worldcell
        for cell_id in tile_map_temp.world_cell.iter() {
            if cell_id == &0 {
                continue;
            }
            let index = rand.gen_range(0, empty_v.len());
            let index_value = empty_v.get(index).unwrap();
            let index_value = *index_value;

            map[index_value] = (*cell_id, true);
            empty_v.remove(index);
            tmd.world_cell_map.insert(index_value as u32, *cell_id);
        }

        //然后就是rare_cell
        for cell_rare in tile_map_temp.cell_rare.iter() {
            let mut type_vec = temp_mgr
                .get_cell_ref()
                .rare_map
                .get(&cell_rare.rare)
                .unwrap().clone();
            let mut size = 0;

            let mut random_vec = temp_mgr.get_cell_ref().type_vec.clone();
            'out:loop{
                if size >= cell_rare.count {
                    break 'out;
                }
                for cell_type in type_vec.iter() {
                    if size >= cell_rare.count {
                        break 'out;
                    }
                    //先随出celltype列表中的一个
                    let mut cell_v = hs_2_v(&random_vec.get(cell_type).unwrap());
                    if cell_v.len() == 0{
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
                    random_vec.get_mut(cell_type).unwrap().remove(&cell_id);
                }
            }
        }
        let mut index = 0;
        for (cell_id, is_world) in map.iter() {
            let mut cell = Cell::default();
            cell.id = *cell_id;
            cell.index = index;
            cell.is_world = *is_world;
            if cell.is_world {
                let res = temp_mgr.get_world_cell_ref().temps.get(cell_id).unwrap();
                cell.buff = res.buff.clone();
            } else if cell_id>&2{
                let res = temp_mgr.get_cell_ref().temps.get(cell_id).unwrap();
                cell.buff = res.buff.clone();
            }
            tmd.map.push(cell);
            index += 1;
        }
        tmd
    }
}

fn hs_2_v(hs:&HashSet<u32>)->Vec<u32>{
    let mut v = Vec::new();
    for i in hs.iter(){
        v.push(*i);
    }
    v
}