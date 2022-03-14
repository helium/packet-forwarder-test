use futures::join;
use regions::Region;
use semtech_udp::{
    pull_resp,
    push_data::RxPk,
    server_runtime::{ClientTx, Event, UdpRuntime},
    MacAddress, StringOrNum,
};
use std::net::SocketAddr;
use structopt::StructOpt;
use tokio::time::{Duration, Instant};
use tokio::{
    sync::{mpsc, oneshot},
    time::timeout,
};

#[derive(Debug, Clone, PartialEq)]
enum Role {
    Tested,
    Control,
}

type Message = (RxPk, MacAddress, Role);

async fn start_server(
    role: Role,
    port: u16,
    mut sender: mpsc::Sender<Message>,
    debug: bool,
    label: &'static str,
) -> Result<(oneshot::Receiver<MacAddress>, ClientTx), Box<dyn std::error::Error>> {
    let test_addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Starting server: {}", test_addr);

    // Splitting is optional and only useful if you are want to run concurrently
    // the client_rx & client_tx can both be held inside the UdpRuntime struct
    let (mut test_client_rx, test_client_tx) = UdpRuntime::new(test_addr).await?.split();

    // prepare a one-shot so that receive can unlocked sending
    let (test_tx, test_rx): (oneshot::Sender<MacAddress>, oneshot::Receiver<MacAddress>) =
        oneshot::channel();

    let mut test_tx = Some(test_tx);

    tokio::spawn(async move {
        loop {
            match test_client_rx.recv().await {
                Event::UnableToParseUdpFrame(buf) => {
                    println!("Semtech UDP Parsing Error");
                    println!("UDP data: {:?}", buf);
                }
                Event::NewClient((mac, addr)) => {
                    println!("New packet forwarder client: {}, {}", mac, addr);

                    // unlock the tx thread by sending it the gateway mac of the
                    // the first client (connection via PULL_DATA frame)
                    if let Some(tx) = test_tx.take() {
                        tx.send(mac).unwrap();
                    }
                }
                Event::UpdateClient((mac, addr)) => {
                    println!("Mac existed, but IP updated: {}, {}", mac, addr);
                }
                Event::PacketReceived(rxpk, addr) => {
                    sender.send((rxpk, addr, role.clone())).await.unwrap();
                }
                Event::NoClientWithMac(_packet, mac) => {
                    println!("Tried to send to client with unknown MAC: {:?}", mac)
                }
                Event::RawPacket(packet) => {
                    if debug {
                        println!("{}: {:?}", label, packet);
                    }
                }
            }
        }
    });

    Ok((test_rx, test_client_tx))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Opt::from_args();
    let (packet_tx, mut packet_rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) =
        mpsc::channel(120);

    let (test_mac, mut test_tx) = start_server(
        Role::Tested,
        cli.test_port,
        packet_tx.clone(),
        cli.debug,
        "Test",
    )
    .await?;
    let (control_mac, mut control_tx) = start_server(
        Role::Control,
        cli.control_port,
        packet_tx,
        cli.debug,
        "Control",
    )
    .await?;

    println!("Blocking until both clients connect");
    let (test_mac, control_mac) = join!(test_mac, control_mac);
    let (test_mac, control_mac) = (test_mac.unwrap(), control_mac.unwrap());

    println!("Testing ability of Test Gateway to Transmit on Uplink Channels");
    run_test(
        Role::Control,
        &cli,
        &mut test_tx,
        &mut packet_rx,
        &test_mac,
        &control_mac,
    )
    .await?;
    println!("Testing ability of Test Gateway to Receive on Uplink Channels");
    run_test(
        Role::Tested,
        &cli,
        &mut control_tx,
        &mut packet_rx,
        &control_mac,
        &test_mac,
    )
    .await?;

    Ok(())
}

async fn run_test(
    receiver_role: Role,
    cli_options: &Opt,
    test_tx: &mut ClientTx,
    receiver: &mut mpsc::Receiver<Message>,
    test_mac: &MacAddress,
    control_mac: &MacAddress,
) -> Result<(), Box<dyn std::error::Error>> {
    let power = cli_options.power;
    let channels = cli_options.region.get_uplink_frequencies();

    for (index, channel) in channels.iter().enumerate() {
        println!(
            "\tDispatching on channel ({:?} {}: {} MHz)",
            cli_options.region,
            index + 1,
            channel
        );
        let txpk = create_packet(channel, &cli_options.datr, power);

        let prepared_send = test_tx.prepare_downlink(Some(txpk.clone()), *test_mac);
        if let Err(e) = prepared_send.dispatch(Some(Duration::from_secs(5))).await {
            panic!("Transmit Dispatch threw error: {:?}", e)
        }

        let start = Instant::now();
        let wait_for = Duration::from_secs(10);
        let mut passed = false;
        while Instant::now().duration_since(start) < wait_for && !passed {
            let (rxpk, mac, role) = timeout(wait_for, receiver.recv())
                .await?
                .expect("Channels should never close");

            if mac == *control_mac
                && role == receiver_role
                && rxpk.get_data() == txpk.data
                && rxpk.get_datarate() == txpk.datr
                && (rxpk.get_frequency() - txpk.freq).abs() < 0.1
            {
                println!(
                    "\tReceived expected packet! RSSI = {}, SNR = {}",
                    rxpk.get_rssi(),
                    rxpk.get_snr()
                );
                passed = true;
            }
        }
    }
    Ok(())
}

fn create_packet(channel: &usize, datr: &str, power: u64) -> pull_resp::TxPk {
    let buffer = vec![0; 52];
    let size = buffer.len() as u64;
    let data = base64::encode(buffer);
    let tmst = StringOrNum::N(0);
    let freq = *channel as f64 / 1_000_000.0;

    pull_resp::TxPk {
        imme: true,
        tmst,
        freq,
        rfch: 0,
        powe: power,
        modu: "LORA".into(),
        datr: datr.into(),
        codr: "4/5".into(),
        ipol: false,
        size,
        data,
        tmms: None,
        fdev: None,
        prea: None,
        ncrc: None,
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "semtech-server", about = "LoRa test device utility")]
pub struct Opt {
    /// Port to run service on
    #[structopt(long, default_value = "1680")]
    test_port: u16,

    /// Port to run service on
    #[structopt(long, default_value = "1681")]
    control_port: u16,

    /// which region to use for the RF test (eg: EU868, US915...)
    #[structopt(long, short)]
    region: Region,

    /// output all UDP frames received from both control and test gateways
    #[structopt(long, short)]
    debug: bool,

    /// transmit power. allowable range, 12-28
    #[structopt(long, default_value = "12")]
    power: u64,

    /// data rate
    #[structopt(long, default_value = "SF12BW125")]
    datr: String,
}
