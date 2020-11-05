use futures::join;
pub async  fn learn_and_sing() {
    // 要唱歌必须得先学会歌曲.
    // 我们这里使用 `.await` 而不是 `block_on` 来
    // 防止线程阻塞, 这样也可以同时跳舞.
    let song = learn_song().await;
    sing_song(song).await;
}

pub async fn learn_song()->String{
    println!("learn_song");
    std::thread::park();
    "learn_song".to_owned()
}

pub async fn sing_song(str:String){
    println!("sing_song");
}

pub async fn dance(){
    println!("dance");
}






pub async fn async_main() {

    let res = async{
        println!("ttt");
    };
    async_std::task::block_on(res);
    // let f1 = learn_and_sing();
    // let f2 = dance();
    //
    // // `join!` 类似 `.await`，但是可以同时等待多个 `future` 执行完成.
    // // 如果我们 `learn_and_sing` 这个 `future` 被阻塞, 那么 `dance`
    // // 这个 `future` 将接管当前的线程. 如果 `dance` 被阻塞, 那么 `learn_and_sing`
    // // 就可以重新开始. 如果这个两个 `future` 都被阻塞, 那么 `async_main`
    // // 也将被阻塞并让位给执行程序.
    // join!(f1, f2);

    // std::task::Waker::drop()
}


