use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

use crate::error::I2pError;
use crate::socket::{I2pSocket, SocketType};
use crate::cmd::*;

const SAM_TCP_PORT: u16  = 7656;
const _SAM_UDP_PORT: u16 = 7655;

pub enum SessionType {
    VirtualStream,
    RepliableDatagram,
    AnonymousDatagram,
}

pub struct I2pSession {
    pub socket: I2pSocket,
    pub nick:   String,
    pub local:  String,
}

impl I2pSession {

    /// Start a new session with the I2P router
    ///
    /// Connect to the router via the default SAM gateway (localhost:7656)
    /// and create a control socket, which is used to create the actual session,
    /// and a nickname for the client (random alphanumeric string)
    ///
    /// # Arguments
    ///
    /// `stype` - Session type: Virtual stream, Repliable or Anonymous datagram
    ///
    pub fn new(stype: SessionType) -> Result<I2pSession, I2pError> {

        let mut socket = match I2pSocket::new(SocketType::Tcp, "localhost", SAM_TCP_PORT) {
            Ok(v)  => v,
            Err(e) => {
                eprintln!("Failed to connect to the router: {:#?}", e);
                return Err(I2pError::TcpConnectionError);
            }
        };

        // generate random nickname
        let nick: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();

        // create a new session of type "stype"
        match session::create(&mut socket, &stype, &nick) {
            Ok(_)  => {},
            Err(e) => return Err(e),
        }

        // and fetch our local destination
        let dest = match naming::lookup(&mut socket, "ME") {
            Ok(v) => {
                if v.1 == "" {
                    return Err(I2pError::InvalidValue);
                }
                v.1
            },
            Err(e) => return Err(e),
        };

        Ok(I2pSession {
            socket: socket,
            nick:   nick.to_string(),
            local:  dest.to_string(),
        })
    }

    /// TODO
    pub fn destroy(&self) -> Result<(), I2pError> {
        Ok(())
    }
}
