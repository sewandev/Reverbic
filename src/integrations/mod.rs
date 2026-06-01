pub trait GameIntegration {
    type State: Clone + Default + Send + 'static;
    fn get() -> Option<Self::State>;
    fn spawn_server() -> tokio::task::JoinHandle<()>;
    fn reset();
}
