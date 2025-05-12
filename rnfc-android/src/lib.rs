#![feature(arbitrary_self_types)]

use std::fmt::Display;
use std::mem::ManuallyDrop;
use std::sync::Arc;

use async_channel::{Receiver, Sender};
use bindings::android::app::Activity;
use bindings::android::nfc::tech::{IsoDep, NfcA};
use bindings::android::nfc::{NfcAdapter, NfcAdapter_ReaderCallback, NfcAdapter_ReaderCallbackProxy, Tag as NfcTag};
use java_spaghetti::sys::{JNIEnv, jobject};
use java_spaghetti::{ByteArray, Env, Global, Local, Null, PrimitiveArray, Ref};
use log::info;
use rnfc_traits::iso_dep::Reader as IsoDepReader;
use rnfc_traits::iso14443a::{self, Reader as Iso14443aReader};

mod bindings;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PollError {
    TechNotSupported,
}
impl Display for PollError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for PollError {}

pub struct Reader<'a> {
    activity: Local<'a, Activity>,
    adapter: Local<'a, NfcAdapter>,
    receiver: Receiver<Global<NfcTag>>,
}

impl<'a> Reader<'a> {
    pub unsafe fn new(env: *mut JNIEnv, activity: jobject) -> Self {
        assert!(!env.is_null());
        assert!(!activity.is_null());
        let env = unsafe { Env::from_raw(env) };
        let activity = unsafe { Ref::from_raw(env, activity).as_local() };

        let env = activity.env();
        let adapter = NfcAdapter::getDefaultAdapter(env, &activity).unwrap().unwrap();

        let (sender, receiver) = async_channel::bounded(1);
        let callback: Local<NfcAdapter_ReaderCallback> =
            NfcAdapter_ReaderCallback::new_proxy(env, Arc::new(ReaderCallback { sender })).unwrap();
        adapter
            .enableReaderMode(
                &activity,
                callback,
                NfcAdapter::FLAG_READER_NFC_A | NfcAdapter::FLAG_READER_SKIP_NDEF_CHECK,
                Null,
            )
            .unwrap();

        Self {
            activity,
            adapter,
            receiver,
        }
    }

    pub async fn poll(&mut self) -> Result<Tag<'_>, PollError> {
        let env = self.activity.env();

        let tag = self.receiver.recv().await.unwrap();
        let tag = tag.as_local(env);

        let uid = tag.getId().unwrap().unwrap();
        let uid = i8tou8_vec(uid.as_vec());

        Ok(Tag { tag, uid })
    }
}

impl<'a> Drop for Reader<'a> {
    fn drop(&mut self) {
        self.adapter.disableReaderMode(&self.activity).unwrap();
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransceiveError {
    BufferTooSmall,
}
impl Display for TransceiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for TransceiveError {}
impl iso14443a::Error for TransceiveError {
    fn kind(&self) -> rnfc_traits::iso14443a_ll::ErrorKind {
        rnfc_traits::iso14443a_ll::ErrorKind::Other
    }
}

pub struct Tag<'a> {
    tag: Local<'a, NfcTag>,
    uid: Vec<u8>,
}

impl<'a> Tag<'a> {
    pub fn uid(&self) -> Vec<u8> {
        self.uid.clone()
    }

    pub fn as_iso_dep(&mut self) -> Result<IsoDepTag<'_>, PollError> {
        let env = self.tag.env();
        let Some(tech) = IsoDep::get(env, &self.tag).unwrap() else {
            return Err(PollError::TechNotSupported);
        };

        tech.connect().unwrap();

        Ok(IsoDepTag {
            tag: self.tag.clone(),
            uid: self.uid.clone(),
            tech,
        })
    }

    pub fn as_iso14443_a(&mut self) -> Result<Iso14443aTag<'_>, PollError> {
        let env = self.tag.env();
        let Some(tech) = NfcA::get(env, &self.tag).unwrap() else {
            return Err(PollError::TechNotSupported);
        };

        tech.connect().unwrap();

        Ok(Iso14443aTag {
            tag: self.tag.clone(),
            uid: self.uid.clone(),
            tech,
        })
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
        self.tech.close().unwrap();
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
        let rxd = self.tech.transceive(tx).unwrap().unwrap();
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
        self.tech.close().unwrap();
    }
}

impl<'a> IsoDepReader for IsoDepTag<'a> {
    type Error = TransceiveError;

    async fn transceive(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<usize, Self::Error> {
        let tx = ByteArray::new_from(self.tag.env(), u8toi8(tx));
        let rxd = self.tech.transceive(tx).unwrap().unwrap();
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

impl Drop for ReaderCallback {
    fn drop(&mut self) {
        info!("drop lol")
    }
}

impl NfcAdapter_ReaderCallbackProxy for ReaderCallback {
    fn onTagDiscovered<'env>(&self, _env: Env<'env>, tag: Option<Ref<'env, NfcTag>>) {
        self.sender.try_send(tag.unwrap().as_global()).unwrap();
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
