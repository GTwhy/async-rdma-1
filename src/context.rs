use crate::*;
use rdma_sys::ibv_get_device_name;
use std::{ffi::CStr, io};

pub struct Context {
    pub(super) inner_ctx: *mut rdma_sys::ibv_context,
    pub(super) inner_port_attr: rdma_sys::ibv_port_attr,
    pub(super) gid: Gid,
}

impl Context {
    pub fn open(dev_name: Option<&str>) -> io::Result<Self> {
        let mut num_devs: i32 = 0;
        let dev_list_ptr = unsafe { rdma_sys::ibv_get_device_list(&mut num_devs as *mut _) };
        if dev_list_ptr.is_null() {
            return Err(io::Error::last_os_error());
        }
        let dev_list = unsafe { std::slice::from_raw_parts(dev_list_ptr, num_devs as usize) };
        let dev = if let Some(dev_name) = dev_name {
            dev_list
                .iter()
                .find(|iter_dev| {
                    let name = unsafe { ibv_get_device_name(**iter_dev) };
                    assert!(!name.is_null());
                    let name = unsafe { CStr::from_ptr(name) }.to_str().unwrap();
                    dev_name.eq(name)
                })
                .ok_or(io::ErrorKind::NotFound)?
        } else {
            dev_list.get(0).ok_or(io::ErrorKind::NotFound)?
        };
        let inner_ctx = unsafe { rdma_sys::ibv_open_device(*dev) };
        if inner_ctx.is_null() {
            return Err(io::Error::last_os_error());
        }
        unsafe { rdma_sys::ibv_free_device_list(dev_list_ptr) };
        let mut gid = Gid::default();
        let errno = unsafe { rdma_sys::ibv_query_gid(inner_ctx, 1, 0, gid.as_mut()) };
        if errno != 0 {
            return Err(io::Error::from_raw_os_error(errno));
        }
        let mut inner_port_attr = unsafe { std::mem::zeroed() };
        let errno = unsafe { rdma_sys::___ibv_query_port(inner_ctx, 1, &mut inner_port_attr) };
        if errno != 0 {
            return Err(io::Error::from_raw_os_error(errno));
        }
        Ok(Context {
            inner_ctx,
            inner_port_attr,
            gid,
        })
    }

    pub fn create_event_channel(&self) -> io::Result<EventChannel> {
        let inner_ec = unsafe { rdma_sys::ibv_create_comp_channel(self.inner_ctx) };
        if inner_ec.is_null() {
            return Err(io::Error::last_os_error());
        }
        Ok(EventChannel {
            ctx: self,
            inner_ec,
        })
    }

    pub fn create_completion_queue<'a>(
        &'a self,
        cq_size: u32,
        event_channel: Option<&'a EventChannel>,
    ) -> io::Result<CompletionQueue> {
        CompletionQueue::create(self, cq_size, event_channel)
    }

    pub fn create_protection_domain(&self) -> io::Result<ProtectionDomain> {
        ProtectionDomain::create(self)
    }

    pub fn get_lid(&self) -> u16 {
        self.inner_port_attr.lid
    }

    pub fn get_active_mtu(&self) -> u32 {
        self.inner_port_attr.active_mtu
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        let errno = unsafe { rdma_sys::ibv_close_device(self.inner_ctx) };
        assert_eq!(errno, 0);
    }
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use crate::*;
    #[test]
    fn test1() {
        let ctx = Context::open(Some("rdma"));
    }
}