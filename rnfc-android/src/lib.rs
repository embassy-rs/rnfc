#![feature(arbitrary_self_types)]

mod bindings;
mod errors;

use std::mem::ManuallyDrop;
use std::sync::Arc;

use async_channel::{Receiver, Sender};
use java_spaghetti::sys::{JNIEnv, jobject};
use java_spaghetti::{ByteArray, Env, Global, Local, Null, PrimitiveArray, Ref, VM};
use log::{info, warn};
use rnfc_traits::iso_dep::Reader as IsoDepReader;
use rnfc_traits::iso14443a::Reader as Iso14443aReader;

use crate::bindings::android::app::Activity;
use crate::bindings::android::nfc::tech::{IsoDep, NfcA};
use crate::bindings::android::nfc::{NfcAdapter, NfcAdapter_ReaderCallback, NfcAdapter_ReaderCallbackProxy, Tag as NfcTag};
pub use crate::errors::*;

/// Utility to hold reader mode enabled.
///
/// Disabling reader mode makes the `Tag` objects stop working.
/// Therefore, we make both the reader and the tags hold an Arc of this
/// struct, so reader mode is disabled when both reader and all tags are dropped.
struct ReaderModeHolder {
    vm: VM,
    activity: Global<Activity>,
    adapter: Global<NfcAdapter>,
}

impl Drop for ReaderModeHolder {
    fn drop(&mut self) {
        info!("disabling reader mode");
        self.vm.with_env(|env| {
            let adapter = self.adapter.as_local(env);
            if let Err(e) = adapter.disableReaderMode(&self.activity) {
                warn!("failed disabling reader mode: {e:?}")
            }
        })
    }
}

pub struct Reader<'a> {
    activity: Local<'a, Activity>,
    receiver: Receiver<Global<NfcTag>>,
    holder: Arc<ReaderModeHolder>,
}

impl<'a> Reader<'a> {
    /// SAFETY:
    /// - `env` must be a valid JNIEnv pointer
    /// - `activity` must be a valid object pointer to an instance of `android.app.Activity`
    /// - The current thread must stay attached to the VM for the duration the `Reader` exists.
    pub unsafe fn new(env: *mut JNIEnv, activity: jobject) -> Result<Self, NewReaderError> {
        assert!(!env.is_null());
        assert!(!activity.is_null());
        let env = unsafe { Env::from_raw(env) };
        let activity = unsafe { Ref::from_raw(env, activity).as_local() };

        let env = activity.env();
        let Some(adapter) = NfcAdapter::getDefaultAdapter(env, &activity)? else {
            return Err(NewReaderError::NfcNotSupported);
        };

        let (sender, receiver) = async_channel::bounded(1);
        let callback: Local<NfcAdapter_ReaderCallback> =
            NfcAdapter_ReaderCallback::new_proxy(env, Arc::new(ReaderCallback { sender }))?;
        adapter.enableReaderMode(
            &activity,
            callback,
            NfcAdapter::FLAG_READER_NFC_A | NfcAdapter::FLAG_READER_SKIP_NDEF_CHECK,
            Null,
        )?;

        let holder = Arc::new(ReaderModeHolder {
            activity: activity.as_global(),
            adapter: adapter.as_global(),
            vm: env.vm(),
        });

        Ok(Self {
            activity,
            receiver,
            holder,
        })
    }

    pub async fn poll(&mut self) -> Result<Tag<'a>, AsTechError> {
        let env = self.activity.env();

        let tag = self.receiver.recv().await.unwrap();
        let tag = tag.as_local(env);

        let uid = match tag.getId()? {
            Some(uid) => i8tou8_vec(uid.as_vec()),
            None => Vec::new(),
        };

        Ok(Tag {
            tag,
            uid,
            holder: self.holder.clone(),
        })
    }
}

pub struct Tag<'a> {
    tag: Local<'a, NfcTag>,
    uid: Vec<u8>,
    holder: Arc<ReaderModeHolder>,
}

impl<'a> Tag<'a> {
    pub fn uid(&self) -> Vec<u8> {
        self.uid.clone()
    }

    pub fn as_iso_dep(&mut self) -> Result<IsoDepTag<'_>, AsTechError> {
        let env = self.tag.env();
        let Some(tech) = IsoDep::get(env, &self.tag)? else {
            return Err(AsTechError::TechNotSupported);
        };

        tech.connect()?;

        Ok(IsoDepTag {
            tag: self.tag.clone(),
            uid: self.uid.clone(),
            tech,
        })
    }

    pub fn as_iso14443_a(&mut self) -> Result<Iso14443aTag<'_>, AsTechError> {
        let env = self.tag.env();
        let Some(tech) = NfcA::get(env, &self.tag)? else {
            return Err(AsTechError::TechNotSupported);
        };

        tech.connect()?;

        Ok(Iso14443aTag {
            tag: self.tag.clone(),
            uid: self.uid.clone(),
            tech,
        })
    }

    pub fn into_global(self) -> GlobalTag {
        GlobalTag {
            tag: self.tag.as_global(),
            uid: self.uid,
            holder: self.holder,
        }
    }
}

pub struct GlobalTag {
    tag: Global<NfcTag>,
    uid: Vec<u8>,
    holder: Arc<ReaderModeHolder>,
}

impl GlobalTag {
    /// SAFETY:
    /// - `env` must be a valid JNIEnv pointer
    /// - The current thread must stay attached to the VM for the duration the `Tag` exists.
    pub unsafe fn as_local<'env>(&self, env: *mut JNIEnv) -> Tag<'env> {
        assert!(!env.is_null());
        let env = unsafe { Env::from_raw(env) };

        Tag {
            tag: self.tag.as_local(env),
            uid: self.uid.clone(),
            holder: self.holder.clone(),
        }
    }
}

pub struct Iso14443aTag<'a> {
    tag: Local<'a, NfcTag>,
    tech: Local<'a, NfcA>,
    uid: Vec<u8>,
}

impl<'a> Iso14443aTag<'a> {
    pub fn uid(&self) -> Vec<u8> {
        self.uid.clone()
    }
}

impl<'a> Drop for Iso14443aTag<'a> {
    fn drop(&mut self) {
        if let Err(e) = self.tech.close() {
            warn!("failed closing iso14443a tech: {e:?}")
        }
    }
}

impl<'a> Iso14443aReader for Iso14443aTag<'a> {
    type Error = TransceiveError;

    fn uid(&self) -> &[u8] {
        &self.uid
    }

    fn atqa(&self) -> [u8; 2] {
        todo!()
    }

    fn sak(&self) -> u8 {
        todo!()
    }

    async fn transceive(&mut self, tx: &[u8], rx: &mut [u8], _timeout_1fc: u32) -> Result<usize, Self::Error> {
        let tx = ByteArray::new_from(self.tag.env(), u8toi8(tx));
        let rxd = self.tech.transceive(tx)?.unwrap();
        let rxd = i8tou8_vec(rxd.as_vec());
        if rxd.len() > rx.len() {
            return Err(TransceiveError::BufferTooSmall);
        }
        rx[..rxd.len()].copy_from_slice(&rxd);
        Ok(rxd.len())
    }
}

pub struct IsoDepTag<'a> {
    tag: Local<'a, NfcTag>,
    tech: Local<'a, IsoDep>,
    uid: Vec<u8>,
}

impl<'a> IsoDepTag<'a> {
    pub fn uid(&self) -> Vec<u8> {
        self.uid.clone()
    }
}

impl<'a> Drop for IsoDepTag<'a> {
    fn drop(&mut self) {
        if let Err(e) = self.tech.close() {
            warn!("failed closing isodep tech: {e:?}")
        }
    }
}

impl<'a> IsoDepReader for IsoDepTag<'a> {
    type Error = TransceiveError;

    async fn transceive(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<usize, Self::Error> {
        let tx = ByteArray::new_from(self.tag.env(), u8toi8(tx));
        let rxd = self.tech.transceive(tx)?.unwrap();
        let rxd = i8tou8_vec(rxd.as_vec());
        if rxd.len() > rx.len() {
            return Err(TransceiveError::BufferTooSmall);
        }
        rx[..rxd.len()].copy_from_slice(&rxd);
        Ok(rxd.len())
    }
}

struct ReaderCallback {
    sender: Sender<Global<NfcTag>>,
}

impl NfcAdapter_ReaderCallbackProxy for ReaderCallback {
    fn onTagDiscovered<'env>(&self, _env: Env<'env>, tag: Option<Ref<'env, NfcTag>>) {
        let Some(tag) = tag else {
            warn!("onTagDiscovered got null tag?");
            return;
        };
        if let Err(e) = self.sender.try_send(tag.as_global()) {
            warn!("onTagDiscovered failed to send tag: {e:?}");
        }
    }
}

fn u8toi8(slice: &[u8]) -> &[i8] {
    let len = slice.len();
    let data = slice.as_ptr() as *const i8;
    // safety: any bit pattern is valid for u8 and i8, so transmuting them is fine.
    unsafe { std::slice::from_raw_parts(data, len) }
}

fn i8tou8_vec(v: Vec<i8>) -> Vec<u8> {
    let mut v = ManuallyDrop::new(v);
    let length = v.len();
    let capacity = v.capacity();
    let ptr = v.as_mut_ptr() as *mut u8;
    // safety: any bit pattern is valid for u8 and i8, so transmuting them is fine.
    unsafe { Vec::from_raw_parts(ptr, length, capacity) }
}
