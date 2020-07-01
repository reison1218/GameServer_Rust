use std::env;
use std::collections::HashMap;
use tools::templates::tile_map_temp::TileMapTempMgr;
use rand::{random, Rng};
use std::borrow::Borrow;

pub fn test_map(){
    let path = env::current_dir().unwrap();
    let str = path.as_os_str().to_str().unwrap();
    let res = str.to_string()+"/template";
    let  temp_mgr = tools::templates::template::init_temps(res.as_str());
    let tile_map = temp_mgr.get_tile_map_ref();

}

struct TileMapData{
    id:u32,
    map:Vec<u32>,
    world_cell_map:HashMap<u32,u32>,
}
//
// impl TileMapData{
//     pub fn init(tile_map_mgr:&TileMapTempMgr)->Self{
//         let tile_map_temp = tile_map_mgr.temps.get(&1001_u32).unwrap();
//
//         let mut empty_v = Vec::new();
//         let mut map = vec_2_array(tile_map_temp.map.borrow());
//
//         //填充空的格子占位下标
//         for index in 0..tile_map_temp.map.len(){
//             let res = tile_map_temp.map.get(index).unwrap();
//             if res !=2{
//                 continue;
//             }
//             empty_v.push(index);
//         }
//
//         //先随机worldcell
//         let world_cell = tile_map_temp.world_cell.get(0).unwrap();
//         let mut rand = rand::thread_rng();
//         let index = rand.gen_range(0,empty_v.len()-1);
//
//     }
// }
//
// pub fn vec_2_array(vec:&[u32])->v{
//     let mut res = [vec.len() as u32;0];
//     for i in 0..vec.len(){
//         res[i] = *vec.get(i).unwrap();
//     }
//     res
// }