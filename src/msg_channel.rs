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

use anyhow::{anyhow, Result};

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

    pub fn get_msg(&mut self) -> Option<Result<String>> {
        if !self.has_msg() {
            return None;
        }
        let buf = &self.buf[1..];
        let len: usize = match &buf.iter().position(|&x| x == b'\0') {
            Some(len) => len + 1, // include the null byte
            None => return Some(Err(anyhow!("message is not null-terminated"))),
        };
        let result = Some(|| -> Result<String> {
            Ok(std::str::from_utf8(&buf[..len - 1])?.to_string())
        }());
        self.buf[0] = 0;
        result
    }

    pub fn send_msg_overwrite(&mut self, msg: &str) -> Result<()> {
        if msg.len() > MSG_CHANNEL_SIZE - 1 {
            return Err(anyhow!("message too long"));
        }
        self.buf[0] = 1;
        self.buf[1..msg.len() + 1].copy_from_slice(msg.as_bytes());
        self.buf[msg.len() + 1] = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use arrayref::array_ref;

    use super::*;

    #[test]
    fn no_message() {
        let mut buf: ChannelBuf = [0; MSG_CHANNEL_SIZE];
        let mut channel = MsgChannel::from(&mut buf);
        assert!(!channel.has_msg());
        assert!(channel.get_msg().is_none());
    }

    #[test]
    fn no_nulls() {
        let mut buf: ChannelBuf = [1; MSG_CHANNEL_SIZE];
        let mut channel = MsgChannel::from(&mut buf);
        assert!(channel.has_msg());
        let msg = channel.get_msg();
        assert!(msg.is_some());
        assert!(msg.unwrap().is_err());
    }

    #[test]
    fn has_message() {
        let mut buf: ChannelBuf = [0; MSG_CHANNEL_SIZE];
        buf[0] = 1;
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

    #[test]
    fn send_msg_overwrite() {
        let mut buf: ChannelBuf = [1; MSG_CHANNEL_SIZE];
        let mut channel = MsgChannel::from(&mut buf);
        channel.send_msg_overwrite("abc").unwrap();
        // assert_eq!(buf[..5], b"\x01abc\0");
        assert_eq!(array_ref![buf, 0, 6], b"\x01abc\0\x01");
    }
}
