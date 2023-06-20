use std::io::Error;

use yarpc::http::{Call, Method, Request};

use crate::state::State;

pub struct ProxyState<T: Call>(pub T);

impl<T: Call> State for ProxyState<T> {
    fn get(&mut self, node_id: String) -> Result<Vec<u8>, Error> {
        let response = self.0.call(Request::new(
            Method::GET,
            format!("/?id={node_id}"),
            Default::default(),
            Default::default(),
        ))?;
        Ok(response.content)
    }
    fn post(&mut self, msg: Vec<u8>) -> Result<(), Error> {
        self.0.call(Request::new(
            Method::POST,
            "/".to_string(),
            Default::default(),
            msg,
        ))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn test() {
        let mut state = ProxyState(Server::default());
        assert!(state.get(1.to_string()).unwrap().is_empty());
        assert!(state.get(3.to_string()).unwrap().is_empty());
        state.post("Msg # 0".as_bytes().to_vec()).unwrap();
        assert_eq!(
            "Msg # 0".as_bytes().to_vec(),
            state.get(1.to_string()).unwrap()
        );
        assert_eq!(
            "Msg # 0".as_bytes().to_vec(),
            state.get(5.to_string()).unwrap()
        );
        assert_eq!(
            "Msg # 0".as_bytes().to_vec(),
            state.get(4.to_string()).unwrap()
        );
        assert!(state.get(1.to_string()).unwrap().is_empty());
        state.post("Msg # 1".as_bytes().to_vec()).unwrap();
        assert_eq!(
            "Msg # 1".as_bytes().to_vec(),
            state.get(1.to_string()).unwrap()
        );
        assert_eq!(
            "Msg # 0".as_bytes().to_vec(),
            state.get(3.to_string()).unwrap()
        );
        assert_eq!(
            "Msg # 1".as_bytes().to_vec(),
            state.get(5.to_string()).unwrap()
        );
        state.post("Msg # 2".as_bytes().to_vec()).unwrap();
        assert_eq!(
            "Msg # 2".as_bytes().to_vec(),
            state.get(1.to_string()).unwrap()
        );
        assert_eq!(
            "Msg # 1".as_bytes().to_vec(),
            state.get(4.to_string()).unwrap()
        );
        assert_eq!(
            "Msg # 2".as_bytes().to_vec(),
            state.get(4.to_string()).unwrap()
        );
    }
    struct BrokenServer();
    impl Call for BrokenServer {
        fn call(&mut self, _: Request) -> Result<yarpc::http::Response, Error> {
            Err(Error::new(std::io::ErrorKind::NotConnected, ""))
        }
    }
    #[test]
    fn post_fail() {
        let mut state = ProxyState(BrokenServer());
        assert!(state.post("Msg # 0".as_bytes().to_vec()).is_err());
    }
}
