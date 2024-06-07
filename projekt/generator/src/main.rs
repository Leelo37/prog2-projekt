use std::net::SocketAddr;

use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Error;
use hyper::{body::Body, Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use serde::{Deserialize, Serialize};

const PORT: u16 = 12345;

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub name: String,
    pub ip: String,
    pub port: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Range {
    pub from: u64,
    pub to: u64, 
    pub step: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SequenceSyntax {
    pub name: String,
    pub parameters: Vec<f64>,
    pub sequences: Vec<Box<SequenceSyntax>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SequenceRequest {
    pub range: Range,
    pub parameters: Vec<f64>,
    pub sequences: Vec<Box<SequenceSyntax>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SequenceInfo {
    name: String,
    description: String,
    parameters: u32,
    sequences: u32,
}

pub trait Sequence {
    fn k_th(&self, k: usize) -> f64;
    fn range(&self, range: Range) -> Vec<f64> {
        let mut result = Vec::new();
        let mut k = range.from;
        while k <= range.to {
            result.push(self.k_th(k as usize));
            k += range.step;
        }
        result
    }
}

pub struct Arithmetic {
    start: f64,
    step: f64,
}

impl Arithmetic {
    pub fn new(start: f64, step: f64) -> Box<Arithmetic> {
        Box::new(Arithmetic { start, step })
    }
}

impl Sequence for Arithmetic {
    fn k_th(&self, k: usize) -> f64 {
        self.start + (k as f64) * self.step
    }
}

pub struct Geometric {
    start: f64,
    factor: f64,
}

impl Geometric {
    pub fn new(start: f64, factor: f64) -> Box<Geometric> {
        Box::new(Geometric { start, factor })
    }
}

impl Sequence for Geometric {
    fn k_th(&self, k: usize) -> f64 {
        self.start * f64::powf(self.factor, k as f64)
    }
}

pub struct Constant {
    start: f64,
}

impl Constant {
    pub fn new(start: f64) -> Box<Constant> {
        Box::new(Constant { start })
    }
}

impl Sequence for Constant {
    fn k_th(&self, _k: usize) -> f64 {
        self.start
    }
}

pub struct Sum {
    seq1: Box<dyn Sequence>,
    seq2: Box<dyn Sequence>
}

impl Sum {
    pub fn new(seq1: Box<dyn Sequence>, seq2: Box<dyn Sequence>) -> Box<Sum> {
        Box::new(Sum { seq1, seq2 })
    }
}

impl Sequence for Sum {
    fn k_th(&self, k: usize) -> f64 {
        self.seq1.k_th(k) + self.seq2.k_th(k)
    }
}

pub struct Prod {
    seq1: Box<dyn Sequence>,
    seq2: Box<dyn Sequence>
}

impl Prod {
    pub fn new(seq1: Box<dyn Sequence>, seq2: Box<dyn Sequence>) -> Box<Prod> {
        Box::new(Prod { seq1, seq2 })
    }
}

impl Sequence for Prod {
    fn k_th(&self, k: usize) -> f64 {
        self.seq1.k_th(k) * self.seq2.k_th(k)
    }
}

pub struct Drop {
    seq: Box<dyn Sequence>,
    n: usize
}

impl Drop {
    pub fn new(seq: Box<dyn Sequence>, n: usize) -> Box<Drop> {
        Box::new(Drop { seq, n })
    }
}

impl Sequence for Drop {
    fn k_th(&self, k: usize) -> f64 {
        self.seq.k_th(k + self.n)
    }
}

pub struct LinComb {
    a: f64,
    b: f64,
    c: f64,
    seq1: Box<dyn Sequence>,
    seq2: Box<dyn Sequence>
}

impl LinComb {
    pub fn new(a: f64, b: f64, c: f64, seq1: Box<dyn Sequence>, seq2: Box<dyn Sequence>) -> Box<LinComb> {
        Box::new(LinComb { a, b, c, seq1, seq2 })
    }
}

impl Sequence for LinComb {
    fn k_th(&self, k: usize) -> f64 {
        self.a * self.seq1.k_th(k) + self.b * self.seq2.k_th(k) + self.c
    }
}

fn sequences() -> Vec<SequenceInfo> {
    let mut sequences = Vec::new();
    sequences.push(SequenceInfo {
        name: "Arithmetic".to_string(),
        description: "Arithmetic sequence".to_string(),
        parameters: 2,
        sequences: 0,
    });
    sequences.push(SequenceInfo {
        name: "Geometric".to_string(),
        description: "Geometric sequence".to_string(),
        parameters: 2,
        sequences: 0,
    });
    sequences.push(SequenceInfo {
        name: "Constant".to_string(),
        description: "Constant sequence".to_string(),
        parameters: 1,
        sequences: 0,
    });
    sequences.push(SequenceInfo {
        name: "Sum".to_string(),
        description: "Element-wise sum of two sequences".to_string(),
        parameters: 0,
        sequences: 2,
    });
    sequences.push(SequenceInfo {
        name: "Prod".to_string(),
        description: "Element-wise product of two sequences".to_string(),
        parameters: 0,
        sequences: 2,
    });
    sequences.push(SequenceInfo {
        name: "Drop".to_string(),
        description: "Drops the first n elements of a sequence".to_string(),
        parameters: 1,
        sequences: 1,
    });
    sequences.push(SequenceInfo {
        name: "LinComb".to_string(),
        description: "Linear combination of two sequences".to_string(),
        parameters: 3,
        sequences: 2,
    });
    sequences
}

fn get_project() -> Project {
    return Project {
        name: "Matija & Filip".to_string(),
        ip: "127.0.0.1".to_string(),
        port: PORT,
    };
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
async fn collect_body(req: Request<Incoming>) -> Result<String, hyper::Error> {
    let max = req.body().size_hint().upper().unwrap_or(u64::MAX);
    if max > 1024 * 64 {
        panic!("Body too big");
    }

    let whole_body = req.collect().await?.to_bytes();
    let whole_body = std::str::from_utf8(&whole_body).unwrap().to_string();
    return Ok(whole_body);
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

async fn send_post(url: String, body: String) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client.post(url).body(body).send().await?.text().await?;
    return Ok(res);
}

async fn send_get(url: String) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client.get(url).send().await?.text().await?;
    return Ok(res);
}

async fn handle_sequence_request(req: Request<Incoming>, sequence_info: &SequenceInfo) -> Result<Response<BoxBody<Bytes, Error>>, hyper::Error> {
    let body = collect_body(req).await?;
    let request: SequenceRequest = serde_json::from_str(&body).unwrap();
    let range = request.range;

    let sequence: Box<dyn Sequence> = match sequence_info.name.as_str() {
        "Arithmetic" => Arithmetic::new(request.parameters[0], request.parameters[1]),
        "Geometric" => Geometric::new(request.parameters[0], request.parameters[1]),
        "Constant" => Constant::new(request.parameters[0]),
        "Sum" => {
            let seq1 = create_sequence_from_syntax(&request.sequences[0]);
            let seq2 = create_sequence_from_syntax(&request.sequences[1]);
            Sum::new(seq1, seq2)
        }
        "Prod" => {
            let seq1 = create_sequence_from_syntax(&request.sequences[0]);
            let seq2 = create_sequence_from_syntax(&request.sequences[1]);
            Prod::new(seq1, seq2)
        }
        "Drop" => {
            let seq = create_sequence_from_syntax(&request.sequences[0]);
            Drop::new(seq, request.parameters[0] as usize)
        }
        "LinComb" => {
            let seq1 = create_sequence_from_syntax(&request.sequences[0]);
            let seq2 = create_sequence_from_syntax(&request.sequences[1]);
            LinComb::new(request.parameters[0], request.parameters[1], request.parameters[2], seq1, seq2)
        }
        _ => panic!("Sequence not implemented")
    };

    Ok(Response::new(full(serde_json::to_string(&sequence.range(range)).unwrap())))
}

fn create_sequence_from_syntax(syntax: &SequenceSyntax) -> Box<dyn Sequence> {
    match syntax.name.as_str() {
        "Arithmetic" => Arithmetic::new(syntax.parameters[0], syntax.parameters[1]),
        "Geometric" => Geometric::new(syntax.parameters[0], syntax.parameters[1]),
        "Constant" => Constant::new(syntax.parameters[0]),
        "Sum" => {
            let seq1 = create_sequence_from_syntax(&syntax.sequences[0]);
            let seq2 = create_sequence_from_syntax(&syntax.sequences[1]);
            Sum::new(seq1, seq2)
        }
        "Prod" => {
            let seq1 = create_sequence_from_syntax(&syntax.sequences[0]);
            let seq2 = create_sequence_from_syntax(&syntax.sequences[1]);
            Prod::new(seq1, seq2)
        }
        "Drop" => {
            let seq = create_sequence_from_syntax(&syntax.sequences[0]);
            Drop::new(seq, syntax.parameters[0] as usize)
        }
        "LinComb" => {
            let seq1 = create_sequence_from_syntax(&syntax.sequences[0]);
            let seq2 = create_sequence_from_syntax(&syntax.sequences[1]);
            LinComb::new(syntax.parameters[0], syntax.parameters[1], syntax.parameters[2], seq1, seq2)
        }
        _ => panic!("Unsupported sequence")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = ([127, 0, 0, 1], PORT).into();

    let b = send_get("http://127.0.0.1:7878/project".to_string()).await?;
    println!("HERE {}", b);

    let b = send_post(
        "http://127.0.0.1:7878/project".to_string(),
        serde_json::to_string(&get_project()).unwrap(),
    )
    .await?;
    println!("HERE {}", b);

    let b = send_get("http://127.0.0.1:7878".to_string()).await?;
    println!("HERE {}", b);

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    let create_404 = || {
        let mut not_found = Response::new(empty());
        *not_found.status_mut() = StatusCode::NOT_FOUND;
        Ok(not_found)
    };

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let service = service_fn(move |req| {
            async move {
                match (req.method(), req.uri().path()) {
                    (&Method::GET, "/ping") => Ok::<_, Error>(Response::new(full(
                        serde_json::to_string(&get_project()).unwrap(),
                    ))),
                    (&Method::GET, "/sequence") => {
                        let sequences = sequences();
                        Ok(Response::new(full(
                            serde_json::to_string(&sequences).unwrap(),
                        )))
                    }
                    (&Method::POST, r) => {
                        let seqs = sequences();
                        if let Some(sequence_info) = seqs
                            .iter()
                            .find(|&x| ("/sequence/".to_string() + &x.name) == r)
                        {
                            handle_sequence_request(req, sequence_info).await
                        } else {
                            create_404()
                        }
                    }
                    _ => create_404(),
                }
            }
        });

        if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
            println!("Error serving connection: {:?}", err);
        }
    }
}