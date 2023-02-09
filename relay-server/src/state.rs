/// An interface that defines the functions that need to be implemented in order to store and
/// retrieve messages.
///
/// This trait provides a way for the relay-server to abstract away the underlying storage
/// mechanism and allows for different storage implementations to be used, such as in-memory
/// or a remote database. By implementing the State trait, you can customize the storage
/// mechanism to fit your specific use case.
pub trait State {
    /// Get all messages received by a client
    ///
    /// # Arguments
    ///
    /// * `node_id` - A string that represents the client's ID
    ///
    /// # Returns
    ///
    /// A vector of bytes that represents the messages received by the client
    ///
    fn get(&mut self, node_id: String) -> Vec<u8>;
    /// Store a message on the server
    ///
    /// # Arguments
    ///
    /// * `msg` - A vector of bytes that represents the message to be stored
    ///    
    fn post(&mut self, msg: Vec<u8>);
}
