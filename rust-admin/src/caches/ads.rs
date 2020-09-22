use std::collections::HashMap;

/// 页面
pub const PAGE_IDS: [usize; 2] = [0, 1];

/// 位置
pub const POSITION_IDS: [usize; 4] = [0, 1, 2, 3];

lazy_static! {
    pub static ref PAGES: HashMap<usize, &'static str> = {
        let mut data = HashMap::new();
        data.insert(0, "网站首页");
        data.insert(1, "详情页面");
        data
    };
}

lazy_static! { 
    pub static ref POSITIONS: HashMap<usize, &'static str> = { 
        let mut data = HashMap::new();
        data.insert(0, "顶部");
        data.insert(1, "中间左侧");
        data.insert(2, "中间右侧");
        data.insert(3, "底部");
        data
    };
}
