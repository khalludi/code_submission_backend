use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::fs::File;
use std::io;
use std::io::Write;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/runCode", post(run_code));

    // run our app with hyper, listening globally on port 5000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn run_code(Json(payload): Json<CreateCode>) -> (StatusCode, Json<Output>) {
    println!("{:?}", payload);

    let mut file = File::create("test.py").expect("unable to create file");
    file.write_all(payload.code.as_bytes()).expect("could not write code to file");

    let mut process = Command::new("docker")
        .args(["build", "-t", "test-py", "."])
        .spawn()
        .unwrap();
    process.wait().unwrap();

    let output = Command::new("docker")
        .args(["run", "test-py"])
        .output()
        .unwrap();
    println!("status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    let result = Output {
        output: String::from_utf8(output.stdout).unwrap(),
    };

    (StatusCode::OK, Json(result))
}

#[derive(Deserialize, Debug)]
struct CreateCode {
    code: String,
}

#[derive(Serialize, Debug)]
struct Output {
    output: String,
}
