use futures::{join, select};
use regions::Region;
use semtech_udp::{
    pull_resp,
    push_data::RxPk,
    server_runtime::{ClientTx, Event, UdpRuntime},
    MacAddress, StringOrNum,
};
use std::net::SocketAddr;
use structopt::StructOpt;
use tokio::time::{delay_for as sleep, Duration, Instant};
use tokio::{
    sync::{mpsc, oneshot},
    time::timeout,
};

async fn start_server(
    port: u16,
    mut sender: mpsc::Sender<(RxPk, MacAddress)>,
) -> Result<(oneshot::Receiver<MacAddress>, ClientTx), Box<dyn std::error::Error>> {
    let test_addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Starting server: {}", test_addr);

    // Splitting is optional and only useful if you are want to run concurrently
    // the client_rx & client_tx can both be held inside the UdpRuntime struct
    let (mut test_client_rx, mut test_client_tx) = UdpRuntime::new(test_addr).await?.split();

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
                    sender.send((rxpk, addr)).await.unwrap();
                }
                Event::NoClientWithMac(_packet, mac) => {
                    println!("Tried to send to client with unknown MAC: {:?}", mac)
                }
                Event::RawPacket(_) => (),
            }
        }
    });

    Ok((test_rx, test_client_tx))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Opt::from_args();
    let (packet_tx, mut packet_rx): (
        mpsc::Sender<(RxPk, MacAddress)>,
        mpsc::Receiver<(RxPk, MacAddress)>,
    ) = mpsc::channel(120);

    let (test_mac, mut test_tx) = start_server(cli.test_port, packet_tx.clone()).await?;
    let (control_mac, mut control_tx) = start_server(cli.control_port, packet_tx).await?;

    println!("Blocking until both clients connect");
    let (gateway_mac, control_mac) = join!(test_mac, control_mac);
    let (gateway_mac, control_mac) = (gateway_mac.unwrap(), control_mac.unwrap());

    let channels = cli.region.get_uplink_frequencies();

    for (index, channel) in channels.iter().enumerate() {
        println!(
            "Dispatching on channel ({:?} {}: {} MHz)",
            cli.region,
            index + 1,
            channel
        );
        let txpk = create_packet(channel, "SF12BW125");
        println!("{:?}", txpk);
        let prepared_send = test_tx.prepare_downlink(Some(txpk.clone()), gateway_mac);
        if let Err(e) = prepared_send.dispatch(Some(Duration::from_secs(5))).await {
            panic!("Transmit Dispatch threw error: {:?}", e)
        } else {
            println!("Send complete");
        }

        let start = Instant::now();
        let wait_for = Duration::from_secs(10);
        let mut passed = false;
        while Instant::now().duration_since(start) < wait_for && !passed {
            let (rxpk, mac) = timeout(wait_for, packet_rx.recv())
                .await?
                .expect("Channels should never close");

            if mac == control_mac
                && rxpk.get_data() == txpk.data
                && rxpk.get_datarate() == txpk.datr
                && (rxpk.get_frequency() - &txpk.freq).abs() < 0.1
            {
                println!("Received expected packet!");
                passed = true;
            } else {
                println!("Received garbage packet {:?}", rxpk);
            }
        }
    }
    Ok(())
}

fn create_packet(channel: &usize, datr: &str) -> pull_resp::TxPk {
    let buffer = vec![0; 32];
    let size = buffer.len() as u64;
    let data = base64::encode(buffer);
    let tmst = StringOrNum::N(0);
    let freq = *channel as f64 / 1_000_000.0;

    pull_resp::TxPk {
        imme: true,
        tmst,
        freq,
        rfch: 0,
        powe: 12, //cli.power as u64,
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

    #[structopt(long, short)]
    region: Region,
}
