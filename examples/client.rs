use async_rdma::Rdma;
use log::debug;
use std::{alloc::Layout, sync::Arc, time::Duration};

async fn example1(rdma: &Rdma) {
    let rmr = Arc::new(rdma.alloc_remote_mr(Layout::new::<i32>()).await.unwrap());
    let mut lmr = rdma.alloc_local_mr(Layout::new::<i32>()).unwrap();
    unsafe { *(lmr.as_mut_ptr() as *mut i32) = 5 };
    rdma.write(&lmr, rmr.as_ref()).await.unwrap();
    debug!("e1 write");
    tokio::time::sleep(Duration::new(1, 0)).await;
    rdma.send_mr(rmr.clone()).await.unwrap();
    debug!("e1 send");
}

async fn example2(rdma: &Rdma) {
    let mut lmr = Arc::new(rdma.alloc_local_mr(Layout::new::<i32>()).unwrap());
    unsafe { *(Arc::get_mut(&mut lmr).unwrap().as_mut_ptr() as *mut i32) = 55 };
    tokio::time::sleep(Duration::new(1, 0)).await;
    rdma.send_mr(lmr.clone()).await.unwrap();
    debug!("e2 send");
}

async fn example3(rdma: &Rdma) {
    let mut lmr = Arc::new(rdma.alloc_local_mr(Layout::new::<i32>()).unwrap());
    unsafe { *(Arc::get_mut(&mut lmr).unwrap().as_mut_ptr() as *mut i32) = 555 };
    // std::thread::sleep(Duration::from_micros(10));
    tokio::time::sleep(Duration::new(1, 0)).await;
    rdma.send(lmr.as_ref()).await.unwrap();
    debug!("e3 send");
}

#[tokio::main]
async fn main() {
    env_logger::init();
    debug!("client start");
    let rdma = Rdma::connect("127.0.0.1:5555").await.unwrap();
    example1(&rdma).await;
    example2(&rdma).await;
    example3(&rdma).await;
    println!("client done");
    loop {
        tokio::time::sleep(Duration::new(100, 0)).await;
    }
}
