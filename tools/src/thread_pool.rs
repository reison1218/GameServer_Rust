use log::error;
use rand::Rng;
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

#[derive(Debug)]
pub enum ThreadIndex {
    Rankdom,
    Index(usize),
}

pub(crate) type Thunk<'a> = Box<dyn FnBox + Send + 'a>;

pub struct ThreadWorkPool {
    pool: Vec<ThreadWork>,
}

impl ThreadWorkPool {
    pub fn new(name: &str, thread_count: usize) -> Self {
        // let mut threads: [ThreadWork<F>; thread_count] = [ThreadWork; thread_count];

        let mut v = std::vec::Vec::with_capacity(thread_count);
        for i in 0..thread_count {
            let s = format!("{}-{}", name, i + 1);
            let res: ThreadWork = ThreadWork::new(s.as_str());
            v.push(res);
        }

        ThreadWorkPool { pool: v }
    }

    pub fn execute<F>(&self, thread_index: ThreadIndex, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let index = match thread_index {
            ThreadIndex::Rankdom => {
                let mut rand = rand::thread_rng();
                let index = rand.gen_range(0..self.pool.len());
                index
            }
            ThreadIndex::Index(index) => index,
        };

        let res = self.pool.get(index);

        match res {
            Some(thread) => thread.execute(Box::new(job)),
            None => error!("there is no Thread for index {:?}", thread_index),
        }
    }
}

struct ThreadWork {
    sender: crossbeam::channel::Sender<Thunk<'static>>,
}
impl ThreadWork {
    pub(crate) fn new(name: &str) -> Self {
        let (sender, rec) = crossbeam::channel::unbounded();
        let tw = ThreadWork { sender };
        std::thread::Builder::new()
            .name(name.to_owned())
            .spawn(move || loop {
                let job = rec.recv();
                match job {
                    Ok(job) => job.call_box(),
                    Err(e) => error!("{:?}", e),
                }
            })
            .expect("build thread failed!");
        tw
    }

    pub(crate) fn execute(&self, job: Thunk<'static>) {
        self.sender.send(job).unwrap();
    }
}

pub(crate) trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}
