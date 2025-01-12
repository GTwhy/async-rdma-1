use async_rdma::{Rdma, RdmaListener};
use futures::Future;
use tokio::{io, net::ToSocketAddrs};

type RdmaFn<R> = fn(Rdma) -> R;

#[tokio::main]
async fn server<A: ToSocketAddrs, R: Future<Output = Result<(), io::Error>>>(
    addr: A,
    f: RdmaFn<R>,
) -> io::Result<()> {
    let rdma = RdmaListener::bind(addr).await?.accept().await?;
    f(rdma).await
}

#[tokio::main]
async fn client<A: ToSocketAddrs, R: Future<Output = Result<(), io::Error>>>(
    addr: A,
    f: RdmaFn<R>,
) -> io::Result<()> {
    let rdma = Rdma::connect(addr).await?;
    f(rdma).await
}

fn test_server_client<
    A: 'static + ToSocketAddrs + Send + Copy,
    SR: Future<Output = Result<(), io::Error>> + 'static,
    CR: Future<Output = Result<(), io::Error>> + 'static,
>(
    addr: A,
    s: RdmaFn<SR>,
    c: RdmaFn<CR>,
) -> io::Result<()> {
    let server = std::thread::spawn(move || server(addr, s));
    std::thread::sleep(std::time::Duration::from_secs(1));
    let client = std::thread::spawn(move || client(addr, c));
    client.join().unwrap()?;
    server.join().unwrap()
}

mod test1 {
    use crate::*;
    use std::{alloc::Layout, sync::Arc};

    async fn server(rdma: Rdma) -> io::Result<()> {
        let mr = rdma.receive_local_mr().await.unwrap();
        dbg!(unsafe { *(mr.as_ptr() as *mut i32) });
        Ok(())
    }

    async fn client(rdma: Rdma) -> io::Result<()> {
        let rmr = Arc::new(rdma.alloc_remote_mr(Layout::new::<i32>()).await.unwrap());
        let lmr = rdma.alloc_local_mr(Layout::new::<i32>()).unwrap();
        unsafe { *(lmr.as_ptr() as *mut i32) = 5 };
        rdma.write(&lmr, rmr.as_ref()).await.unwrap();
        rdma.send_mr(rmr.clone()).await.unwrap();
        Ok(())
    }

    #[test]
    fn test() -> io::Result<()> {
        test_server_client("127.0.0.1:8000", server, client)
    }
}
