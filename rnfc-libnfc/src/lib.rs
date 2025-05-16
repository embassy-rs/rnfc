use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::mem::{zeroed, MaybeUninit};
use std::ptr::{null, null_mut};
use std::str::FromStr;

use anyhow::bail;
use log::warn;
use nfc1_sys::{
    nfc_baud_rate_NBR_106, nfc_close, nfc_context, nfc_dep_mode_NDM_PASSIVE, nfc_device, nfc_device_get_name, nfc_exit,
    nfc_init, nfc_initiator_deselect_target, nfc_initiator_init, nfc_initiator_select_dep_target, nfc_open, nfc_target,
    nfc_version,
};
use rnfc_traits::iso_dep::Reader as IsoDepReader;

pub struct Context {
    context: *mut nfc_context,
}

impl Context {
    pub fn new() -> Self {
        let version = unsafe { CStr::from_ptr(nfc_version()) }.to_str().unwrap();
        println!("libnfc v{}", version);

        let mut context: *mut nfc_context = null_mut();
        unsafe { nfc_init(&mut context) };

        Self { context }
    }

    pub fn open(&self, connstring: Option<&str>) -> Result<Device<'_>, anyhow::Error> {
        let connstring = connstring.map(|s| CString::new(s).unwrap());
        let connstring_ptr = connstring.as_ref().map(|s| s.as_ptr()).unwrap_or(null()).cast_mut();
        let device = unsafe { nfc_open(self.context, connstring_ptr) };
        if device.is_null() {
            bail!("Error opening NFC reader");
        }

        Ok(Device {
            context: self.context,
            device,
            _phantom: PhantomData,
        })
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { nfc_exit(self.context) };
    }
}

pub struct Device<'a> {
    context: *mut nfc_context,
    device: *mut nfc_device,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Device<'a> {
    pub fn name(&self) -> String {
        unsafe { CStr::from_ptr(nfc_device_get_name(self.device)) }
            .to_str()
            .unwrap()
            .to_string()
    }

    pub fn as_iso_dep(&mut self) -> Result<IsoDepTag<'_>, anyhow::Error> {
        let mut nt: nfc_target = unsafe { zeroed() };

        let ret = unsafe { nfc_initiator_init(self.device) };
        if ret < 0 {
            warn!("nfc_initiator_init failed")
        }

        let ret = unsafe {
            nfc_initiator_select_dep_target(
                self.device,
                nfc_dep_mode_NDM_PASSIVE,
                nfc_baud_rate_NBR_106,
                null(),
                &mut nt,
                1000,
            )
        };
        if ret < 0 {
            warn!("nfc_initiator_select_dep_target failed")
        }

        Ok(IsoDepTag {
            device: self.device,
            _phantom: PhantomData,
        })
    }
}

impl<'a> Drop for Device<'a> {
    fn drop(&mut self) {
        unsafe { nfc_close(self.device) };
    }
}

pub struct IsoDepTag<'a> {
    device: *mut nfc_device,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Drop for IsoDepTag<'a> {
    fn drop(&mut self) {
        if (unsafe { nfc_initiator_deselect_target(self.device) } < 0) {
            warn!("nfc_initiator_deselect_target failed")
        }
    }
}

impl<'a> IsoDepReader for IsoDepTag<'a> {
    type Error = anyhow::Error;

    async fn transceive(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<usize, Self::Error> {
        todo!()
    }
}

/*



 printf("Sending: %s\n", abtTx);
 int res;
 if ((res = nfc_initiator_transceive_bytes(pnd, abtTx, sizeof(abtTx), abtRx, sizeof(abtRx), 0)) < 0) {
   nfc_perror(pnd, "nfc_initiator_transceive_bytes");
   nfc_close(pnd);
   nfc_exit(context);
   exit(EXIT_FAILURE);
 }

 abtRx[res] = 0;
 printf("Received: %s\n", abtRx);

 if (nfc_initiator_deselect_target(pnd) < 0) {
   nfc_perror(pnd, "nfc_initiator_deselect_target");
   nfc_close(pnd);
   nfc_exit(context);
   exit(EXIT_FAILURE);
 }

*/
