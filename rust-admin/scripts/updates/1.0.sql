/** 後臺用戶列表 **/
DROP TABLE IF EXISTS admins;
CREATE TABLE IF NOT EXISTS admins ( 
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    name VARCHAR(20) NOT NULL DEFAULT '' COMMENT '登錄名稱',
    password CHAR(32) NOT NULL DEFAULT '' COMMENT '登錄密碼',
    secret CHAR(32) NOT NULL DEFAULT '' COMMENT '密钥',
    last_ip VARCHAR(32) NOT NULL DEFAULT '' COMMENT '最後IP',
    state TINYINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '狀態',
    login_count INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '登錄次數',
    last_login INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '上次登錄時間',
    role_id INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '角色编号',
    created INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '創建時間',
    updated INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '修改時間',
    seq INT NOT NULL DEFAULT 0 COMMENT '排序',
    INDEX(name),
    PRIMARY KEY(id)
) ENGINE=INNODB DEFAULT CHARSET=UTF8 COLLATE=UTF8_GENERAL_CI;
INSERT INTO admins (name, password, last_ip, state, login_count, last_login, role_id, created, updated) VALUES 
('admin', md5('qwe123'), '127.0.0.1', 1, 1, UNIX_TIMESTAMP(), 1, UNIX_TIMESTAMP(), UNIX_TIMESTAMP());
UPDATE admins SET secret = '25BdEMN6yterb6OfCB5aNYyKG87G5Msr', password = 'c54ef8b81f95b8657f988fb609266ee3' WHERE id = 1;

/** 菜单管理 **/
DROP TABLE IF EXISTS menus;
CREATE TABLE IF NOT EXISTS menus (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    parent_id INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '上级编号',
    name VARCHAR(50) NOT NULL DEFAULT '' COMMENT '菜单名称',
    level_id TINYINT NOT NULL DEFAULT 0 COMMENT '级别ID,1:主菜单;2:子菜单',
    state TINYINT NOT NULL DEFAULT 0 COMMENT '状态,0:隐藏;1:显示',
    url VARCHAR(200) NOT NULL DEFAULT '' COMMENT '链接地址',
    is_blank TINYINT NOT NULL DEFAULT 0 COMMENT '是否外链,0:否,1:是',
    is_show TINYINT NOT NULL DEFAULT 0 COMMENT '是否显式,0:否,1:是',
    seq INT NOT NULL DEFAULT 0 COMMENT '排序',
    PRIMARY KEY(id),
    INDEX(parent_id)
) ENGINE=INNODB DEFAULT CHARSET=UTF8 COLLATE=UTF8_GENERAL_Ci;
INSERT INTO menus (parent_id, name, level_id, state, url, is_show) VALUES 
(0, '后台管理', 0, 1, '#', 1),
(0, '内容管理', 0, 1, '#', 1),
(0, '前台管理', 0, 1, '#', 1),

(1, '后台用户', 1, 1, '/admins', 1),
(1, '后台用户编辑', 1, 1, '/admins/edit/\\d+|/admins/save/\\d+', 0),
(1, '后台用户删除', 1, 1, '/admins/delete/\\d+', 0),

(1, '菜单列表', 1, 1, '/menus', 1),
(1, '菜单添加', 1, 1, '/menus/edit/\\d+|/menus/save/\\d+', 0),
(1, '菜单删除', 1, 1, '/menus/delete/\\d+', 0),

(1, '后台角色', 1, 1, '/admin_roles', 1),
(1, '后台角色编辑', 1, 1, '/admin_roles/edit/\\d+|/admin_roles/save/\\d+', 0),
(1, '后台角色删除', 1, 1, '/admin_roles/\\d+', 0),

(2, '视频分类', 1, 1, '/video_categories', 1),
(2, '视频分类编辑', 1, 1, '/video_categories/edit/\\d+|/video_categories/save/\\d+', 0),
(2, '视频分类删除', 1, 1, '/video_categories/delete/\\d+', 0),

(2, '视频标签', 1, 1, '/video_tags', 1),
(2, '视频标签添加', 1, 1, '/video_tags/edit/\\d+|/videos/save/\\d+', 0),
(2, '视频标签删除', 1, 1, '/video_tags/delete/\\d+', 0),

(2, '视频管理', 1, 1, '/videos', 1),
(2, '视频管理添加', 1, 1, '/videos/edit/\\d+|/videos/save/\\d+', 0),
(2, '视频管理删除', 1, 1, '/videos/delete/\\d+', 0),

(2, '视频评论', 1, 1, '/video_replies', 1),
(2, '视频评论添加', 1, 1, '/video_replies/edit/\\d+|/video_replies/save/\\d+', 0),
(2, '视频评论删除', 1, 1, '/video_replies/delete/\\d+', 0),

(3, '用户列表', 1, 1, '/users', 1),
(3, '用户列表编辑', 1, 1, '/users/edit/\\d+|/users/save/\\d+', 0),
(3, '用户列表删除', 1, 1, '/users/delete/\\d+', 0),

(3, '用户等级', 1, 1, '/user_levels', 1),
(3, '用户等级编辑', 1, 1, '/user_levels/edit/\\d+|/user_levels/save/\\d+', 0),
(3, '用户等级删除', 1, 1, '/user_levels/delete/\\d+', 0),

(3, '观看记录', 1, 1, '/watch_records', 1),
(3, '观看记录', 1, 1, '/watch_records/delete/\\d+', 0),

(2, '广告管理', 1, 1, '/ads', 1),
(2, '广告管理编辑', 1, 1, '/ads/edit/\\d+|/ads/save/\\d+', 0),
(2, '广告管理删除', 1, 1, '/ads/delete/\\d+', 0);


/** 角色管理 **/
DROP TABLE IF EXISTS admin_roles;
CREATE TABLE IF NOT EXISTS admin_roles (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    name VARCHAR(20) NOT NULL DEFAULT '' COMMENT '角色名称',
    remark VARCHAR(50) NOT NULL DEFAULT '' COMMENT '备注',
    menu_ids TEXT COMMENT '菜单编号',
    seq INT NOT NULL DEFAULT 0 COMMENT '排序',
    PRIMARY KEY(id)
) ENGINE=INNODB DEFAULT CHARSET=UTF8 COLLATE=UTF8_GENERAL_CI;
INSERT INTO admin_roles (name, remark, menu_ids) VALUES 
('系统管理员', '后台用户管理', '1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,33,34,35,25,26,27,28,29,30,31,32');

/** 前台用户 **/
DROP TABLE IF EXISTS users;
CREATE TABLE IF NOT EXISTS users (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    name VARCHAR(20) NOT NULL DEFAULT '' COMMENT '用户名称',
    password CHAR(32) NOT NULL DEFAULT '' COMMENT '登錄密碼',
    secret CHAR(32) NOT NULL DEFAULT '' COMMENT '用户密钥',
    mail VARCHAR(100) NOT NULL DEFAULT '' COMMENT '电子邮件',
    last_ip VARCHAR(32) NOT NULL DEFAULT '' COMMENT '最後IP',
    state TINYINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '狀態',
    login_count INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '登錄次數',
    level_id INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '等级编号',
    score INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '积分',
    last_login INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '上次登錄時間',
    created INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '創建時間',
    updated INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '修改時間',
    remark VARCHAR(200) NOT NULL DEFAULT '' COMMENT '备注',
    PRIMARY KEY(id),
    INDEX(name)
) ENGINE=INNODB DEFAULT CHARSET=UTF8 COLLATE=UTF8_GENERAL_CI;
INSERT INTO users (name, password, mail, created, updated) VALUES 
('user', md5('qwe123'), 'user@gmail.com', UNIX_TIMESTAMP(), UNIX_TIMESTAMP());

/** 用户等级 **/
DROP TABLE IF EXISTS user_levels;
CREATE TABLE IF NOT EXISTS user_levels (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    name VARCHAR(20) NOT NULL DEFAULT '' COMMENT '等级名称',
    remark VARCHAR(100) NOT NULL DEFAULT '' COMMENT '备注',
    watch_per_day INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '每天观数',
    score_min INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '最低积分',
    score_max INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '最高积分',
    seq INT NOT NULL DEFAULT 0 COMMENT '排序',
    PRIMARY KEY(id)
) ENGINE=INNODB DEFAULT CHARSET=UTF8 COLLATE=UTF8_GENERAL_CI;
INSERT INTO user_levels (name, remark) VALUES 
('VIP', '基本级VIP'),
('青铜VIP', '高等级VIP'),
('白银VIP', '高等级VIP'),
('黄金VIP', '高等级VIP'),
('钻石VIP', '高等级VIP');

/** 观看记录 **/
DROP TABLE IF EXISTS watch_records;
CREATE TABLE IF NOT EXISTS watch_records (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    user_id INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '用户编号',
    user_name VARCHAR(20) NOT NULL DEFAULT '' COMMENT '用户各称',
    video_id INT UNSIGNED NOT NULL DEFAULt 0 COMMENT '视频编号',
    created INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '創建時間',
    INDEX(user_id),
    INDEX(video_id),
    PRIMARY KEY(id)
) ENGINE=INNODB DEFAULT CHARSET=UTF8 COLLATE=UTF8_GENERAL_CI;
INSERT INTO watch_records (user_id, user_name, video_id, created) VALUES 
(1, 'user', 1, UNIX_TIMESTAMP());

/** 视频分类 **/
DROP TABLE IF EXISTS video_categories;
CREATE TABLE IF NOT EXISTS video_categories (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    name VARCHAR(20) NOT NULL DEFAULT '' COMMENT '名称',
    remark VARCHAR(100) NOT NULL DEFAULT '' COMMENT '备注',
    seq INT NOT NULL DEFAULT 0 COMMENT '排序',
    PRIMARY KEY(id)
) ENGINE=INNODB DEFAULT CHARSET=UTF8 COLLATE=UTF8_GENERAL_CI;
INSERT INTO video_categories (name, remark) VALUES 
('电视剧', '电视剧'),
('电影', '电影');

/** 视频标签 **/
DROP TABLE IF EXISTS video_tags;
CREATE TABLE IF NOT EXISTS video_tags (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    name VARCHAR(20) NOT NULL DEFAULT '' COMMENT '名称',
    remark VARCHAR(100) NOT NULL DEFAULT '' COMMENT '备注',
    seq INT NOT NULL DEFAULT 0 COMMENT '排序',
    PRIMARY KEY(id)
) ENGINE=INNODB ENGINE=INNODB DEFAULT CHARSET=UTF8 COLLATE=UTF8_GENERAL_CI;
INSERT INTO video_tags (name, remark) VALUES 
('国产', '国产'),
('日韩', '日韩'),
('欧美', '欧美');

/** 视频 **/
DROP TABLE IF EXISTS videos;
CREATE TABLE IF NOT EXISTS videos (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    title VARCHAR(50) NOT NULL DEFAULT '' COMMENT '标题',
    remark VARCHAR(100) NOT NULL DEFAULT '' COMMENT '备注',
    cover_image VARCHAR(200) NOT NULL DEFAULT '' COMMENT '封面',
    duration INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '时长(秒)',
    seq INT NOT NULL DEFAULT 0 COMMENT '排序',
    state TINYINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '狀態',
    created INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '創建時間',
    updated INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '修改時間',
    content TExT COMMENT '内容',
    PRIMARY KEY(id)
) ENGINE=INNODB DEFAULT CHARSET=UTF8 COLLATE=UTF8_GENERAL_CI;
INSERT videos (title, remark, duration, created, updated) VALUES 
('头号玩家', '贫民窟小伙逆袭带领大军攻破世界头号游戏公司,迎娶白富美', 210, UNIX_TIMESTAMP(), UNIX_TIMESTAMP());

/** 评论 **/
DROP TABLE IF EXISTS video_replies;
CREATE TABLE IF NOT EXISTS video_replies (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    video_id INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '视频编号',
    reply_id INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '评论编号',
    user_id INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '用户编号',
    user_name VARCHAR(200) NOT NULL DEFAULT '' COMMENT '用户名称',
    content TEXT COMMENT '内容',
    seq INT NOT NULL DEFAULT 0 COMMENT '排序',
    state TINYINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '狀態',
    created INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '創建時間',
    PRIMARY KEY(id)
) ENGINE=INNODB DEFAULT CHARSET=UTF8 COLLATE=UTF8_GENERAL_CI;
INSERT INTO video_replies (user_name, content, created) VALUES 
('user', '可以, 这波666', UNIX_TIMESTAMP());

/** 广告 **/
DROP TABLE IF EXISTS ads;
CREATE TABLE IF NOT EXISTS ads (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    name VARCHAR(20) NOT NULL DEFAULT '' COMMENT '名称',
    remark VARCHAR(100) NOT NULL DEFAULT '' COMMENT '备注',
    image VARCHAR(200) NOT NULL DEFAULT '' COMMENT '图片地址',
    page_id INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '页面,0:首页;1:详情页',
    position_id INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '位置,0:顶部;1:中左;2:中右;3:底部',
    url VARCHAR(200) NOT NULL DEFAULT '' COMMENT '链接地址',
    is_blank TINYINT UNSIGNED NOT NULL DEFAULT 1 COMMENT '是否外链,0:否,1:是',
    seq INT NOT NULL DEFAULT 0 COMMENT '排序',
    PRIMARY KEY(id)
) ENGINE=INNODB DEFAULT CHARSET=UTF8 COLLATE=UTF8_GENERAL_CI;
INSERT INTO ads (name, remark) VALUES 
('脚气灵', '治脚气一抹就灵');
