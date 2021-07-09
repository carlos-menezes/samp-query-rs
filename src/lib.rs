use tokio::time::Instant;
use tokio::time::timeout_at;
use std::{net::{Ipv4Addr}, time::Duration};
use tokio::net::{UdpSocket};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

#[derive(Debug)]
pub struct InfoPacket {
  pub password: bool,
  pub players: u16,
  pub max_players: u16,
  pub hostname: String,
  pub gamemode: String,
  pub language: String,
}

impl Default for InfoPacket {
  fn default() -> Self {
    Self { password: false, players: 0, max_players: 0, hostname: String::new(), gamemode: String::new(), language: String::new() }
  }
}

pub struct Query {
    address: Ipv4Addr,
    port: i32,
    socket: UdpSocket,
}

impl Query {
  pub async fn new(addr: &str, port: i32) -> Result<Self, std::io::Error> {
    let data = Self {
        address: addr.parse::<Ipv4Addr>().unwrap(),
        port,
        socket: UdpSocket::bind("0.0.0.0:0").await?,
    };
    data.socket.connect(format!("{}:{}", addr, port)).await?;
    Ok(data)
  }

  pub async fn send(&self) -> Result<usize, std::io::Error> {
    let mut packet: Vec<u8> = Vec::new();
    packet.append(&mut "SAMP".to_owned().into_bytes());
    for i in 0..4 {
      packet.push(self.address.octets()[i]);
    }
    packet.push((self.port & 0xFF) as u8);
    packet.push((self.port >> 8 & 0xFF) as u8);
    packet.push('i' as u8);
    let amt = self.socket.send(&packet).await?;
    Ok(amt)
  }
  
  pub async fn recv(&self) -> Result<InfoPacket, std::io::Error> {
    let mut buf = [0; 1500];
    let amt;
    match timeout_at(Instant::now() + Duration::from_secs(2), self.socket.recv(&mut buf)).await? {
        Ok(n) => { amt = n},
        Err(e) => { return Err(e) },
    }
    let packet = Cursor::new(buf[11..amt].to_vec());
    self.build_info_packet(packet)
  }

  fn build_info_packet(&self, mut packet: Cursor<Vec<u8>>) -> Result<InfoPacket, std::io::Error> {
    let mut data = InfoPacket::default();
    data.password = packet.read_i8()? != 0;
    data.players = packet.read_u16::<LittleEndian>()?;
    data.max_players = packet.read_u16::<LittleEndian>()?;

    let hostname_len = packet.read_u32::<LittleEndian>()?;
    let mut hostname_buf = vec![0u8; hostname_len as usize];
    packet.read_exact(&mut hostname_buf).unwrap();
    data.hostname = String::from_utf8(hostname_buf).unwrap();

    let gamemode_len = packet.read_u32::<LittleEndian>()?;
    let mut gamemode_buf = vec![0u8; gamemode_len as usize];
    packet.read_exact(&mut gamemode_buf).unwrap();
    data.gamemode = String::from_utf8(gamemode_buf).unwrap();

    let language_len = packet.read_u32::<LittleEndian>()?;
    let mut language_buf = vec![0u8; language_len as usize];
    packet.read_exact(&mut language_buf).unwrap();
    data.language = String::from_utf8(language_buf).unwrap();
    Ok(data)
  }
}


#[cfg(test)]
mod tests {
  use super::*;
  
  #[tokio::test]
    async fn get_info() -> Result<(), std::io::Error> {
        struct Server(&'static str, i32, &'static str);
    
        let mut tests: Vec<Server> = Vec::new();
        tests.push(Server("51.91.203.178", 7777, ""));
        tests.push(Server("185.169.134.3", 7777, ""));
        tests.push(Server("193.203.39.13", 80, "slice index starts at 11 but ends at 0"));
        
        for s in tests {
            let query = Query::new(s.0, s.1).await?;
            let data = query.recv().await;
            if s.2 != "" {
                if data.is_err() {
                    assert_eq!(s.2, "slice index starts at 11 but ends at 0");
                } else {
                    let packet = data.unwrap();
                    assert_ne!(packet.hostname, "");
                    assert_ne!(packet.gamemode, "");
                    assert_ne!(packet.max_players, 0);
                    assert_ne!(packet.language, "");
                }
            }
        }
        Ok(())
    }
}
