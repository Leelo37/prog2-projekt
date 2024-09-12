use std::net::SocketAddr;
use core::convert::Infallible;

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

pub struct Recursive {
    x0: f64,
    x1: f64,
    a: f64,
    b: f64
}

impl Recursive {
    pub fn new(x0: f64, x1: f64, a: f64, b: f64) -> Box<Recursive> {
        Box::new(Recursive { x0, x1, a, b })
    }
}

impl Sequence for Recursive {
    fn kth(&self, k: usize) -> f64 {
        if k == 0 {
            return self.x0;
        }
        else if k == 1 {
            return self.x1;
        }
        let mut prev1 = self.x1;
        let mut prev2 = self.x0;
        let mut current = 0.0;
        for  in 2..=k {
            current = self.a * prev2 + self.b * prev1;
            prev2 = prev1;
            prev1 = current;
        }
        current
    }
}

pub struct Average {
    seq1: Box<dyn Sequence>,
    seq2: Box<dyn Sequence>
}

impl Average {
    pub fn new(seq1: Box<dyn Sequence>, seq2: Box<dyn Sequence>) -> Box<Average> {
        Box::new(Average { seq1, seq2 })
    }
}

impl Sequence for Average {
    fn k_th(&self, k: usize) -> f64 {
        (self.seq1.k_th(k) + self.seq2.k_th(k)) / 2.0
    }
}

pub struct Cyclic {
    seq: Box<dyn Sequence>,
    cycle_length: usize
}

impl Cyclic {
    pub fn new(seq: Box<dyn Sequence>, cycle_length: usize) -> Box<Cyclic> {
        Box::new(Cyclic { seq, cycle_length })
    }
}

impl Sequence for Cyclic {
    fn k_th(&self, k: usize) -> f64 {
        self.seq.k_th(k % self.cycle_length)
    }
}

pub struct Alternating {
    seq: Box<dyn Sequence>
}

impl Alternating {
    pub fn new(seq: Box<dyn Sequence>) -> Box<Alternating> {
        Box::new(Alternating { seq })
    }
}

impl Sequence for Alternating {
    fn k_th(&self, k: usize) -> f64 {
        (-1.0f64).powi(k.try_into().unwrap()) * self.seq.k_th(k)
    }
}

pub struct Smoothed {
    seq: Box<dyn Sequence>
}

impl Smoothed {
    pub fn new(seq: Box<dyn Sequence>) -> Box<Smoothed> {
        Box::new(Smoothed { seq })
    }
}

impl Sequence for Smoothed {
    fn k_th(&self, k: usize) -> f64 {
        if k == 0 {
            (self.seq.k_th(k) + self.seq.k_th(k+1)) / 2.0
        } else {
            (self.seq.k_th(k-1) + self.seq.k_th(k) + self.seq.k_th(k+1)) / 3.0
        }
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
    sequences.push(SequenceInfo {
        name: "Recursive".to_string(),
        description: "Recursive sequence of the form x(n) = ax(n-1) + bx(n-2)".to_string(),
        parameters: 4,
        sequences: 0,
    });
    sequences.push(SequenceInfo {
        name: "Average".to_string(),
        description: "Element-wise average of two given sequences".to_string(),
        parameters: 0,
        sequences: 2,
    });
    sequences.push(SequenceInfo {
        name: "Cyclic".to_string(),
        description: "Sequence that cycles over a specified number of starting terms of another sequence".to_string(),
        parameters: 1,
        sequences: 1,
    });
    sequences.push(SequenceInfo {
        name: "Alternating".to_string(),
        description: "Sequence where the sign of each element alternates based on its index.".to_string(),
        parameters: 0,
        sequences: 1,
    });
    sequences.push(SequenceInfo {
        name: "Smoothed".to_string(),
        description: "Sequence where each element is the average of itself and its immediate neighbors".to_string(),
        parameters: 0,
        sequences: 1,
    });
    sequences
}

fn get_project() -> Project {
    return Project {
        name: "Lucija & Lev".to_string(),
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
    let body2 = body.clone();
    let request: SequenceRequest = serde_json::from_str(&body).unwrap();
    let request1: SequenceRequest = serde_json::from_str(&body2).unwrap();
    let range = request.range;

    let sequence: Option<Box<dyn Sequence>> = match sequence_info.name.as_str() {
        "Arithmetic" => Some(Arithmetic::new(request.parameters[0], request.parameters[1])),
        "Geometric" => Some(Geometric::new(request.parameters[0], request.parameters[1])),
        "Constant" => Some(Constant::new(request.parameters[0])),
        "Sum" => {
            let seq1 = create_sequence_from_syntax(&request.sequences[0]);
            let seq2 = create_sequence_from_syntax(&request.sequences[1]);
            Some(Sum::new(seq1, seq2))
        }
        "Prod" => {
            let seq1 = create_sequence_from_syntax(&request.sequences[0]);
            let seq2 = create_sequence_from_syntax(&request.sequences[1]);
            Some(Prod::new(seq1, seq2))
        }
        "Drop" => {
            let seq = create_sequence_from_syntax(&request.sequences[0]);
            Some(Drop::new(seq, request.parameters[0] as usize))
        }
        "LinComb" => {
            let seq1 = create_sequence_from_syntax(&request.sequences[0]);
            let seq2 = create_sequence_from_syntax(&request.sequences[1]);
            Some(LinComb::new(request.parameters[0], request.parameters[1], request.parameters[2], seq1, seq2))
        }
        "Recursive" => Some(Recursive::new(
            request.parameters[0], request.parameters[1], request.parameters[2], request.parameters[3])),
        "Average" => {
            let seq1 = create_sequence_from_syntax(&request.sequences[0]);
            let seq2 = create_sequence_from_syntax(&request.sequences[1]);
            Some(Average::new(seq1, seq2))
        }
        "Cyclic" => {
            let seq = create_sequence_from_syntax(&request.sequences[0]);
            Some(Cyclic::new(seq, request.parameters[0] as usize))
        }
        "Alternating" => {
            let seq = create_sequence_from_syntax(&request.sequences[0]);
            Some(Alternating::new(seq))
        }
        "Smoothed" => {
            let seq = create_sequence_from_syntax(&request.sequences[0]);
            Some(Smoothed::new(seq))
        }
        _ => None,
    };

    return match sequence {
        Some(seq) => {
            println!("{:?}", Response::new(full(serde_json::to_string(&seq.range(request1.range)).unwrap())));
            Ok(Response::new(full(serde_json::to_string(&seq.range(range)).unwrap())))
            // let mut builder = Response::builder()
            // .status(200)
            // .body(BoxBody::new(serde_json::to_string(&seq.range(request1.range)).unwrap()));
            // builder
        },
        None => {
            let mut builder = Response::builder()
            .status(200)
            .body(BoxBody::new(delegate(request1, &sequence_info.name).await.unwrap()));
            builder
        }};
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
        "Recursive" => Recursive::new(syntax.parameters[0], syntax.parameters[1], syntax.parameters[2], syntax.parameters[3]),
        "Average" => {
            let seq1 = create_sequence_from_syntax(&syntax.sequences[0]);
            let seq2 = create_sequence_from_syntax(&syntax.sequences[1]);
            Average::new(seq1, seq2)
        }
        "Cyclic" => {
            let seq = create_sequence_from_syntax(&syntax.sequences[0]);
            Cyclic::new(seq, syntax.parameters[0] as usize)
        }
        "Alternating" => {
            let seq = create_sequence_from_syntax(&syntax.sequences[0]);
            Alternating::new(seq)
        }
        "Smoothed" => {
            let seq = create_sequence_from_syntax(&syntax.sequences[0]);
            Smoothed::new(seq)
        }
        _ => panic!("Unsupported sequence")
    }
}

async fn delegate(request: SequenceRequest, sequence_name: &str) -> Result<String, hyper::Error> {
    
    let b = match send_get("http://127.0.0.1:7878/project".to_string()).await {
        Ok(resp) => resp,
        Err(e) => panic!("{}", e),
    };
    let projects: serde_json::Value = serde_json::from_str(&b).unwrap();
    let length = projects.as_array().unwrap().len();

    let body = match serde_json::to_string(&request) {
        Ok(req) => req,
        Err(e) => panic!("{}", e),
    };

    if length > 0 {
        for i in 0..length {
            let ip = &projects[i]["ip"].to_string().replace("\"", "");
            // let port = &values[i]["port"];
            let port = 9000;
            println!("ip:port = {}:{}", &ip, &port);

            if ip != "127.0.0.1" || port != PORT {
                let url = format!("http://{}:{}/sequence", ip, port);
                let data = match send_get(url.clone()).await {
                    Ok(resp) => resp,
                    Err(e) => panic!("{}", e),
                };
                println!("{:?}", data);

                let seqs: serde_json::Value = serde_json::from_str(&data).unwrap();
                for i in 0..seqs.as_array().unwrap().len() {
                    if seqs[i]["name"] == sequence_name {
                        let url_of_seq = format!("{}/{}", &url, sequence_name);
                        return match send_post(url_of_seq, body).await {
                            Ok(resp) => Ok(resp),
                            Err(e) => panic!("{}", e),
                        };
                        // return match serde_json::from_str(&wanted_seq) {
                        //     Ok(json) => json,
                        //     Err(e) => panic!("{}", e),
                        // };
                    };
                }
            }
        }
        panic!("Nobody has this sequence")
    } else {
        panic!("Nobody has this sequence")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = ([127, 0, 0, 1], PORT).into();

    let b = send_get("http://127.0.0.1:7878/project".to_string()).await?;
    println!("HERE GET {}", b);

    let b = send_post(
        "http://127.0.0.1:7878/project".to_string(),
        serde_json::to_string(&get_project()).unwrap(),
    ).await?;    
    println!("HERE POST {}", b);

    let b = send_get("http://127.0.0.1:7878/project".to_string()).await?;
    println!("HERE GET {}", b);

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

        tokio::task::spawn(async move {
            let service = service_fn(move |req| {
                async move {
                    match (req.method(), req.uri().path()) {
                        (&Method::GET, "/ping") => Ok::<_, Error>(Response::new(full(
                            serde_json::to_string(&get_project()).unwrap(),
                        ))),
                        (&Method::GET, "/sequence") => {
                            let sequences = sequences();
                            let data = serde_json::to_string(&sequences).unwrap();
                            Ok(Response::new(full(data,
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
    });  
    }
}