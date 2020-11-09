use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::prelude::*;

#[tokio::main]
 pub async fn test_tokio_server() -> io::Result<()> {
    let  mut listener = TcpListener::bind("127.0.0.1:8080").await?;
    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Write the data back
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}

 pub async fn test_tokio_client(){
    let mut ts = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    let mut buf = [0; 1024];
    loop{
        let res = ts.read(&mut buf);
    }
}