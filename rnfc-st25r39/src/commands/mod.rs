#[cfg(feature = "st25r3916")]
pub mod cmds_st25r3916;
#[cfg(feature = "st25r3916")]
pub use cmds_st25r3916::Command;

#[cfg(feature = "st25r3911b")]
pub mod cmds_st25r3911b;
#[cfg(feature = "st25r3911b")]
pub use cmds_st25r3911b::Command;

#[cfg(all(not(feature = "st25r3911b"), not(feature = "st25r3916")))]
mod stub;
#[cfg(all(not(feature = "st25r3911b"), not(feature = "st25r3916")))]
pub use stub::Command;
