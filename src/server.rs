use tonic::{transport::Server, Request, Response, Status};

use link_eternal::{
    link_service_server::{LinkService, LinkServiceServer},
    AddLinkRequest, ListLinksRequest, ListLinksResponse,
};

pub mod link_eternal {
    tonic::include_proto!("api");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("api_descriptor");
}

#[derive(Debug, Default)]
pub struct EternalLinkService {}

#[tonic::async_trait]
impl LinkService for EternalLinkService {
    async fn list_links(
        &self,
        request: Request<ListLinksRequest>,
    ) -> Result<Response<ListLinksResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = link_eternal::ListLinksResponse {
            data: vec![link_eternal::Link {
                id: "1".to_string(),
                url: "https://google.com".into(),
                tags: vec!["search".into(), "google".into()],
            }],
        };

        Ok(Response::new(reply))
    }

    async fn add_link(
        &self,
        request: Request<AddLinkRequest>,
    ) -> Result<Response<link_eternal::Link>, Status> {
        println!("Got a request: {:?}", request);

        let reply = link_eternal::Link {
            id: "1".to_string(),
            url: "https://google.com".into(),
            tags: vec!["search".into(), "google".into()],
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(link_eternal::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let addr = "[::1]:50051".parse()?;
    let greeter = EternalLinkService::default();

    Server::builder()
        .add_service(service)
        .add_service(LinkServiceServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
