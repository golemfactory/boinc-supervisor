/*
Copyright 2020 Golem Factory <contact@golem.network>

This file is part of boinc-supervisor.

boinc-supervisor is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

boinc-supervisor is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with boinc-supervisor.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::ffi::{CStr, CString};

pub const MSG_CHANNEL_SIZE: usize = 1024;

type ChannelBuf = [u8; MSG_CHANNEL_SIZE];

pub struct MsgChannel<'a> {
    buf: &'a mut ChannelBuf,
}

impl<'a> From<&'a mut ChannelBuf> for MsgChannel<'a> {
    fn from(buf: &'a mut ChannelBuf) -> Self {
        MsgChannel { buf }
    }
}

impl MsgChannel<'_> {
    pub fn has_msg(&self) -> bool {
        self.buf[0] != 0
    }

    pub fn get_msg(&mut self) -> Option<Result<String, String>> {
        if !self.has_msg() {
            return None;
        }
        let buf = &self.buf[1..];
        let len: usize = match &buf.iter().position(|&x| x == b'\0') {
            Some(len) => len + 1, // include the null byte
            None => return Some(Err("message is not null-terminated".to_string())),
        };
        let result = Some(
            CStr::from_bytes_with_nul(&buf[..len])
                .map_err(|err| err.to_string())
                .and_then(|cstr| {
                    CString::from(cstr)
                        .into_string()
                        .map_err(|err| err.to_string())
                }),
        );
        self.buf[0] = 0;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_message() {
        let mut buf: ChannelBuf = [b'\0'; MSG_CHANNEL_SIZE];
        let mut channel = MsgChannel::from(&mut buf);
        assert!(!channel.has_msg());
        assert_eq!(channel.get_msg(), None);
    }

    #[test]
    fn no_nulls() {
        let mut buf: ChannelBuf = [b'\x01'; MSG_CHANNEL_SIZE];
        let mut channel = MsgChannel::from(&mut buf);
        assert!(channel.has_msg());
        let msg = channel.get_msg();
        assert!(msg.is_some());
        assert!(msg.unwrap().is_err());
    }

    #[test]
    fn has_message() {
        let mut buf: ChannelBuf = [b'\0'; MSG_CHANNEL_SIZE];
        buf[0] = b'\x01';
        buf[1] = b'a';
        buf[2] = b'b';
        buf[3] = b'c';
        let mut channel = MsgChannel::from(&mut buf);
        assert!(channel.has_msg());
        {
            let msg = channel.get_msg();
            assert!(msg.is_some());
            let msg = msg.unwrap();
            assert!(msg.is_ok());
            assert_eq!(msg.unwrap(), "abc");
        }
        assert!(!channel.has_msg());
    }
}
