use crate::{
    http::{
        Call,
        Method::{GET, POST},
    },
    state::State,
};

pub struct ProxyState<T: Call>(pub T);

impl<T: Call> State for ProxyState<T> {
    fn get(&mut self, node_id: String) -> Vec<u8> {
        self.0
            .call(GET.request(
                format!("/?id={node_id}"),
                Default::default(),
                Default::default(),
            ))
            .content
    }

    fn post(&mut self, msg: Vec<u8>) {
        self.0
            .call(POST.request("/".to_string(), Default::default(), msg));
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn test() {
        let mut state = ProxyState(Server::default());
        assert!(state.get(1.to_string()).is_empty());
        assert!(state.get(3.to_string()).is_empty());
        // assert_eq!(0, state.highwaters.len());
        state.post("Msg # 0".as_bytes().to_vec());
        assert_eq!("Msg # 0".as_bytes().to_vec(), state.get(1.to_string()));
        assert_eq!("Msg # 0".as_bytes().to_vec(), state.get(5.to_string()));
        assert_eq!("Msg # 0".as_bytes().to_vec(), state.get(4.to_string()));
        assert!(state.get(1.to_string()).is_empty());
        state.post("Msg # 1".as_bytes().to_vec());
        assert_eq!("Msg # 1".as_bytes().to_vec(), state.get(1.to_string()));
        assert_eq!("Msg # 0".as_bytes().to_vec(), state.get(3.to_string()));
        assert_eq!("Msg # 1".as_bytes().to_vec(), state.get(5.to_string()));
        state.post("Msg # 2".as_bytes().to_vec());
        assert_eq!("Msg # 2".as_bytes().to_vec(), state.get(1.to_string()));
        assert_eq!("Msg # 1".as_bytes().to_vec(), state.get(4.to_string()));
        assert_eq!("Msg # 2".as_bytes().to_vec(), state.get(4.to_string()));
    }
}
