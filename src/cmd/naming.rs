use crate::error::I2pError;
use crate::socket::I2pSocket;
use crate::parser::{Command, Subcommand, parse};
use crate::cmd::aux;

fn parser(response: &str) -> Result<Vec<(String, String)>, I2pError> {

    let parsed = match parse(response, Command::Naming, Some(Subcommand::Reply)) {
        Ok(v)  => v,
        Err(e) => {
            eprintln!("Failed to parse response: {:#?}", e);
            return Err(I2pError::InvalidValue);
        }
    };

    match parsed.get_value("RESULT") {
        Some(v) => {
            match &v[..] {
                "OK" => {
                },
                "KEY_NOT_FOUND" => {
                    return Err(I2pError::DoesntExist);
                },
                "INVALID_KEY" | "INVALID" => {
                    return Err(I2pError::InvalidValue);
                }
                _ => {
                    todo!();
                }
            }
        },
        None => {
            eprintln!("Router response did not contain RESULT!");
            return Err(I2pError::InvalidValue);
        }
    }

    let value = match parsed.get_value("VALUE") {
        Some(v) => v.to_string(),
        None    => "".to_string(),
    };

    match parsed.get_value("NAME") {
        Some(v) => {
            return Ok(vec![(v.to_string(), value)]);
        },
        None => {
            eprintln!("Router's respone did not contain NAME!");
            return Err(I2pError::InvalidValue);
        }
    };
}

/// Handshake with the router to establish initial connection
///
/// # Arguments
///
/// `socket` - I2pSocket object created by the caller
///
pub fn lookup(socket: &mut I2pSocket, addr: &str) -> Result<(String, String), I2pError> {
    let msg = format!("NAMING LOOKUP NAME={}\n", addr);

    match aux::exchange_msg(socket, &msg, &parser) {
        Ok(v)  => Ok(v[0].clone()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::socket::{I2pSocket, SocketType};
    use crate::cmd::hello::*;

    #[test]
    fn test_lookup() {
        let mut socket = I2pSocket::new(SocketType::Tcp, "localhost", 7656).unwrap();

        // zzz.i2p exists
        assert_eq!(
           lookup(&mut socket, "zzz.i2p").unwrap().0,
            "zzz.i2p".to_string(),
        );

        assert_eq!(
            lookup(&mut socket, "abcdefghijklmnopqrstuvwxyz234567abcdefghijklmnopqrst.b32.i2p"),
            Err(I2pError::DoesntExist)
        );
    }

    // calls from the same socket to destination ME should result in the same public key
    #[test]
    fn test_lookup_same_socket() {
        let mut socket = I2pSocket::new(SocketType::Tcp, "localhost", 7656).unwrap();

        assert_eq!(
            lookup(&mut socket, "ME"),
            lookup(&mut socket, "ME"),
        );
    }

    // two separate connections, even from the same machine, should get different destinations
    // TODO fix this
    #[test]
    fn test_lookup_two_sockets() {
        let mut socket1 = I2pSocket::new(SocketType::Tcp, "localhost", 7656).unwrap();
        let mut socket2 = I2pSocket::new(SocketType::Tcp, "localhost", 7656).unwrap();

        assert_ne!(
            lookup(&mut socket1, "ME"),
            lookup(&mut socket2, "ME"),
        );
    }
}
