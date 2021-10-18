use std::ptr::slice_from_raw_parts;

use serde::{de::DeserializeOwned, Serialize};

/// FFiVec should be used for communication between [super::ScriptType::DynamicLib] scripts and the main program
#[repr(C)]
pub struct FFiVec {
    ptr: *mut u8,
    len: usize,
    cap: usize,
}
impl FFiVec {
    /// Crate a new FFiVec from any serialize-able data
    pub fn serialize_from<D: Serialize>(data: &D) -> Result<Self, bincode::Error> {
        let data = bincode::serialize(data)?;
        let mut vec = std::mem::ManuallyDrop::new(data);
        let ptr = vec.as_mut_ptr();
        let len = vec.len();
        let cap = vec.capacity();
        Ok(FFiVec { ptr, len, cap })
    }
    /// De-serialize into a concrete type
    pub fn deserialize<D: DeserializeOwned>(&self) -> Result<D, bincode::Error> {
        let data: &[u8] = unsafe { &*slice_from_raw_parts(self.ptr, self.len) };
        bincode::deserialize(data)
    }
}
impl Drop for FFiVec {
    fn drop(&mut self) {
        let _ = unsafe { Vec::from_raw_parts(self.ptr, self.len, self.cap) };
    }
}
