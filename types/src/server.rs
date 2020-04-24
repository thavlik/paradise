use crate::types;
use actix_web;
use colored::*;

fn error_json<T: std::string::ToString>(e: T) -> String {
    format!("{{\"error\":\"{}\"}}", e.to_string())
}

pub mod audio_interface {
    use super::*;

    pub fn routes<T>(cfg: &mut actix_web::web::ServiceConfig)
    where
        T: types::AudioInterface + std::clone::Clone + 'static,
    {
        cfg.service(
            actix_web::web::resource("/oto/AudioInterface.CreateStream").to(create_stream::<T>),
        )
        .service(
            actix_web::web::resource("/oto/AudioInterface.DeleteStream").to(delete_stream::<T>),
        )
        .service(
            actix_web::web::resource("/oto/AudioInterface.GetDeviceInfo").to(get_device_info::<T>),
        )
        .service(actix_web::web::resource("/oto/AudioInterface.ListStreams").to(list_streams::<T>));
    }

    // address: e.g. "127.0.0.1:8080"
    pub async fn main<T>(svc: T, address: &str) -> std::io::Result<()>
    where
        T: types::AudioInterface + std::clone::Clone + 'static,
    {
        actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .wrap(actix_web::middleware::Logger::default())
                .data(svc.clone())
                .configure(routes::<T>)
        })
        .bind(address)?
        .run()
        .await
    }

    pub async fn create_stream<'a, T>(
        svc: actix_web::web::Data<T>,
        req: actix_web::web::Json<types::CreateStreamRequest>,
    ) -> impl actix_web::Responder
    where
        T: types::AudioInterface + 'a,
    {
        let (status, body) = match svc.create_stream(req.into_inner()).await {
            Ok(res) => match serde_json::to_string(&res) {
                Ok(body) => (actix_web::http::StatusCode::OK, body),
                Err(e) => (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    error_json(format!("error serializing response: {:?}", &e)),
                ),
            },
            Err(e) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                error_json(&e),
            ),
        };
        //let (status, body, error) = match svc.create_stream(req.into_inner()).await {
        //    Ok(res) => match serde_json::to_string(&res) {
        //        Ok(body) => (actix_web::http::StatusCode::OK, body, None),
        //        Err(e) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, error_json(format!("error serializing response: {:?}", &e)), Some(format!("{:?}", &e))),
        //    },
        //    Err(e) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, error_json(&e), Some(format!("{:?}", &e))),
        //};
        //match &error {
        //    None => println!("{}", "[200] AudioInterface.CreateStream".green()),
        //    Some(e) => println!("{} {}", "[500] AudioInterface.CreateStream: {}".red(), e.red()),
        //};
        actix_web::web::HttpResponse::build(status)
            .content_type("application/json")
            .body(body)
    }

    pub async fn delete_stream<'a, T>(
        svc: actix_web::web::Data<T>,
        req: actix_web::web::Json<types::DeleteStreamRequest>,
    ) -> impl actix_web::Responder
    where
        T: types::AudioInterface + 'a,
    {
        let (status, body) = match svc.delete_stream(req.into_inner()).await {
            Ok(res) => match serde_json::to_string(&res) {
                Ok(body) => (actix_web::http::StatusCode::OK, body),
                Err(e) => (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    error_json(format!("error serializing response: {:?}", &e)),
                ),
            },
            Err(e) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                error_json(&e),
            ),
        };
        //let (status, body, error) = match svc.delete_stream(req.into_inner()).await {
        //    Ok(res) => match serde_json::to_string(&res) {
        //        Ok(body) => (actix_web::http::StatusCode::OK, body, None),
        //        Err(e) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, error_json(format!("error serializing response: {:?}", &e)), Some(format!("{:?}", &e))),
        //    },
        //    Err(e) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, error_json(&e), Some(format!("{:?}", &e))),
        //};
        //match &error {
        //    None => println!("{}", "[200] AudioInterface.DeleteStream".green()),
        //    Some(e) => println!("{} {}", "[500] AudioInterface.DeleteStream: {}".red(), e.red()),
        //};
        actix_web::web::HttpResponse::build(status)
            .content_type("application/json")
            .body(body)
    }

    pub async fn get_device_info<'a, T>(
        svc: actix_web::web::Data<T>,
        req: actix_web::web::Json<types::GetDeviceInfoRequest>,
    ) -> impl actix_web::Responder
    where
        T: types::AudioInterface + 'a,
    {
        let (status, body) = match svc.get_device_info(req.into_inner()).await {
            Ok(res) => match serde_json::to_string(&res) {
                Ok(body) => (actix_web::http::StatusCode::OK, body),
                Err(e) => (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    error_json(format!("error serializing response: {:?}", &e)),
                ),
            },
            Err(e) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                error_json(&e),
            ),
        };
        //let (status, body, error) = match svc.get_device_info(req.into_inner()).await {
        //    Ok(res) => match serde_json::to_string(&res) {
        //        Ok(body) => (actix_web::http::StatusCode::OK, body, None),
        //        Err(e) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, error_json(format!("error serializing response: {:?}", &e)), Some(format!("{:?}", &e))),
        //    },
        //    Err(e) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, error_json(&e), Some(format!("{:?}", &e))),
        //};
        //match &error {
        //    None => println!("{}", "[200] AudioInterface.GetDeviceInfo".green()),
        //    Some(e) => println!("{} {}", "[500] AudioInterface.GetDeviceInfo: {}".red(), e.red()),
        //};
        actix_web::web::HttpResponse::build(status)
            .content_type("application/json")
            .body(body)
    }

    pub async fn list_streams<'a, T>(
        svc: actix_web::web::Data<T>,
        req: actix_web::web::Json<types::ListStreamsRequest>,
    ) -> impl actix_web::Responder
    where
        T: types::AudioInterface + 'a,
    {
        let (status, body) = match svc.list_streams(req.into_inner()).await {
            Ok(res) => match serde_json::to_string(&res) {
                Ok(body) => (actix_web::http::StatusCode::OK, body),
                Err(e) => (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    error_json(format!("error serializing response: {:?}", &e)),
                ),
            },
            Err(e) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                error_json(&e),
            ),
        };
        //let (status, body, error) = match svc.list_streams(req.into_inner()).await {
        //    Ok(res) => match serde_json::to_string(&res) {
        //        Ok(body) => (actix_web::http::StatusCode::OK, body, None),
        //        Err(e) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, error_json(format!("error serializing response: {:?}", &e)), Some(format!("{:?}", &e))),
        //    },
        //    Err(e) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, error_json(&e), Some(format!("{:?}", &e))),
        //};
        //match &error {
        //    None => println!("{}", "[200] AudioInterface.ListStreams".green()),
        //    Some(e) => println!("{} {}", "[500] AudioInterface.ListStreams: {}".red(), e.red()),
        //};
        actix_web::web::HttpResponse::build(status)
            .content_type("application/json")
            .body(body)
    }
}

#[cfg(test)]
pub mod test {

    pub mod audio_interface {
        use super::super::*;
        use actix_web::test;

        /*
        pub mod client {
            use super::*;
            pub struct Client<T> where T: types::AudioInterface + std::clone::Clone {
                svc: T,
            }

            impl<T> Client<T> where T: types::AudioInterface + std::clone::Clone + 'static {
                pub fn new(svc: T) -> Self {
                    let mut app = test::init_service(actix_web::App::new()
                        .data(svc.clone())
                        .configure(super::super::super::audio_interface::routes::<T>)
                    );
                    Client{ svc }
                }
            }



            #[async_trait]
            impl<T> types::AudioInterface for Client<T> where T: types::AudioInterface + std::clone::Clone {

                    async fn create_stream(&self, req: types::CreateStreamRequest) -> Result<types::CreateStreamResponse, String> {
                        Err(String::new())
                    }

                    async fn delete_stream(&self, req: types::DeleteStreamRequest) -> Result<types::DeleteStreamResponse, String> {
                        Err(String::new())
                    }

                    async fn get_device_info(&self, req: types::GetDeviceInfoRequest) -> Result<types::GetDeviceInfoResponse, String> {
                        Err(String::new())
                    }

                    async fn list_streams(&self, req: types::ListStreamsRequest) -> Result<types::ListStreamsResponse, String> {
                        Err(String::new())
                    }

            }
        }*/

        async fn index() -> impl actix_web::Responder {
            String::from("Hello, world!")
        }

        #[actix_rt::test]
        async fn create_stream_ok() {
            let endpoint = "/oto/AudioInterface.CreateStream";
            let body = types::CreateStreamRequest::new();
            let req = test::TestRequest::post()
                .uri(endpoint)
                .set_json(&body)
                .to_request();
            let svc = types::mock::MockAudioInterface::new();
            let mut app = test::init_service(actix_web::App::new().data(svc.clone()).configure(
                super::super::audio_interface::routes::<types::mock::MockAudioInterface>,
            ))
            .await;
            let resp = test::call_service(&mut app, req).await;
            assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
            let result = test::read_body(resp).await;
            let obj = types::CreateStreamResponse::new();
            let expected = serde_json::to_string(&obj).unwrap();
            assert_eq!(result, expected);
        }

        #[actix_rt::test]
        async fn create_stream_error() {
            let endpoint = "/oto/AudioInterface.CreateStream";
            let body = types::CreateStreamRequest::new();
            let req = test::TestRequest::post()
                .uri(endpoint)
                .set_json(&body)
                .to_request();
            let svc =
                types::mock::MockAudioInterface::error("Hello from AudioInterface.CreateStream!");
            let mut app = test::init_service(actix_web::App::new().data(svc.clone()).configure(
                super::super::audio_interface::routes::<types::mock::MockAudioInterface>,
            ))
            .await;
            let resp = test::call_service(&mut app, req).await;
            assert_eq!(
                resp.status(),
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
            );
            let result = test::read_body(resp).await;
            let expected = error_json("Hello from AudioInterface.CreateStream!");
            assert_eq!(result, expected);
        }

        #[actix_rt::test]
        async fn delete_stream_ok() {
            let endpoint = "/oto/AudioInterface.DeleteStream";
            let body = types::DeleteStreamRequest::new();
            let req = test::TestRequest::post()
                .uri(endpoint)
                .set_json(&body)
                .to_request();
            let svc = types::mock::MockAudioInterface::new();
            let mut app = test::init_service(actix_web::App::new().data(svc.clone()).configure(
                super::super::audio_interface::routes::<types::mock::MockAudioInterface>,
            ))
            .await;
            let resp = test::call_service(&mut app, req).await;
            assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
            let result = test::read_body(resp).await;
            let obj = types::DeleteStreamResponse::new();
            let expected = serde_json::to_string(&obj).unwrap();
            assert_eq!(result, expected);
        }

        #[actix_rt::test]
        async fn delete_stream_error() {
            let endpoint = "/oto/AudioInterface.DeleteStream";
            let body = types::DeleteStreamRequest::new();
            let req = test::TestRequest::post()
                .uri(endpoint)
                .set_json(&body)
                .to_request();
            let svc =
                types::mock::MockAudioInterface::error("Hello from AudioInterface.DeleteStream!");
            let mut app = test::init_service(actix_web::App::new().data(svc.clone()).configure(
                super::super::audio_interface::routes::<types::mock::MockAudioInterface>,
            ))
            .await;
            let resp = test::call_service(&mut app, req).await;
            assert_eq!(
                resp.status(),
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
            );
            let result = test::read_body(resp).await;
            let expected = error_json("Hello from AudioInterface.DeleteStream!");
            assert_eq!(result, expected);
        }

        #[actix_rt::test]
        async fn get_device_info_ok() {
            let endpoint = "/oto/AudioInterface.GetDeviceInfo";
            let body = types::GetDeviceInfoRequest::new();
            let req = test::TestRequest::post()
                .uri(endpoint)
                .set_json(&body)
                .to_request();
            let svc = types::mock::MockAudioInterface::new();
            let mut app = test::init_service(actix_web::App::new().data(svc.clone()).configure(
                super::super::audio_interface::routes::<types::mock::MockAudioInterface>,
            ))
            .await;
            let resp = test::call_service(&mut app, req).await;
            assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
            let result = test::read_body(resp).await;
            let obj = types::GetDeviceInfoResponse::new();
            let expected = serde_json::to_string(&obj).unwrap();
            assert_eq!(result, expected);
        }

        #[actix_rt::test]
        async fn get_device_info_error() {
            let endpoint = "/oto/AudioInterface.GetDeviceInfo";
            let body = types::GetDeviceInfoRequest::new();
            let req = test::TestRequest::post()
                .uri(endpoint)
                .set_json(&body)
                .to_request();
            let svc =
                types::mock::MockAudioInterface::error("Hello from AudioInterface.GetDeviceInfo!");
            let mut app = test::init_service(actix_web::App::new().data(svc.clone()).configure(
                super::super::audio_interface::routes::<types::mock::MockAudioInterface>,
            ))
            .await;
            let resp = test::call_service(&mut app, req).await;
            assert_eq!(
                resp.status(),
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
            );
            let result = test::read_body(resp).await;
            let expected = error_json("Hello from AudioInterface.GetDeviceInfo!");
            assert_eq!(result, expected);
        }

        #[actix_rt::test]
        async fn list_streams_ok() {
            let endpoint = "/oto/AudioInterface.ListStreams";
            let body = types::ListStreamsRequest::new();
            let req = test::TestRequest::post()
                .uri(endpoint)
                .set_json(&body)
                .to_request();
            let svc = types::mock::MockAudioInterface::new();
            let mut app = test::init_service(actix_web::App::new().data(svc.clone()).configure(
                super::super::audio_interface::routes::<types::mock::MockAudioInterface>,
            ))
            .await;
            let resp = test::call_service(&mut app, req).await;
            assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
            let result = test::read_body(resp).await;
            let obj = types::ListStreamsResponse::new();
            let expected = serde_json::to_string(&obj).unwrap();
            assert_eq!(result, expected);
        }

        #[actix_rt::test]
        async fn list_streams_error() {
            let endpoint = "/oto/AudioInterface.ListStreams";
            let body = types::ListStreamsRequest::new();
            let req = test::TestRequest::post()
                .uri(endpoint)
                .set_json(&body)
                .to_request();
            let svc =
                types::mock::MockAudioInterface::error("Hello from AudioInterface.ListStreams!");
            let mut app = test::init_service(actix_web::App::new().data(svc.clone()).configure(
                super::super::audio_interface::routes::<types::mock::MockAudioInterface>,
            ))
            .await;
            let resp = test::call_service(&mut app, req).await;
            assert_eq!(
                resp.status(),
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
            );
            let result = test::read_body(resp).await;
            let expected = error_json("Hello from AudioInterface.ListStreams!");
            assert_eq!(result, expected);
        }
    }
}
