use clap::Parser;
use std::borrow::Borrow;
use std::fs;
use std::ops::Deref;
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
	let lista_canzoni = fs::read_dir(input_folder.clone()).unwrap();
	let mut array_canzoni:Vec<String> = vec!["".to_string()];
	// La converto in un array, altrimenti non riesco a passarla ai thread
	for elemento in lista_canzoni {
		array_canzoni.push(elemento.unwrap().file_name().into_string().unwrap())
	}


	// Perndo il numero di canzoni nella cartella
	let numero_canzoni = array_canzoni.capacity();

	// Instanzio il lucchetto Mutex che usero per accedere al contatore della lista, 
	let mutex_lock = Arc::new(Mutex::new(0));

	// Vettore mutabile per gestire i thread, specialmente la parte di join
	let mut thread_vector: Vec<thread::JoinHandle<()>> = vec![];

	for _ in 0..threadcount {
		// Creo una copia del lock
		let mutex_lock = Arc::clone(&mutex_lock);

		// Qui metto in nuovo thread nell'array, è come una lista.
		thread_vector.push(thread::spawn(|| {

			// Ciclo che passa tutte le canzoni se necessario, quasi garantito che esca prima
			for _ in 0..numero_canzoni {
				// Instanzio il contatore protetto dal mutex_lock e chiudo il lucchetto
				let mut contatore = mutex_lock.lock().unwrap();
				if *contatore > numero_canzoni {
					drop(contatore);
					break;
				}
				
				// Copio il valore del contatore prima di aumentarlo per il prossimo thread
				let posizione = *contatore;
				*contatore = *contatore + 1;
				drop(contatore);

				let nome_canzone:String = array_canzoni[posizione].clone();

			}
		}));
	}

	// In questo ciclo faccio il join dei thread, altrimenti il programma padre termina prima dei thread
	for thread_element in thread_vector {
		let _ = thread_element.join();
	}

}