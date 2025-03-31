use std::sync::Arc;
use tokio::net::TcpStream;
use hyper::{Body, Request, Response, Server, service::{make_service_fn, service_fn}};
use hyper::server::conn::Http;
use tracing::{error, info, debug};
use std::convert::Infallible;

use crate::core::config::Config;
use crate::handlers::static_files::StaticFileHandler;
use crate::routing::router::Router;

/// Handler for TCP connections that processes HTTP requests
pub struct ConnectionHandler {
    /// The TCP stream for this connection
    stream: TcpStream,
    /// Server configuration
    config: Arc<Config>,
}

impl ConnectionHandler {
    /// Create a new connection handler
    pub fn new(stream: TcpStream, config: Arc<Config>) -> Self {
        ConnectionHandler {
            stream,
            config,
        }
    }
    
    /// Process the connection
    pub async fn process(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create a hyper HTTP connection
        let http = Http::new();
        
        // Create a router for request handling
        let router = Router::new(Arc::clone(&self.config));
        
        // Create a static file handler
        let static_handler = StaticFileHandler::new(
            &self.config.static_files.root_dir,
            self.config.static_files.directory_listing.unwrap_or(false),
            self.config.static_files.default_file.clone().unwrap_or_else(|| "index.html".to_string()),
        );
        
        // Create service for handling requests
        let service = make_service_fn(move |_conn| {
            let router_clone = router.clone();
            let static_handler_clone = static_handler.clone();
            let config_clone = Arc::clone(&self.config);
            
            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    let router = router_clone.clone();
                    let handler = static_handler_clone.clone();
                    let config = Arc::clone(&config_clone);
                    
                    async move {
                        Self::handle_request(req, router, handler, config).await
                    }
                }))
            }
        });
        
        // Create server with the service
        let server = Server::builder(hyper::server::accept::from_stream(futures::stream::once(
            futures::future::ok::<_, hyper::Error>(self.stream.clone())
        )))
        .serve(service);
        
        // Run the server to process the connection
        if let Err(e) = server.await {
            error!("Error serving connection: {}", e);
            return Err(Box::new(e));
        }
        
        Ok(())
    }
    
    /// Handle an individual HTTP request
    async fn handle_request(
        req: Request<Body>,
        router: Router,
        static_handler: StaticFileHandler,
        config: Arc<Config>,
    ) -> Result<Response<Body>, Infallible> {
        let method = req.method().clone();
        let uri = req.uri().clone();
        
        info!("{} {}", method, uri);
        
        // Route the request to the appropriate handler
        let route_result = router.route(&req);
        
        match route_result {
            Ok(route) => {
                debug!("Route matched: {:?}", route);
                
                // Handle the request based on the route type
                match route.handler_type.as_str() {
                    "static" => static_handler.handle(req).await,
                    // Add other handler types as needed
                    _ => {
                        error!("Unknown handler type: {}", route.handler_type);
                        Ok(Response::builder()
                            .status(500)
                            .body(Body::from("Internal Server Error: Unknown handler type"))
                            .unwrap())
                    }
                }
            }
            Err(_) => {
                // If no route matches, default to static file handler
                static_handler.handle(req).await
            }
        }
    }
}
