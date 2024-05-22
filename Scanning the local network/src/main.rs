use hostname::get;
use oui::OuiDatabase;
use pnet::datalink::{Channel, MacAddr, NetworkInterface};
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::EtherTypes;
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::packet::{MutablePacket, Packet};
use pnet::datalink;
use std::time::{Duration, Instant};
use std::{io, net::{IpAddr, Ipv4Addr}};
use std::{sync::{mpsc, Arc, Mutex}, thread,};
use eui48::MacAddress;

fn count_addresses(mask: IpAddr) -> u32 {
    match mask {
        IpAddr::V4(ipv4) => {
            let bits: u32 = ipv4.into();
            2u32.pow(32 - bits.count_ones())
        }
        _ => 0
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}


type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}


impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for i in 0..size {
            workers.push(Worker::new(i, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender: Some(sender) }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker {
    fn new(id:usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || 
                loop {
                    let message = receiver.lock().unwrap().recv();

                    match message {
                        Ok(job) => {
                            job();
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
    );
        Worker{id, thread: Some(thread)}
    }

}


fn send_arp_request(inter: &NetworkInterface){
    let mut source_ip = inter
    .ips
    .iter()
    .find(|ip| ip.is_ipv4())
    .map(|ip| match ip.ip() {
        IpAddr::V4(ip) => ip,
        _ => unreachable!(),
    })
    .unwrap();

    let mut ip_bytes = source_ip.octets();
    ip_bytes[3] = 1; 
    let mut modified_ip: Ipv4Addr = Ipv4Addr::from(ip_bytes);

    let pool = ThreadPool::new(10);
    for i in 0..count_addresses(inter.ips[0].mask()) - 1{
        let interf = inter.clone();
        pool.execute(move || {
            let (mut sender, mut receiver) = match pnet::datalink::channel(&interf, Default::default()) {
                Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
                Ok(_) => panic!("Unknown channel type"),
                Err(e) => panic!("Error happened {}", e),
            };

            let mut ethernet_buffer = [0u8; 42];
            let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buffer).unwrap();
        
            ethernet_packet.set_destination(MacAddr::broadcast());
            ethernet_packet.set_source(interf.mac.unwrap());
            ethernet_packet.set_ethertype(EtherTypes::Arp);

            let mut arp_buffer = [0u8; 28];
            let mut arp_packet = MutableArpPacket::new(&mut arp_buffer).unwrap();
        
            arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
            arp_packet.set_protocol_type(EtherTypes::Ipv4);
            arp_packet.set_hw_addr_len(6);
            arp_packet.set_proto_addr_len(4);
            arp_packet.set_operation(ArpOperations::Request);
            arp_packet.set_sender_hw_addr(interf.mac.unwrap());
            arp_packet.set_sender_proto_addr(source_ip);
            arp_packet.set_target_hw_addr(MacAddr::zero());
            arp_packet.set_target_proto_addr(Ipv4Addr::from(u32::from(modified_ip) + i - 1));
           
        
            ethernet_packet.set_payload(arp_packet.packet_mut());
        
            sender
                .send_to(ethernet_packet.packet(), None)
                .unwrap()
                .unwrap();
        
            let timeout_duration = Duration::from_secs(2); // Пример: тайм-аут в 5 секунд

            let start_time = Instant::now();
            // println!("{}", Ipv4Addr::from(u32::from(modified_ip) + i - 1));
            loop {
                if Instant::now() - start_time > timeout_duration {
                   println!("Тайм-аут. Ответ не получен. ({}) ", Ipv4Addr::from(u32::from(modified_ip) + i - 1));
                   break;
                }
                let buf = receiver.next().unwrap();
                let arp = ArpPacket::new(&buf[MutableEthernetPacket::minimum_packet_size()..]).unwrap();
                if arp.get_sender_proto_addr() == Ipv4Addr::from(u32::from(modified_ip) + i - 1)
                && arp.get_target_hw_addr() == interf.mac.unwrap()
                {
                     let oui_db = OuiDatabase::new_from_file("D:/Uni/Labs/ksisproject/src/dbM.txt").expect("База данных не открылась");
                     print!("Получен ответ! - {} - {}", arp.get_sender_hw_addr(), arp.get_sender_proto_addr());
                     let vender = oui_db.query_by_mac(&MacAddress::new(arp.get_sender_hw_addr().octets())).unwrap();
                     if vender == None {
                        println!("; производитель - неизвестен")
                     } else {
                         println!("; производитель - {:?}", vender.unwrap().name_long.unwrap());
                     }
                     break;
                }
            }});
    }

}

fn main(){
    println!("Interfaces by PNET lib:");
    // Getting all of available interfaces by pnet
    let interfaces = datalink::interfaces();
    for interface in interfaces {
        println!("MAC-adress: {:?};  Description: {}", interface.mac.unwrap(), interface.description);
    }
    println!("-------------------------------------------------------------------------------------------");
    println!("Выберите интерфейс для ARP: ");
    let mut user_inp = String::new();
    io::stdin().read_line(&mut user_inp).expect("Ошибка при считывании ввода");
    let num_int:usize = user_inp.trim().parse().unwrap(); 
    let interfaces = datalink::interfaces();
    let selected_interface = &interfaces[num_int.wrapping_sub(1)];
    println!("Данные этого устройства: MAC-адрес: {:?}; имя: {:?}", selected_interface.mac.unwrap(), get().unwrap());
    println!("Начало опроса выбранного сетевого интерфейса ({}; маска подсети: {})!", selected_interface.description, selected_interface.ips[0].mask());
    // Starting ARP
    send_arp_request(selected_interface);
}
