#![allow(unexpected_cfgs)]

use core::ptr::NonNull;
use std::cell::OnceCell;
use std::fmt::Display;

use async_channel::{Receiver, Sender};
use block2::RcBlock;
use dispatch2::{DispatchQoS, DispatchQueue, GlobalQueueIdentifier};
use log::{debug, info};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{AnyThread, DefinedClass, define_class, msg_send};
use objc2_core_nfc::{
    NFCISO7816APDU, NFCISO7816Tag, NFCMiFareFamily, NFCMiFareTag, NFCPollingOption, NFCReaderSession, NFCReaderSessionProtocol,
    NFCTag, NFCTagReaderSession, NFCTagReaderSessionDelegate, NFCTagType,
};
use objc2_foundation::{NSArray, NSData, NSError, NSObject, NSObjectProtocol, NSThread};
use rnfc_traits::iso_dep::Reader as IsoDepReader;

#[derive(Clone, Debug)]
pub enum ReaderError {
    NfcNotSupported,
    TypeNotSupported,
    ConnectFailed,
    InvalidData,
    CommandFailed,
    BufferTooSmall,
    Inactive,
    Internal,
}

impl Display for ReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// An NFC reader.
pub struct Reader {
    session: Retained<NFCTagReaderSession>,
    _delegate: Retained<SessionDelegate>,
    events: Receiver<NFCReaderEvent>,
}

impl Reader {
    /// Create a new instance of the NFC reader.
    pub async fn new() -> Result<Self, ReaderError> {
        if !unsafe { NFCReaderSession::readingAvailable() } {
            return Err(ReaderError::NfcNotSupported);
        }

        let queue = DispatchQueue::global_queue(GlobalQueueIdentifier::QualityOfService(DispatchQoS::Utility));
        let a = NFCTagReaderSession::alloc();

        let (sender, receiver) = async_channel::unbounded();

        let delegate = SessionDelegate::new(sender);
        let object = ProtocolObject::from_ref(&*delegate);
        let session = unsafe {
            NFCTagReaderSession::initWithPollingOption_delegate_queue(
                a,
                NFCPollingOption::ISO14443,
                object,
                Some(queue.as_ref()),
            )
        };

        let mut reader = Reader {
            session,
            _delegate: delegate,
            events: receiver,
        };
        reader.start().await?;
        Ok(reader)
    }

    /// Poll until the reader has detected a tag.
    pub async fn poll(&mut self) -> Result<Tag, ReaderError> {
        loop {
            unsafe { self.session.restartPolling() }
            match self.events.recv().await {
                Ok(NFCReaderEvent::TagsDetected { tags }) => {
                    for tag in tags {
                        if let Some(t) = unsafe { tag.asNFCISO7816Tag() } {
                            let uid = unsafe { t.identifier().to_vec() };
                            return Ok(Tag {
                                session: self.session.clone(),
                                uid,
                                tag,
                            });
                        } else if let Some(t) = unsafe { tag.asNFCMiFareTag() } {
                            let uid = unsafe { t.identifier().to_vec() };
                            return Ok(Tag {
                                session: self.session.clone(),
                                uid,
                                tag,
                            });
                        }
                    }
                }
                Ok(NFCReaderEvent::SessionInactive) => {
                    return Err(ReaderError::Inactive);
                }
                Ok(NFCReaderEvent::SessionActive) => {
                    panic!("unexpected session active event");
                }
                Err(_) => return Err(ReaderError::Internal),
            }
        }
    }

    async fn start(&mut self) -> Result<(), ReaderError> {
        unsafe { self.session.beginSession() }
        loop {
            match self.events.recv().await {
                Ok(NFCReaderEvent::SessionActive) => break,
                Err(_) => return Err(ReaderError::Internal),
                _ => {}
            }
        }
        Ok(())
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        unsafe { self.session.invalidateSession() }
        loop {
            match self.events.try_recv() {
                Ok(NFCReaderEvent::SessionInactive) => break,
                _ => {}
            }
            unsafe { NSThread::sleepForTimeInterval(0.5) }
        }
    }
}

/// An NFC tag that was found by the reader.
pub struct Tag {
    session: Retained<NFCTagReaderSession>,
    tag: Retained<ProtocolObject<dyn NFCTag>>,
    uid: Vec<u8>,
}

impl Tag {
    /// Get the UID of the NFC tag.
    pub fn uid(&self) -> Vec<u8> {
        self.uid.clone()
    }

    /// Check that the Tag is compatible with ISO7816 and return a type that
    /// can be used to perform commands.
    pub async fn as_iso_dep(&mut self) -> Result<IsoDepTag, ReaderError> {
        let t = unsafe { self.tag.r#type() };
        if !(t == NFCTagType::ISO7816Compatible || t == NFCTagType::MiFare) {
            return Err(ReaderError::TypeNotSupported);
        }

        let (s, mut r) = async_broadcast::broadcast(1);
        let completion = RcBlock::new(move |e: *mut NSError| {
            if e.is_null() {
                info!("connect succeeded!");
                s.try_broadcast(true).unwrap();
            } else {
                info!("connect failed!");
                s.try_broadcast(false).unwrap();
            }
        });
        unsafe { self.session.connectToTag_completionHandler(&self.tag, &completion) };
        let Ok(true) = r.recv().await else {
            return Err(ReaderError::ConnectFailed);
        };

        Ok(IsoDepTag { tag: self.tag.clone() })
    }
}

pub struct IsoDepTag {
    tag: Retained<ProtocolObject<dyn NFCTag>>,
}

impl IsoDepReader for IsoDepTag {
    type Error = ReaderError;

    async fn transceive(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<usize, Self::Error> {
        let data = NSData::with_bytes(tx);
        let apdu = unsafe { NFCISO7816APDU::initWithData(NFCISO7816APDU::alloc(), &data).ok_or(ReaderError::InvalidData)? };
        let (s, mut r) = async_broadcast::broadcast(1);
        let completion = RcBlock::new(move |data: NonNull<NSData>, sw1: u8, sw2: u8, e: *mut NSError| {
            let data: &NSData = unsafe { data.as_ref() };
            if e.is_null() {
                let mut data = data.to_vec();
                data.push(sw1);
                data.push(sw2);
                s.try_broadcast(Ok(data)).unwrap();
            } else {
                s.try_broadcast(Err(())).unwrap();
            }
        });
        if let Some(t) = unsafe { self.tag.asNFCISO7816Tag() } {
            unsafe { t.sendCommandAPDU_completionHandler(&apdu, &completion) };
        } else if let Some(t) = unsafe { self.tag.asNFCMiFareTag() } {
            let family = unsafe { t.mifareFamily() };
            let family = match family {
                NFCMiFareFamily::Ultralight => "ultralight",
                NFCMiFareFamily::Plus => "plus",
                NFCMiFareFamily::DESFire => "desfire",
                _ => "unknown",
            };
            debug!("mifare family: {:?}", family);
            unsafe { t.sendMiFareISO7816Command_completionHandler(&apdu, &completion) };
        }

        let Ok(Ok(data)) = r.recv().await else {
            return Err(ReaderError::CommandFailed);
        };

        if rx.len() < data.len() {
            return Err(ReaderError::BufferTooSmall);
        }
        rx[..data.len()].copy_from_slice(&data);
        Ok(data.len())
    }
}

#[derive(Debug, Default)]
struct SessionDelegateIvars {
    sender: OnceCell<Sender<NFCReaderEvent>>,
}

define_class!(
    // SAFETY:
    // - The superclass NSObject does not have any subclassing requirements.
    // - `AppDelegate` does not implement `Drop`.
    #[unsafe(super(NSObject))]
    #[ivars = SessionDelegateIvars]
    struct SessionDelegate;

    unsafe impl NSObjectProtocol for SessionDelegate {}

    unsafe impl NFCTagReaderSessionDelegate for SessionDelegate {
        #[unsafe(method(tagReaderSessionDidBecomeActive:))]
        fn on_session_active(&self, _session: &NFCTagReaderSession) {
            info!("on session active");
            self.ivars().sender.get().map(|s| s.try_send(NFCReaderEvent::SessionActive));
        }

        #[unsafe(method(tagReaderSession:didInvalidateWithError:))]
        fn on_session_inactive(&self, _session: &NFCTagReaderSession, _error: &NSError) {
            info!("on session inactive");
            self.ivars().sender.get().map(|s| s.try_send(NFCReaderEvent::SessionInactive));
        }

        #[unsafe(method(tagReaderSession:didDetectTags:))]
        fn on_tags_detected(&self, _session: &NFCTagReaderSession, tags: &NSArray<ProtocolObject<dyn NFCTag>>) {
            info!("on tags detected");
            let tags = tags.to_vec();
            self.ivars()
                .sender
                .get()
                .map(|s| s.try_send(NFCReaderEvent::TagsDetected { tags }));
        }
    }
);

impl SessionDelegate {
    pub fn new(sender: Sender<NFCReaderEvent>) -> Retained<Self> {
        let cell = OnceCell::new();
        cell.set(sender).unwrap();
        let this = Self::alloc().set_ivars(SessionDelegateIvars { sender: cell });
        unsafe { msg_send![super(this), init] }
    }
}

#[derive(Clone)]
pub enum NFCReaderEvent {
    SessionActive,
    SessionInactive,
    TagsDetected {
        tags: Vec<Retained<ProtocolObject<dyn NFCTag>>>,
    },
}

impl std::error::Error for ReaderError {}
