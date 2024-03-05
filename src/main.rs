use clap::Parser;
use std::{clone, fs};
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

	// Creazione delle cartelle nel caso non esistano
	let input_folder_result = fs::create_dir_all(input_folder.clone());
	if input_folder_result.is_ok() {
		println!("Cartella input creata: {}", input_folder)
	} else {
		panic!("Cartella input non creata: {}", input_folder)
	}

	let output_folder_result = fs::create_dir_all(output_folder.clone());
	if output_folder_result.is_ok() {
		println!("Cartella output creata: {}", output_folder)
	} else {
		panic!("Cartella output non creata: {}", output_folder)
	}

	// Instanzio il lucchetto Mutex che userò per accedere al contatore
	// Devo usare Atomic reference counting, in questo caso è il migliore per gestire la concorrenza
	let mutex_lock = Arc::new(Mutex::new(0));

	// Devo impostare il valore iniziale del contatore, quindi prendo il contatore con lock
	let mut contatore = mutex_lock.lock().unwrap();
	// Imposto il valore del contatore
	*contatore = threadcount;
	// Cancello la variabile contatore con drop. Questo è l'equivalente di lock, la funzione si limita a cancellare la variabile
	drop(contatore);

	// Creo un vettore mutabile che conterrà tutte le informazioni di tutti i thread. Esso serve per effettuare il join, altrimenti il programma esce prima di aver finito tutti i thread
	let mut thread_vector: Vec<thread::JoinHandle<()>> = vec![];

	// In questo ciclo for vengono fatti partire i thread, non verranno eseguiti in ordine ovviamente
	for numero in 0..num_cpus::get() {
		// Creo una copia del lock
		let mutex_lock = Arc::clone(&mutex_lock);

		// Qui metto in nuovo thread nell'array, è come una lista.
		thread_vector.push(thread::spawn(move || {
			// Instanzio il contatore protetto dal mutex_lock e chiudo il lucchetto
			let mut contatore = mutex_lock.lock().unwrap();

			// Effettuo l'operazione voluta
			println!(
				"Io sono il processo figlio {}, {}->{}",
				numero,
				*contatore,
				*contatore + 1
			);
			*contatore += 1;
		}));
	}

	// In questo ciclo faccio il join dei thread, altrimenti il programma padre termina prima dei thread
	for thread_element in thread_vector {
		let _ = thread_element.join();
	}

	println!("Risultato finale = {}", *mutex_lock.lock().unwrap());
	println!("Risultato aspettato = {}", 2 * num_cpus::get());
}

/*
fn esegui_comando(numero: usize) {

	let comando_string:String = format!("echo {}", numero);

	let comando = Command::new("sh")
	.arg("-c")
	.arg(comando_string)
	.output()
	.expect("Non sono riuscito ad eseguire il processo");

	println!("{:?}", comando);
}
*/
