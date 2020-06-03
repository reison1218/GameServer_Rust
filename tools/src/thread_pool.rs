use threadpool::ThreadPool;

//线程池类型枚举
pub enum ThreadPoolType {
    Game = 1,
    User = 2,
    Sys = 3,
}

//线程池结构体封装
pub struct MyThreadPool {
    game_pool: ThreadPool, //游戏线程池
    user_pool: ThreadPool, //用户线程池
    sys_pool: ThreadPool,  //系统线程池
}

unsafe impl Sync for MyThreadPool {}

pub trait ThreadPoolHandler {
    fn submit_game<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;

    fn submit_user<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;

    fn submit_sys<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;

    fn submit<F>(&self, pool_type: ThreadPoolType, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        match pool_type {
            ThreadPoolType::Game => self.submit_game(job),
            ThreadPoolType::User => self.submit_user(job),
            ThreadPoolType::Sys => self.submit_sys(job),
        }
    }
}

impl ThreadPoolHandler for MyThreadPool {
    fn submit_game<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.game_pool.execute(job);
    }

    fn submit_user<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.user_pool.execute(job);
    }

    fn submit_sys<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sys_pool.execute(job);
    }
}

impl MyThreadPool {
    ///初始化线程池结构体
    pub fn init(
        game_name: String,
        game_size: usize,
        user_name: String,
        user_size: usize,
        sys_name: String,
        sys_size: usize,
    ) -> Self {
        let gtp = ThreadPool::with_name(game_name, game_size);
        let utp = ThreadPool::with_name(user_name, user_size);
        let stp = ThreadPool::with_name(sys_name, sys_size);
        let mtp = MyThreadPool {
            game_pool: gtp,
            user_pool: utp,
            sys_pool: stp,
        };
        mtp
    }
}
