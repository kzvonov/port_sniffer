use std::env;
use std::io::{self, Write};
use std::net::IpAddr;
use std::net::ToSocketAddrs;
use std::net::{SocketAddr, TcpStream};
use std::process;
use std::str::FromStr;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use url::Host;

const MAX: u16 = 65535;

struct Arguments {
    ipaddr: IpAddr,
    threads: u16,
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &str> {
        if args.len() < 1 {
            return Err("Not enough arguments");
        } else if args.len() > 4 {
            return Err("Too many arguments");
        }

        let raw_ip: &str = &args[args.len() - 1].clone();
        let ipaddr: IpAddr = match get_ip_addr(raw_ip) {
            Ok(value) => value,
            Err(e) => return Err(e),
        };

        let flag = args[1].clone();
        if flag.contains("-h") || flag.contains("--help") {
            println!("Usage:\r\n-j to select how many threads you want\r\n-h or --help to show this help message");
            return Err("help");
        } else if flag.contains("-j") {
            let threads = match args[2].parse::<u16>() {
                Ok(value) => value,
                Err(_) => return Err("Failed to parse number of threads"),
            };
            return Ok(Arguments { threads, ipaddr });
        }

        return Err("Something is wrong with arguments, call with the -h flag for help");
    }
}

fn get_ip_addr(value: &str) -> Result<IpAddr, &'static str> {
    if let Ok(ip) = IpAddr::from_str(value) {
        return Ok(ip);
    }

    let host = match Host::parse(value) {
        Ok(s) => s,
        Err(_) => return Err("Not a valid IPADDR; must be IPv4 or IPv6"),
    };

    let mut addrs = match (host.to_string(), 0).to_socket_addrs() {
        Ok(addrs) => addrs,
        Err(_) => return Err("Failed to resolve ipaddr"),
    };

    addrs
        .find_map(|addr| Some(addr.ip()))
        .ok_or("Failed to resolve ipaddr")
}

fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_threads: u16) {
    let mut port: u16 = start_port + 1;
    loop {
        let socket_addr = SocketAddr::new(addr, port);
        match TcpStream::connect_timeout(&socket_addr, std::time::Duration::from_millis(500)) {
            Ok(_) => {
                print!(".");
                io::stdout().flush().unwrap();
                tx.send(port).unwrap();
            }
            Err(_) => {
                io::stdout().flush().unwrap();
            }
        }
        if MAX - port <= num_threads {
            break;
        }
        port += num_threads;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program: String = args[0].clone();

    let arguments = Arguments::new(&args).unwrap_or_else(|err| {
        if err.contains("help") {
            process::exit(0);
        } else {
            eprint!("{} problem parsing arguments: {}\n\n", program, err);
            process::exit(1);
        }
    });

    println!("Scanning the ipaddr: {}", arguments.ipaddr);

    let num_threads = arguments.threads;
    let (tx, rx) = channel();
    for i in 0..num_threads {
        let tx = tx.clone();
        thread::spawn(move || {
            scan(tx, i, arguments.ipaddr, num_threads);
        });
    }

    let mut out = vec![];
    drop(tx);
    for p in rx {
        out.push(p);
    }

    println!("");
    out.sort();
    for v in out {
        println!("Port {} is open", v);
    }
}
