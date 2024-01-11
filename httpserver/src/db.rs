use sqlx::{MySqlPool, Row};

static DB_VERION: i32 = 7;

pub fn check() {
    let pool: &MySqlPool = &crate::POOL;
    let res = async_std::task::block_on(async {
        sqlx::query("show tables").fetch_all(pool).await.unwrap()
    });
    if res.is_empty() {
        log::info!("数据库里不存在表，重新创建所有表");
        create_tables();
    }
    //检查db_version表是否存在，不存在就创建一个
    check_db_version_table();
    let current_db_version = get_db_version();
    log::info!("当前数据库版本：{}", current_db_version);
    if current_db_version == DB_VERION || current_db_version >= DB_VERION {
        return;
    }
    let mut version = current_db_version;
    loop {
        if version >= DB_VERION {
            break;
        }
        check_db_verion(pool);
        match version {
            0 => {
                upgrade_0to1(pool);
            }
            1 => {
                upgrade_1to2(pool);
            }

            2 => {
                upgrade_2to3(pool);
            }
            3 => {
                upgrade_3to4(pool);
            }

            4 => {
                upgrade_4to5(pool);
            }

            5 => {
                upgrade_5to6(pool);
            }

            6 => {
                upgrade_6to7(pool);
            }
            _ => {}
        }
        version += 1;
    }
}

pub fn upgrade_0to1(pool: &MySqlPool) {
    log::info!("升级数据库, 从0号升级到1号");
    let sql0="ALTER TABLE `server_list` ADD COLUMN `server_type` INT DEFAULT 0 NULL COMMENT '是否版署服（0：不是 1是）' AFTER `merge_times`,CHARSET=utf8mb3;";
    let set_version = "update db_version set version=1 where version=0";
    if !check_field("server_list", "server_type", pool) {
        async_std::task::block_on(async {
            sqlx::query(sql0).execute(pool).await.unwrap();
        });
    }

    async_std::task::block_on(async {
        sqlx::query(set_version).execute(pool).await.unwrap();
    });
}

fn upgrade_1to2(pool: &MySqlPool) {
    log::info!("升级数据库, 从1号升级到2号");

    let sql0 = "ALTER TABLE `server_list` ADD COLUMN `manager` VARCHAR(128) NULL COMMENT 'http' AFTER `merge_times`, CHARSET=utf8mb3;";

    let set_version = "update db_version set version=2 where version=1";
    if !check_field("server_list", "manager", pool) {
        async_std::task::block_on(async {
            sqlx::query(sql0).execute(pool).await.unwrap();
        });
    }

    async_std::task::block_on(async {
        sqlx::query(set_version).execute(pool).await.unwrap();
    });
}

fn upgrade_2to3(pool: &MySqlPool) {
    log::info!("升级数据库, 从2号升级到3号");
    let sql0 = "ALTER TABLE `server_list` ADD COLUMN `inner_manager` VARCHAR(128) NULL COMMENT '内网http' AFTER `manager`,CHARSET=utf8mb3;";

    let set_version = "update db_version set version=3 where version=2";
    if !check_field("server_list", "inner_manager", pool) {
        async_std::task::block_on(async {
            sqlx::query(sql0).execute(pool).await.unwrap();
        });
    }

    async_std::task::block_on(async {
        sqlx::query(set_version).execute(pool).await.unwrap();
    });
}

fn upgrade_3to4(pool: &MySqlPool) {
    log::info!("升级数据库, 从3号升级到4号");
    let sql0 = "CREATE TABLE IF NOT EXISTS  `merge_change` ( `reload` INT NOT NULL DEFAULT 0 ) ENGINE=INNODB CHARSET=utf8mb3;";

    let set_version = "update db_version set version=4 where version=3";

    async_std::task::block_on(async {
        sqlx::query(sql0).execute(pool).await.unwrap();
    });

    async_std::task::block_on(async {
        sqlx::query(set_version).execute(pool).await.unwrap();
    });
}

fn upgrade_4to5(pool: &MySqlPool) {
    log::info!("升级数据库，从4号升级到5号");
    let sql0 =
            "ALTER TABLE `server_list` ADD COLUMN `update_merge_times_time` DATETIME DEFAULT NULL COMMENT '修改merge_times的时间' AFTER `merge_times`,CHARSET=utf8mb3;";
    let set_version = "update db_version set version=5 where version=4";
    if !check_field("server_list", "update_merge_times_time", pool) {
        async_std::task::block_on(async {
            sqlx::query(sql0).execute(pool).await.unwrap();
        });
    }

    async_std::task::block_on(async {
        sqlx::query(set_version).execute(pool).await.unwrap();
    });
}

fn upgrade_5to6(pool: &MySqlPool) {
    log::info!("升级数据库，从5号升级到6号");
    let sql0 = "ALTER TABLE `users` ADD COLUMN `level` INT DEFAULT 0 NULL COMMENT '基地等级' AFTER `server_id`, CHARSET=utf8mb3";
    let set_version = "update db_version set version=6 where version=5";
    if !check_field("`users`", "level", pool) {
        async_std::task::block_on(async {
            sqlx::query(sql0).execute(pool).await.unwrap();
        });
    }

    async_std::task::block_on(async {
        sqlx::query(set_version).execute(pool).await.unwrap();
    });
}

fn upgrade_6to7(pool: &MySqlPool) {
    log::info!("升级数据库，从6号升级到7号");
    let sql0 =
        "CREATE TABLE `wx_users_subscribe` (`name` VARCHAR(128) NOT NULL COMMENT '玩家账号',  `open_id` VARCHAR(128) COMMENT '玩家微信open_id', `templ_ids` VARCHAR(1024) COMMENT '玩家订阅的消息膜拜id', PRIMARY KEY (`name`) ) ENGINE=INNODB CHARSET=utf8mb4;";
    let set_version = "update db_version set version=7 where version=6";
    async_std::task::block_on(async {
        sqlx::query(sql0).execute(pool).await.unwrap();
    });

    async_std::task::block_on(async {
        sqlx::query(set_version).execute(pool).await.unwrap();
    });
}

pub fn check_field(table_name: &str, column: &str, pool: &MySqlPool) -> bool {
    let check_sql = format!("SHOW COLUMNS FROM {} LIKE '{}'", table_name, column);
    let row = async_std::task::block_on(async {
        sqlx::query(check_sql.as_str())
            .fetch_all(pool)
            .await
            .unwrap()
    });
    !row.is_empty()
}

pub fn check_db_verion(pool: &MySqlPool) {
    let create_sql ="CREATE TABLE IF NOT EXISTS  `db_version` (`version` int NOT NULL, PRIMARY KEY (`version`)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;";
    let query_sql = "select version from  `db_version`";
    let insert_sql = "insert into `db_version` values(0)";
    async_std::task::block_on(async { sqlx::query(create_sql).execute(pool).await.unwrap() });

    let res =
        async_std::task::block_on(async { sqlx::query(query_sql).fetch_all(pool).await.unwrap() });
    if !res.is_empty() {
        return;
    }
    async_std::task::block_on(async { sqlx::query(insert_sql).fetch_all(pool).await.unwrap() });
}

pub fn check_db_version_table() {
    let pool: &MySqlPool = &crate::POOL;
    let sql = "SELECT * FROM information_schema.TABLES WHERE TABLE_NAME = 'db_version'";
    let create_sql="CREATE TABLE `db_version` (`version` int NOT NULL DEFAULT 0,PRIMARY KEY (`version`)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci";
    let insert_sql = "insert into db_version(version) values(0)";
    let rows = async_std::task::block_on(async { sqlx::query(sql).fetch_all(pool).await.unwrap() });
    if !rows.is_empty() {
        return;
    }
    log::info!("不存在db_version表，现在开始创建");
    async_std::task::block_on(async { sqlx::query(create_sql).fetch_all(pool).await.unwrap() });
    async_std::task::block_on(async { sqlx::query(insert_sql).fetch_all(pool).await.unwrap() });
}

pub fn get_db_version() -> i32 {
    let pool: &MySqlPool = &crate::POOL;
    let sql = "select * from db_version";
    let row = async_std::task::block_on(async { sqlx::query(sql).fetch_all(pool).await.unwrap() });
    for row in row {
        let res: i32 = row.get(0);
        return res;
    }
    0
}

pub fn create_tables() {
    let pool: &MySqlPool = &crate::POOL;
    let tx = async_std::task::block_on(async { pool.begin().await });

    let sql = "DROP TABLE IF EXISTS `db_version`;";
    async_std::task::block_on(async {
        sqlx::query(sql).execute(pool).await.unwrap();
    });

    let sql = "CREATE TABLE `db_version` (`version` int NOT NULL DEFAULT '0',PRIMARY KEY (`version`)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;";
    async_std::task::block_on(async {
        sqlx::query(sql).execute(pool).await.unwrap();
    });

    let sql = "insert into db_version(version) values(0);";
    async_std::task::block_on(async {
        sqlx::query(sql).execute(pool).await.unwrap();
    });

    let sql = "DROP TABLE IF EXISTS `server_list`;";
    async_std::task::block_on(async {
        sqlx::query(sql).execute(pool).await.unwrap();
    });

    let sql = "CREATE TABLE `server_list` (`server_id` int NOT NULL COMMENT '服务器id',`name` varchar(128) DEFAULT NULL COMMENT '服务器名字',`ws` varchar(128) DEFAULT NULL COMMENT 'ws连接',`open_time` datetime DEFAULT NULL COMMENT '开服时间',`register_state` int DEFAULT NULL COMMENT '0超过N天不可注册1可注册',`state` int DEFAULT NULL COMMENT '0:正常开服状态 4：停服维护状态',`letter` int DEFAULT NULL COMMENT '0正常1强行推荐',`target_server_id` int DEFAULT NULL COMMENT '目标服id',`merge_times` int DEFAULT NULL COMMENT '第几次合服',`type` varchar(128) CHARACTER SET utf8mb3 COLLATE utf8_general_ci DEFAULT NULL COMMENT '类型：0:开发服 1：测试服  2：内测服 10：正式服',PRIMARY KEY (`server_id`)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;";
    async_std::task::block_on(async {
        sqlx::query(sql).execute(pool).await.unwrap();
    });

    let sql = "DROP TABLE IF EXISTS `users`;";
    async_std::task::block_on(async {
        sqlx::query(sql).execute(pool).await.unwrap();
    });

    let sql = "CREATE TABLE `users` (`id` int NOT NULL AUTO_INCREMENT,`name` char(50) NOT NULL,`combine_id` bigint NOT NULL,`operator_id` int NOT NULL,`server_id` int NOT NULL,`player_name` char(50) DEFAULT NULL,`login_time` bigint DEFAULT NULL COMMENT '最近登录时间',PRIMARY KEY (`id`,`server_id`,`operator_id`),UNIQUE KEY `combine` (`combine_id`),UNIQUE KEY `name` (`name`,`server_id`,`operator_id`) USING BTREE,UNIQUE KEY `player_name` (`server_id`,`operator_id`,`player_name`) USING BTREE) ENGINE=InnoDB AUTO_INCREMENT=19811 DEFAULT CHARSET=utf8mb3;";
    async_std::task::block_on(async {
        sqlx::query(sql).execute(pool).await.unwrap();
    });

    let sql = "DROP TABLE IF EXISTS `white_users`;";
    async_std::task::block_on(async {
        sqlx::query(sql).execute(pool).await.unwrap();
    });

    let sql = "CREATE TABLE `white_users` (`name` varchar(128) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '玩家账号',PRIMARY KEY (`name`)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;";
    async_std::task::block_on(async {
        sqlx::query(sql).execute(pool).await.unwrap();
    });

    async_std::task::block_on(async {
        tx.unwrap().commit().await.unwrap();
    });
}
