#[tokio::test]
async fn test() {
    use ghost_actor::{GhostError, dependencies::futures};
    use futures::future::{BoxFuture, FutureExt};

    #[derive(Debug)]
    pub struct MyError(Box<dyn std::error::Error + Send + Sync>);

    impl std::fmt::Display for MyError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl std::error::Error for MyError {}

    impl From<GhostError> for MyError {
        fn from(e: GhostError) -> Self {
            Self(e.into())
        }
    }

    #[ghost_actor_derive::ghost_actor]
    pub trait Api {
        fn test(&mut self, input: String) -> BoxFuture<'static, Result<String, MyError>>;
    }

    #[ghost_actor_derive::ghost_actor]
    pub trait InternalApi {
        fn i_s_test(&mut self, input: String) -> BoxFuture<'static, Result<String, MyError>>;
    }

    struct I1;

    impl ghost_actor::GhostActor for I1 {}

    impl Api for I1 {
        fn test(&mut self, input: String) -> BoxFuture<'static, Result<String, MyError>> {
            async move {
                Ok(format!("echo: {}", input))
            }.boxed()
        }
    }

    impl InternalApi for I1 {
        fn i_s_test(&mut self, input: String) -> BoxFuture<'static, Result<String, MyError>> {
            async move {
                Ok(format!("internal echo: {}", input))
            }.boxed()
        }
    }

    let mut s = spawn_api_actor(I1);
    let mut s2 = s.clone();

    let res = futures::future::join_all(vec![
        s.test("test1".to_string()),
        s2.test("test2".to_string()),
    ]).await;

    assert_eq!(
        "[Ok(\"echo: test1\"), Ok(\"echo: test2\")]",
        &format!("{:?}", res),
    );
}
