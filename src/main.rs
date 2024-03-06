use clap::Parser;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::{self};

static DEFAULT_INPUT:&str = "./input";
static DEFAULT_OUTPUT:&str = "./output";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	// Numero dei thread
	#[arg(short, long, default_value_t = num_cpus::get())]
	threadcount: usize,

	/// cartella input
	#[arg(short, long, default_value = DEFAULT_INPUT)]
	input: String,

	/// cartella output
	#[arg(short, long, default_value = DEFAULT_OUTPUT)]
	output: String,
}

fn main() {

	// Instazio la variabile contenente gli argomenti
	let args = Args::parse();
	let threadcount:usize = args.threadcount;
	let input_folder:String = args.input;
	let output_folder:String = args.output;

	// Controllo se esiste la cartella di input
	if !Path::new(input_folder.clone().as_str()).exists() {
		panic!("La cartella di input non esiste")
	}

	// Creo la cartella di output nel caso non esiste, se esiste continua
	let output_folder_result = fs::create_dir_all(output_folder.clone());
	if output_folder_result.is_ok() || Path::new(output_folder.clone().as_str()).exists() {
		println!("Cartella output creata: {}", output_folder)
	} else {
		panic!("Cartella output non creata: {}", output_folder)
	}

	// Creo una lista iteratore di stringhe con le canzoni
	let mut lista_canzoni = fs::read_dir(input_folder.clone()).unwrap();

	// Instanzio il lucchetto Mutex che usero per accedere al contatore della lista
	let mutex_lock = Arc::new(Mutex::new(lista_canzoni));

	// Vettore mutabile per gestire i thread, specialmente la parte di join
	let mut thread_vector: Vec<thread::JoinHandle<()>> = vec![];

	for _ in 0..threadcount {
		// Creo una copia del lock
		let mutex_lock = Arc::clone(&mutex_lock);

		// Qui metto in nuovo thread nell'array, è come una lista.
		thread_vector.push(thread::spawn(move || {

			// Instanzio il contatore protetto dal mutex_lock e chiudo il lucchetto
			let mut lista_canzoni = mutex_lock.lock().unwrap();
			
			let canzone = (*lista_canzoni).next().unwrap().unwrap().path();
			
		}));
	}

	// In questo ciclo faccio il join dei thread, altrimenti il programma padre termina prima dei thread
	for thread_element in thread_vector {
		let _ = thread_element.join();
	}

}