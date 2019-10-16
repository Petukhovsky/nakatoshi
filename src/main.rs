mod address;

use std::time::Instant;
use std::sync::mpsc;
use std::{ thread };
use std::sync::{ Arc, atomic };

use address::Address;
use indicatif::{ MultiProgress, ProgressBar, HumanDuration };

fn calculate_address (starts_with: &str, should_stop: &atomic::AtomicBool, cpu_num: usize, spinner: ProgressBar) -> Address {
    let mut address = Address::new();

    while !address.starts_with(starts_with) && !should_stop.load(atomic::Ordering::Relaxed) {
        address = Address::new();
        let message = format!("CPU {}: Finding vanity address {}", cpu_num, address.address.to_string());
        spinner.set_message(&message)
    }

    spinner.finish_and_clear();
    return address
}

fn main() {
    let started_at = Instant::now();
    let cpus = num_cpus::get();
    let multi_progress_bar = MultiProgress::new();
    let matches = clap::App::new("Bitcoin vanity address generator")
        .version("0.1.0")
        .about("This tool creates a set of Bitcoin mainnet private, public key and vanity address")
        .author("ndelvalle <nicolas.delvalle@gmail.com>")
        .arg(clap::Arg::with_name("startswith")
            .required(true)
            .takes_value(true)
            .index(1)
            .help("Address starts with")
        )
        .get_matches();

    let starts_with = String::from(matches.value_of("startswith").unwrap());
    let (tx, rx) = mpsc::channel();
    let has_finished = Arc::new(atomic::AtomicBool::new(false));

    for cpu_num in 0..cpus {
        let progress_bar = multi_progress_bar.add(ProgressBar::new_spinner());
        let starts_with = starts_with.clone();
        let should_stop = has_finished.clone();
        let tx = tx.clone();

        thread::spawn(move || {
            let found_address = calculate_address(&starts_with, &should_stop, cpu_num + 1, progress_bar);
            should_stop.store(true, atomic::Ordering::Relaxed);

            tx.send(found_address).unwrap();
        });
    }

    multi_progress_bar.join().unwrap();
    let address = rx.recv().unwrap();

    println!("Private key:  {}", address.private_key);
    println!("Public key:   {}", address.public_key);
    println!("Address:      {}", address.address);
    println!("Time elapsed: {}", HumanDuration(started_at.elapsed()));
}
