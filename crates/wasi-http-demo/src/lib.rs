use std::sync::LazyLock;

use wstd::{
    http::{
        body::{BodyForthcoming, IncomingBody, OutgoingBody},
        server::{Finished, Responder},
        Client, IntoBody, Method, Request, Response, StatusCode, Uri,
    },
    io::{copy, empty, AsyncWrite},
    task, time,
};

#[wstd::http_server]
async fn main(req: Request<IncomingBody>, responder: Responder) -> Finished {
    match req.uri().path_and_query().unwrap().as_str() {
        "/client_get" => http_client_get(req, responder).await,
        "/client_post" => http_client_post(req, responder).await,
        "/wait" => http_wait(req, responder).await,
        "/echo" => http_echo(req, responder).await,
        "/echo-headers" => http_echo_headers(req, responder).await,
        "/echo-trailers" => http_echo_trailers(req, responder).await,
        "/fail" => http_fail(req, responder).await,
        "/bigfail" => http_bigfail(req, responder).await,
        "/" => http_home(req, responder).await,
        _ => http_not_found(req, responder).await,
    }
}

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    let mut client = Client::new();
    client.set_connect_timeout(time::Duration::from_secs(5));
    client.set_first_byte_timeout(time::Duration::from_secs(5));
    client.set_between_bytes_timeout(time::Duration::from_secs(5));
    client
});

async fn http_client_get(_req: Request<IncomingBody>, responder: Responder) -> Finished {
    let now = time::Instant::now();
    let request = Request::builder()
        .uri(Uri::from_static("https://www.baidu.com"))
        .method(Method::GET)
        .body(empty())
        .unwrap();

    let res = CLIENT.send(request).await.unwrap();
    let elapsed = time::Instant::now().duration_since(now).as_millis();
    println!("get baidu cost {elapsed} millis");
    responder.respond(res).await
}

async fn http_client_post(req: Request<IncomingBody>, responder: Responder) -> Finished {
    let now = time::Instant::now();
    let request = Request::builder()
        .uri("https://httpbin.org/post")
        .method(Method::POST)
        .header("Content-Type", "application/json; charset=utf-8")
        .body(req.into_body())
        .unwrap();

    let res = CLIENT.send(request).await.unwrap();
    let elapsed = time::Instant::now().duration_since(now).as_millis();
    println!("post httpbin cost {elapsed} millis");

    responder.respond(res).await
}

async fn http_wait(_req: Request<IncomingBody>, responder: Responder) -> Finished {
    let now = time::Instant::now();
    task::sleep(time::Duration::from_secs(1)).await;

    let elapsed = time::Instant::now().duration_since(now).as_millis();

    let mut body = responder.start_response(Response::new(BodyForthcoming));
    let result = body
        .write_all(format!("sleep for {elapsed} millis\n").as_bytes())
        .await;

    Finished::finish(body, result, None)
}

async fn http_echo(mut req: Request<IncomingBody>, responder: Responder) -> Finished {
    let mut body = responder.start_response(Response::new(BodyForthcoming));
    let result = copy(req.body_mut(), &mut body).await;
    Finished::finish(body, result, None)
}

async fn http_fail(_request: Request<IncomingBody>, responder: Responder) -> Finished {
    let body = responder.start_response(Response::new(BodyForthcoming));
    Finished::fail(body)
}

async fn http_bigfail(_request: Request<IncomingBody>, responder: Responder) -> Finished {
    async fn write_body(body: &mut OutgoingBody) -> wstd::io::Result<()> {
        for _ in 0..0x10 {
            body.write_all("big big big big\n".as_bytes()).await?;
        }
        body.flush().await?;
        Ok(())
    }

    let mut body = responder.start_response(Response::new(BodyForthcoming));
    let _ = write_body(&mut body).await;
    Finished::fail(body)
}

async fn http_echo_headers(req: Request<IncomingBody>, responder: Responder) -> Finished {
    let mut response = Response::builder();
    *response.headers_mut().unwrap() = req.into_parts().0.headers;
    let response = response.body(empty()).unwrap();
    responder.respond(response).await
}

async fn http_echo_trailers(req: Request<IncomingBody>, responder: Responder) -> Finished {
    let body = responder.start_response(Response::new(BodyForthcoming));

    let (trailers, result) = match req.into_body().finish().await {
        Ok(trailers) => (trailers, Ok(())),
        Err(err) => (Default::default(), Err(std::io::Error::other(err))),
    };

    Finished::finish(body, result, trailers)
}

async fn http_not_found(_req: Request<IncomingBody>, responder: Responder) -> Finished {
    responder
        .respond(
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(empty())
                .unwrap(),
        )
        .await
}

async fn http_home(_req: Request<IncomingBody>, responder: Responder) -> Finished {
    responder
        .respond(Response::new("Hello, wasi:http/proxy world!\n".into_body()))
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
