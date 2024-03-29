use pacesetter::init_tracing;
use rust_rest_web::run;

#[tokio::main]
async fn main() {
    init_tracing();

    if let Err(e) = run().await {
        tracing::error!(
            error.msg = %e,
            error.error_chain = ?e,
            "Shutting down due to error"
        )
    }
}
